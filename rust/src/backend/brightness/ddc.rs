use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Result};

use crate::backend::xdg;
use crate::error::AppError;
use crate::models::monitor::Monitor;

use super::Brightnessctl;

pub(super) struct Ddc {
    name: String,
    display: String,
}

impl Ddc {
    pub(super) fn new(name: String, display: String) -> Self {
        Self { name, display }
    }

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

    /// Laptop panels show up as "Invalid display" blocks that still carry a connector line, so a
    /// mapping is only recorded while a numbered display is open.
    fn parse_detect(text: &str) -> Vec<(String, String)> {
        let mut found = Vec::new();
        let mut display: Option<&str> = None;
        for line in text.lines() {
            if line.starts_with("Display ") {
                display = line.split_whitespace().nth(1);
            } else if line.starts_with("Invalid display") {
                display = None;
            } else if line.contains("DRM connector:") {
                if let (Some(d), Some(raw)) = (display, line.split_whitespace().nth(2)) {
                    found.push((Self::strip_card_prefix(raw), d.to_string()));
                }
            }
        }
        found
    }

    /// "VCP 10 C <current> <max>"
    fn vcp_max(text: &str) -> Option<i64> {
        text.split_whitespace().nth(4)?.parse().ok()
    }

    fn vcp_percent(text: &str) -> Option<i64> {
        let fields: Vec<&str> = text.split_whitespace().collect();
        let current: f64 = fields.get(3).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let max: f64 = fields.get(4).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        (max > 0.0).then(|| (current * 100.0 / max) as i64)
    }

    fn cache_path() -> PathBuf {
        xdg::runtime_dir().join("screen").join("ddc-map")
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
                for (connector, display) in
                    Self::parse_detect(&String::from_utf8_lossy(&detect.stdout))
                {
                    out.push_str(&format!("{connector}\t{display}\n"));
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

    pub(super) fn display_for(name: &str, monitors: &[Monitor]) -> Result<Option<String>> {
        Ok(Self::map(monitors)?.get(name).cloned())
    }

    fn target_file(name: &str) -> PathBuf {
        xdg::runtime_dir()
            .join("screen")
            .join(format!("brightness-target-{name}"))
    }

    fn lock_file(name: &str) -> PathBuf {
        xdg::runtime_dir()
            .join("screen")
            .join(format!("brightness-lock-{name}"))
    }

    fn write_hardware(&self, percent: i64) -> Result<()> {
        let out = Command::new("ddcutil")
            .args(["--display", &self.display, "getvcp", "10", "--brief"])
            .output()?;
        let max = Self::vcp_max(&String::from_utf8_lossy(&out.stdout)).unwrap_or(100);
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
            bail!(AppError::DdcNotResponding {
                name: self.name.clone()
            });
        }
        Ok(())
    }
}

impl Brightnessctl for Ddc {
    fn get(&self) -> Result<i64> {
        let out = Command::new("ddcutil")
            .args(["--display", &self.display, "getvcp", "10", "--brief"])
            .output()?;
        match Self::vcp_percent(&String::from_utf8_lossy(&out.stdout)) {
            Some(percent) => Ok(percent),
            None => bail!("display {} did not report a usable VCP range", self.display),
        }
    }

    fn set(&self, percent: i64) -> Result<()> {
        self.write_hardware(percent)?;

        let file = Self::target_file(&self.name);
        fs::create_dir_all(file.parent().unwrap())?;
        fs::write(file, percent.to_string())?;
        Ok(())
    }

    fn step(&self, delta: i64) -> Result<i64> {
        let current = fs::read_to_string(Self::target_file(&self.name))
            .ok()
            .and_then(|s| s.trim().parse::<i64>().ok())
            .or_else(|| self.get().ok());
        let Some(current) = current else {
            bail!("cannot determine brightness for {}", self.name);
        };
        let target = (current + delta).clamp(0, 100);

        let file = Self::target_file(&self.name);
        fs::create_dir_all(file.parent().unwrap())?;
        fs::write(file, target.to_string())?;

        Ok(target)
    }

    /// A burst of steps only ever leaves one invocation holding this lock; the rest just update
    /// the target file and return, so the holder's loop is what converges to the last one queued.
    fn settle(&self) -> Result<()> {
        let file_path = Self::lock_file(&self.name);
        fs::create_dir_all(file_path.parent().unwrap())?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(file_path)?;
        let mut rw = fd_lock::RwLock::new(&mut file);
        let Ok(_guard) = rw.try_write() else {
            return Ok(());
        };

        let mut applied: Option<i64> = None;
        loop {
            let target = fs::read_to_string(Self::target_file(&self.name))
                .ok()
                .and_then(|s| s.trim().parse::<i64>().ok());
            let Some(target) = target else {
                return Ok(());
            };
            if Some(target) == applied {
                return Ok(());
            }
            self.write_hardware(target)?;
            applied = Some(target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verbatim `ddcutil detect --brief` output.
    #[test]
    fn detect_maps_the_working_displays_and_skips_the_laptop_panel() {
        let text = "\
Display 1
   I2C bus:          /dev/i2c-5
   DRM connector:    card1-DP-3
   drm_connector_id: 112
   Monitor:          LG Electronics:LG HDR WQHD:303NTZN51357
Display 2
   I2C bus:          /dev/i2c-6
   DRM connector:    card1-HDMI-A-1
   drm_connector_id: 113
   Monitor:          Dell Inc.:DELL U2412M:PMSXXXX
Invalid display
   I2C bus:          /dev/i2c-4
   DRM connector:    card1-eDP-1
   drm_connector_id: 111
   Monitor:          BOE::
";

        assert_eq!(
            Ddc::parse_detect(text),
            vec![
                ("DP-3".to_string(), "1".to_string()),
                ("HDMI-A-1".to_string(), "2".to_string()),
            ]
        );
    }

    #[test]
    fn a_vcp_reply_becomes_a_percentage_of_the_range_the_monitor_reports() {
        assert_eq!(Ddc::vcp_percent("VCP 10 C 50 100"), Some(50));
        assert_eq!(Ddc::vcp_percent("VCP 10 C 128 255"), Some(50));
        assert_eq!(Ddc::vcp_max("VCP 10 C 128 255"), Some(255));

        assert_eq!(Ddc::vcp_percent("VCP 10 C 50 0"), None, "a zero range");
        assert_eq!(Ddc::vcp_percent("VCP 10 ERR"), None, "a truncated reply");
        assert_eq!(Ddc::vcp_percent(""), None);
    }
}
