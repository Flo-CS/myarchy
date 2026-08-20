use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use crate::error::AppError;
use crate::models::monitor::Monitor;

use super::notifierctl::Notifier;
use super::osdctl::Osd;

pub(crate) trait Brightness {
    fn get(&self) -> Result<i64>;
    fn set(&self, percent: i64) -> Result<()>;
    fn step(&self, delta: &str, osd: &dyn Osd) -> Result<()>;
}

fn is_internal(name: &str) -> bool {
    name.starts_with("eDP-") || name.starts_with("LVDS-") || name.starts_with("DSI-")
}

pub(crate) fn resolve_adapter(name: &str, monitors: &[Monitor]) -> Result<Box<dyn Brightness>> {
    try_resolve_adapter(name, monitors)?.ok_or_else(|| {
        AppError::DdcNotResponding {
            name: name.to_string(),
        }
        .into()
    })
}

pub(crate) fn try_resolve_adapter(
    name: &str,
    monitors: &[Monitor],
) -> Result<Option<Box<dyn Brightness>>> {
    if is_internal(name) {
        return Ok(Some(Box::new(Backlight {
            name: name.to_string(),
        })));
    }
    Ok(Ddc::display_for(name, monitors)?.map(|display| {
        Box::new(Ddc {
            name: name.to_string(),
            display,
        }) as Box<dyn Brightness>
    }))
}

pub(crate) fn run_worker(notify: &dyn Notifier, name: &str, monitors: &[Monitor]) -> Result<()> {
    Ddc::run_worker(notify, name, monitors)
}

struct Backlight {
    name: String,
}

impl Brightness for Backlight {
    fn get(&self) -> Result<i64> {
        let out = Command::new("brightnessctl")
            .args(["--class=backlight", "--machine-readable"])
            .output()?;
        let text = String::from_utf8_lossy(&out.stdout);
        // "intel_backlight,backlight,60000,50%,120000"
        let line = text.lines().next().unwrap_or("");
        let pct = line.split(',').nth(3).unwrap_or("").trim_end_matches('%');
        pct.parse::<i64>()
            .context("could not read backlight brightness")
    }

    fn set(&self, percent: i64) -> Result<()> {
        let out = Command::new("brightnessctl")
            .args(["--class=backlight", "set", &format!("{percent}%")])
            .output()?;
        if !out.status.success() {
            bail!("brightnessctl failed to set backlight brightness");
        }
        Ok(())
    }

    fn step(&self, delta: &str, osd: &dyn Osd) -> Result<()> {
        osd.show_brightness_progress(&self.name, delta)
    }
}

struct Ddc {
    name: String,
    display: String,
}

impl Ddc {
    fn strip_card_prefix(connector: &str) -> String {
        if let Some(rest) = connector.strip_prefix("card") {
            if let Some(dash) = rest.find('-') {
                let (digits, _) = rest.split_at(dash);
                if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) {
                    return rest[dash + 1..].to_string();
                }
            }
        }
        connector.to_string()
    }

    fn cache_path() -> PathBuf {
        super::xdg::runtime_dir().join("screen").join("ddc-map")
    }

    /// `ddcutil detect` probes the buses and takes ~0.4s, too slow to call per
    /// lookup. Cache the connector -> display map, keyed on the connected screens
    /// so plugging one in invalidates it by itself.
    ///
    /// Laptop panels show up as "Invalid display" blocks that still carry a
    /// connector line, so only record a mapping when the block actually had a
    /// number.
    fn map(monitors: &[Monitor]) -> Result<HashMap<String, String>> {
        let mut names: Vec<&str> = monitors.iter().map(|m| m.name.as_str()).collect();
        names.sort_unstable();
        let key = names.join(",");

        let cache_path = Self::cache_path();
        let stale = match fs::read_to_string(&cache_path) {
            Ok(content) => content.lines().next() != Some(key.as_str()),
            Err(_) => true,
        };

        if stale {
            let mut out = String::new();
            out.push_str(&key);
            out.push('\n');

            if let Ok(detect) = Command::new("ddcutil").args(["detect", "--brief"]).output() {
                let text = String::from_utf8_lossy(&detect.stdout);
                let mut display: Option<String> = None;
                for line in text.lines() {
                    if line.starts_with("Display ") {
                        display = line.split_whitespace().nth(1).map(|s| s.to_string());
                    } else if line.starts_with("Invalid display") {
                        display = None;
                    } else if line.contains("DRM connector:") {
                        if let Some(d) = &display {
                            if let Some(connector_raw) = line.split_whitespace().nth(2) {
                                let connector = Self::strip_card_prefix(connector_raw);
                                out.push_str(&format!("{connector}\t{d}\n"));
                            }
                        }
                    }
                }
            }

            fs::create_dir_all(cache_path.parent().unwrap())?;
            fs::write(&cache_path, out)?;
        }

        let content = fs::read_to_string(&cache_path).unwrap_or_default();
        Ok(content
            .lines()
            .skip(1)
            .filter_map(|l| l.split_once('\t'))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect())
    }

    fn display_for(name: &str, monitors: &[Monitor]) -> Result<Option<String>> {
        Ok(Self::map(monitors)?.get(name).cloned())
    }

    fn target_file(name: &str) -> PathBuf {
        super::xdg::runtime_dir()
            .join("screen")
            .join(format!("brightness-target-{name}"))
    }

    fn lock_file(name: &str) -> PathBuf {
        super::xdg::runtime_dir()
            .join("screen")
            .join(format!("brightness-lock-{name}"))
    }

    /// Serializes writes to one display: a burst of key presses only ever queues
    /// the latest target, so it converges in one trailing write instead of one
    /// per press.
    fn spawn_worker(name: &str) -> Result<()> {
        let exe = std::env::current_exe()?;
        Command::new(exe)
            .arg("screen")
            .arg("brightness-worker")
            .arg(name)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        Ok(())
    }

    /// A non-blocking lock means a press during an in-flight apply just updates
    /// the target file and returns — the running worker picks it up on its next
    /// loop instead of a second worker racing it.
    fn run_worker(notify: &dyn Notifier, name: &str, monitors: &[Monitor]) -> Result<()> {
        let file_path = Self::lock_file(name);
        fs::create_dir_all(file_path.parent().unwrap())?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(file_path)?;
        let mut rw = fd_lock::RwLock::new(&mut file);
        let Ok(_guard) = rw.try_write() else {
            return Ok(());
        };

        let mut applied: Option<String> = None;
        loop {
            let target = fs::read_to_string(Self::target_file(name))
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());

            let Some(target) = target else { return Ok(()) };
            if Some(&target) == applied.as_ref() {
                return Ok(());
            }

            if let Ok(percent) = target.parse::<i64>() {
                match Self::display_for(name, monitors)? {
                    Some(display) => {
                        let ddc = Ddc {
                            name: name.to_string(),
                            display,
                        };
                        let _ = ddc.set(percent);
                    }
                    None => {
                        let _ = notify.send(
                            "Brightness",
                            &format!("{name} does not respond to DDC/CI"),
                            "video-display",
                            None,
                        );
                    }
                }
            }
            applied = Some(target);
        }
    }
}

impl Brightness for Ddc {
    fn get(&self) -> Result<i64> {
        // "VCP 10 C <current> <max>"
        let out = Command::new("ddcutil")
            .args(["--display", &self.display, "getvcp", "10", "--brief"])
            .output()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let fields: Vec<&str> = text.split_whitespace().collect();
        let current: f64 = fields.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let max: f64 = fields.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.0);

        if max > 0.0 {
            Ok((current * 100.0 / max) as i64)
        } else {
            bail!("display {} did not report a usable VCP range", self.display);
        }
    }

    fn set(&self, percent: i64) -> Result<()> {
        let out = Command::new("ddcutil")
            .args(["--display", &self.display, "getvcp", "10", "--brief"])
            .output()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let max: i64 = text
            .split_whitespace()
            .nth(4)
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        let value = percent * max / 100;
        let out = Command::new("ddcutil")
            .args([
                "--display",
                &self.display,
                "setvcp",
                "10",
                &value.to_string(),
            ])
            .output()?;
        if !out.status.success() {
            bail!(
                "ddcutil failed to set brightness on display {}",
                self.display
            );
        }

        // Cache the applied value so a later step (or the worker catching up
        // a queued one) starts from a fresh baseline instead of re-reading
        // the hardware over DDC/CI.
        let file = Self::target_file(&self.name);
        fs::create_dir_all(file.parent().unwrap())?;
        fs::write(file, percent.to_string())?;
        Ok(())
    }

    /// DDC/CI is too slow to apply on every keypress, so a step only updates
    /// the target and hands off to a background worker to catch up — see
    /// `run_worker`.
    fn step(&self, delta: &str, osd: &dyn Osd) -> Result<()> {
        let delta_val: i64 = delta.parse().context("invalid delta")?;

        let current = fs::read_to_string(Self::target_file(&self.name))
            .ok()
            .and_then(|s| s.trim().parse::<i64>().ok())
            .or_else(|| self.get().ok());
        let Some(current) = current else {
            bail!("cannot determine brightness for {}", self.name);
        };
        let target = (current + delta_val).clamp(0, 100);

        let file = Self::target_file(&self.name);
        fs::create_dir_all(file.parent().unwrap())?;
        fs::write(file, target.to_string())?;

        Self::spawn_worker(&self.name)?;

        let _ = osd.show_custom_progress(
            &self.name,
            target as f64 / 100.0,
            "display-brightness-symbolic",
        );
        Ok(())
    }
}
