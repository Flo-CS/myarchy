use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

use crate::error::AppError;
use crate::models::monitor::Monitor;
use crate::ports::compositorctl::Compositor;
use crate::ports::notifierctl::Notifier;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    desc: String,
    enabled: bool,
    mode: String,
    position: String,
    scale: String,
}

impl Row {
    fn to_tsv(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}",
            self.desc,
            if self.enabled { 1 } else { 0 },
            self.mode,
            self.position,
            self.scale,
        )
    }

    fn from_tsv(line: &str) -> Option<Row> {
        let mut f = line.split('\t');
        Some(Row {
            desc: f.next()?.to_string(),
            enabled: f.next()? == "1",
            mode: f.next()?.to_string(),
            position: f.next()?.to_string(),
            scale: f.next()?.to_string(),
        })
    }
}

fn rows_to_text(rows: &[Row]) -> String {
    let mut out: String = rows
        .iter()
        .map(|r| r.to_tsv())
        .collect::<Vec<_>>()
        .join("\n");
    out.push('\n');
    out
}

fn rows_from_text(text: &str) -> Vec<Row> {
    text.lines().filter_map(Row::from_tsv).collect()
}

fn mon_names(snap: &[Monitor]) -> Vec<&str> {
    snap.iter().map(|m| m.name.as_str()).collect()
}

fn mon_enabled(snap: &[Monitor]) -> Vec<&str> {
    snap.iter()
        .filter(|m| !m.disabled)
        .map(|m| m.name.as_str())
        .collect()
}

fn name_of_desc<'a>(snap: &'a [Monitor], desc: &str) -> Option<&'a str> {
    snap.iter()
        .find(|m| m.description() == desc)
        .map(|m| m.name.as_str())
}

fn desc_of_name<'a>(snap: &'a [Monitor], name: &str) -> Option<&'a str> {
    snap.iter()
        .find(|m| m.name == name)
        .map(|m| m.description())
}

fn is_disabled(snap: &[Monitor], name: &str) -> bool {
    snap.iter()
        .find(|m| m.name == name)
        .map(|m| m.disabled)
        .unwrap_or(false)
}

fn mode_unfloored(m: &Monitor) -> String {
    format!("{}x{}@{}", m.width, m.height, m.refresh_rate)
}

fn layout_rows(snap: &[Monitor]) -> Vec<Row> {
    snap.iter()
        .map(|m| Row {
            desc: m.description().to_string(),
            enabled: !m.disabled,
            mode: mode_unfloored(m),
            position: format!("{}x{}", m.x, m.y),
            scale: format!("{}", m.scale),
        })
        .collect()
}

fn list_monitors(snap: &[Monitor]) -> String {
    snap.iter()
        .map(|m| {
            format!(
                "{}\t{}\t{}\t{}",
                m.name,
                m.description(),
                m.resolution(),
                if m.disabled { "disabled" } else { "enabled" }
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn list_modes_of(snap: &[Monitor], name: &str) -> String {
    snap.iter()
        .find(|m| m.name == name)
        .map(|m| {
            m.resolutions
                .iter()
                .map(|mode| mode.strip_suffix("Hz").unwrap_or(mode))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

fn lua_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c == '\\' || c == '"' {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Matches bash's `^-?[0-9]+(\.[0-9]+)?$`.
fn is_numeric(v: &str) -> bool {
    let v = v.strip_prefix('-').unwrap_or(v);
    let mut parts = v.splitn(2, '.');
    let int_part = parts.next().unwrap_or("");
    if int_part.is_empty() || !int_part.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    match parts.next() {
        Some(frac) => !frac.is_empty() && frac.chars().all(|c| c.is_ascii_digit()),
        None => true,
    }
}

fn lua_scalar(v: &str) -> String {
    if is_numeric(v) {
        v.to_string()
    } else {
        format!("\"{v}\"")
    }
}

fn emit_rules(rows: &[Row]) -> String {
    let mut out = String::new();
    for row in rows {
        if !row.enabled {
            out.push_str(&format!(
                "hl.monitor({{ output = \"desc:{}\", disabled = true }})\n",
                lua_string(&row.desc)
            ));
        } else {
            out.push_str(&format!(
                "hl.monitor({{ output = \"desc:{}\", mode = \"{}\", position = \"{}\", scale = {}, disabled = false }})\n",
                lua_string(&row.desc),
                row.mode,
                row.position,
                lua_scalar(&row.scale)
            ));
        }
    }
    out
}

fn state_dir() -> PathBuf {
    crate::ports::xdg::state_dir().join("display")
}

fn lock_file() -> PathBuf {
    crate::ports::xdg::runtime_dir()
        .join("display")
        .join("lock")
}

fn profile_key(snap: &[Monitor]) -> String {
    let mut descs: Vec<&str> = snap.iter().map(|m| m.description()).collect();
    descs.sort_unstable();
    let mut joined = descs.join("\n");
    joined.push('\n');

    let mut hasher = Sha256::new();
    hasher.update(joined.as_bytes());
    let hex = format!("{:x}", hasher.finalize());
    hex[..12].to_string()
}

fn profile_file(snap: &[Monitor]) -> PathBuf {
    state_dir().join(format!("{}.lua", profile_key(snap)))
}

fn anchor_file(snap: &[Monitor]) -> PathBuf {
    state_dir().join(format!("{}.primary", profile_key(snap)))
}

fn geometry_file(snap: &[Monitor]) -> PathBuf {
    state_dir().join(format!("{}.rows", profile_key(snap)))
}

fn write_atomic(file: &Path, contents: &str) -> Result<()> {
    fs::create_dir_all(state_dir())?;
    let tmp = PathBuf::from(format!("{}.tmp{}", file.display(), std::process::id()));
    fs::write(&tmp, contents).with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, file).with_context(|| format!("failed to replace {}", file.display()))?;
    Ok(())
}

fn write_profile(file: &Path, rows: &[Row]) -> Result<()> {
    write_atomic(file, &emit_rules(rows))
}

fn link_current(file: &Path) -> Result<()> {
    let link = state_dir().join("current.lua");
    let target = file.file_name().context("profile file has no file name")?;
    let _ = fs::remove_file(&link);
    std::os::unix::fs::symlink(target, &link)
        .with_context(|| format!("failed to symlink {}", link.display()))
}

fn save_profile(snap: &[Monitor]) -> Result<()> {
    let file = profile_file(snap);
    write_profile(&file, &current_rows(snap)?)?;
    link_current(&file)
}

fn anchor_desc(snap: &[Monitor]) -> Option<String> {
    let file = anchor_file(snap);
    fs::read_to_string(file)
        .ok()
        .map(|s| s.trim_end_matches('\n').to_string())
        .filter(|s| !s.is_empty())
}

fn anchor_name(snap: &[Monitor]) -> Option<String> {
    let desc = anchor_desc(snap)?;
    name_of_desc(snap, &desc).map(|s| s.to_string())
}

fn set_anchor(snap: &[Monitor], name: &str) -> Result<()> {
    let desc = desc_of_name(snap, name).ok_or_else(|| AppError::UnknownMonitor {
        name: name.to_string(),
    })?;
    fs::create_dir_all(state_dir())?;
    fs::write(anchor_file(snap), format!("{desc}\n"))?;
    Ok(())
}

fn anchor_or_focused(snap: &[Monitor]) -> Option<String> {
    if let Some(desc) = anchor_desc(snap) {
        if let Some(name) = name_of_desc(snap, &desc) {
            return Some(name.to_string());
        }
    }
    if let Some(m) = snap.iter().find(|m| m.focused) {
        return Some(m.name.clone());
    }
    mon_enabled(snap).first().map(|s| s.to_string())
}

fn cached_row(snap: &[Monitor], desc: &str) -> Option<Row> {
    let file = geometry_file(snap);
    let text = fs::read_to_string(file).ok()?;
    rows_from_text(&text).into_iter().find(|r| r.desc == desc)
}

fn cached_row_exists(snap: &[Monitor], desc: &str) -> bool {
    cached_row(snap, desc).is_some()
}

fn current_rows(snap: &[Monitor]) -> Result<Vec<Row>> {
    let file = geometry_file(snap);
    let live = layout_rows(snap);
    match fs::read_to_string(&file) {
        Ok(text) => {
            let fallback = rows_from_text(&text);
            Ok(merge_disabled_geometry(&fallback, &live))
        }
        Err(_) => Ok(live),
    }
}

/// hyprctl reports 0x0 for a disabled monitor's mode/position/scale, so a
/// disabled row falls back to the last known-good geometry instead of that.
fn merge_disabled_geometry(fallback: &[Row], current: &[Row]) -> Vec<Row> {
    current
        .iter()
        .map(|row| {
            if !row.enabled {
                if let Some(f) = fallback.iter().find(|f| f.desc == row.desc) {
                    return Row {
                        desc: row.desc.clone(),
                        enabled: false,
                        mode: f.mode.clone(),
                        position: f.position.clone(),
                        scale: f.scale.clone(),
                    };
                }
            }
            row.clone()
        })
        .collect()
}

enum ApplyProfileOutcome {
    NoProfile,
    Applied,
}

/// Returning `Applied` on a match (nothing to reload) is what stops our own
/// writes looping back through the hotplug hooks.
fn apply_profile(compositor: &dyn Compositor, snap: &[Monitor]) -> Result<ApplyProfileOutcome> {
    let file = profile_file(snap);
    if !file.exists() {
        return Ok(ApplyProfileOutcome::NoProfile);
    }
    link_current(&file)?;

    let live = emit_rules(&layout_rows(snap));
    let saved = fs::read_to_string(&file)?;
    if live == saved {
        return Ok(ApplyProfileOutcome::Applied);
    }

    compositor.reload()?;
    Ok(ApplyProfileOutcome::Applied)
}

/// Rules land asynchronously; poll until two consecutive reads agree, or give up.
fn settle(compositor: &dyn Compositor) -> Result<(Vec<Monitor>, bool)> {
    let mut cur = compositor.described_monitors(true)?;
    let mut prev = layout_rows(&cur);
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(100));
        cur = compositor.described_monitors(true)?;
        let sig = layout_rows(&cur);
        if sig == prev {
            return Ok((cur, true));
        }
        prev = sig;
    }
    Ok((cur, false))
}

/// Writing the rules is what applies them, so the layout is re-read afterwards
/// and stored as the concrete geometry the compositor settled on. Disabled
/// rows keep whatever geometry they came in with — settling can't recover it.
fn apply_layout(compositor: &dyn Compositor, rows: &[Row]) -> Result<Vec<Monitor>> {
    let snap = compositor.described_monitors(true)?;
    let file = profile_file(&snap);

    write_profile(&file, rows)?;
    link_current(&file)?;
    compositor.reload()?;

    let (settled, ok) = settle(compositor)?;
    if !ok {
        eprintln!("layout still moving, saving the last reading");
    }

    let merged = merge_disabled_geometry(rows, &layout_rows(&settled));
    write_profile(&file, &merged)?;
    fs::create_dir_all(state_dir())?;
    fs::write(geometry_file(&snap), rows_to_text(&merged))?;

    Ok(settled)
}

fn direction_position(direction: &str) -> Result<&'static str> {
    Ok(match direction {
        "left" => "auto-left",
        "right" => "auto-right",
        "above" => "auto-up",
        "below" => "auto-down",
        other => bail!(AppError::UnknownDirection {
            direction: other.to_string()
        }),
    })
}

pub(crate) fn list(compositor: &dyn Compositor) -> Result<String> {
    let snap = compositor.described_monitors(true)?;
    Ok(list_monitors(&snap))
}

pub(crate) fn list_modes(compositor: &dyn Compositor, name: &str) -> Result<String> {
    let snap = compositor.described_monitors(true)?;
    Ok(list_modes_of(&snap, name))
}

pub(crate) fn extend(compositor: &dyn Compositor, direction: &str) -> Result<()> {
    let position = direction_position(direction)?;
    let snap = compositor.described_monitors(true)?;
    let anchor = anchor_or_focused(&snap);
    let anchor_desc = anchor
        .as_deref()
        .and_then(|n| desc_of_name(&snap, n))
        .unwrap_or("");

    let rows: Vec<Row> = snap
        .iter()
        .map(|m| {
            let desc = m.description().to_string();
            let is_anchor = desc == anchor_desc;
            let scale = if m.scale > 0.0 {
                format!("{}", m.scale)
            } else {
                "auto".to_string()
            };
            Row {
                desc,
                enabled: true,
                mode: "preferred".to_string(),
                position: if is_anchor {
                    "0x0".to_string()
                } else {
                    position.to_string()
                },
                scale,
            }
        })
        .collect();

    apply_layout(compositor, &rows)?;
    Ok(())
}

struct Geo {
    x: i64,
    y: i64,
    w: i64,
    h: i64,
}

/// Laying the whole axis out sequentially is what makes overlap impossible.
pub(crate) fn place(
    compositor: &dyn Compositor,
    name: &str,
    side: &str,
    refname: &str,
) -> Result<()> {
    let horizontal = match side {
        "left-of" | "right-of" => true,
        "above" | "below" => false,
        other => bail!(AppError::UnknownSide {
            side: other.to_string()
        }),
    };

    if name == refname {
        bail!(AppError::CannotPlaceSelfRelative);
    }

    let snap = compositor.described_monitors(true)?;

    let mut enabled: Vec<&Monitor> = snap.iter().filter(|m| !m.disabled).collect();
    if horizontal {
        enabled.sort_by_key(|m| m.x);
    } else {
        enabled.sort_by_key(|m| m.y);
    }

    let mut geo: HashMap<&str, Geo> = HashMap::new();
    let mut order: Vec<&str> = Vec::new();
    for m in &enabled {
        geo.insert(
            m.name.as_str(),
            Geo {
                x: m.x,
                y: m.y,
                w: (m.width as f64 / m.scale).round() as i64,
                h: (m.height as f64 / m.scale).round() as i64,
            },
        );
        if m.name != name {
            order.push(m.name.as_str());
        }
    }

    if !geo.contains_key(name) || !geo.contains_key(refname) {
        bail!(AppError::CannotPlace {
            name: name.to_string(),
            side: side.to_string(),
            refname: refname.to_string()
        });
    }

    let mut moved: Vec<&str> = Vec::new();
    for n in &order {
        if *n == refname && matches!(side, "left-of" | "above") {
            moved.push(name);
        }
        moved.push(n);
        if *n == refname && matches!(side, "right-of" | "below") {
            moved.push(name);
        }
    }

    let mut rows = current_rows(&snap)?;
    let mut offset: i64 = 0;
    for n in &moved {
        let g = &geo[n];
        let desc = desc_of_name(&snap, n).unwrap_or("").to_string();
        let position = if horizontal {
            let p = format!("{offset}x{}", g.y);
            offset += g.w;
            p
        } else {
            let p = format!("{}x{offset}", g.x);
            offset += g.h;
            p
        };
        if let Some(row) = rows.iter_mut().find(|r| r.desc == desc) {
            row.position = position;
        }
    }

    apply_layout(compositor, &rows)?;
    Ok(())
}

pub(crate) fn only(compositor: &dyn Compositor, keep: &str) -> Result<()> {
    let snap = compositor.described_monitors(true)?;
    let keep_desc = desc_of_name(&snap, keep)
        .ok_or_else(|| AppError::UnknownMonitor {
            name: keep.to_string(),
        })?
        .to_string();

    let mut rows = current_rows(&snap)?;
    if let Some(row) = rows.iter_mut().find(|r| r.desc == keep_desc) {
        row.enabled = true;
        row.mode = "preferred".to_string();
        row.position = "0x0".to_string();
    }
    for name in mon_names(&snap) {
        if name == keep {
            continue;
        }
        if let Some(desc) = desc_of_name(&snap, name) {
            if let Some(row) = rows.iter_mut().find(|r| r.desc == desc) {
                row.enabled = false;
            }
        }
    }

    let settled = apply_layout(compositor, &rows)?;

    for name in mon_names(&snap) {
        if name == keep {
            continue;
        }
        if !is_disabled(&settled, name) {
            bail!(AppError::DidNotSwitchOff {
                name: name.to_string()
            });
        }
    }

    set_anchor(&settled, keep)
}

pub(crate) fn enable_monitor(compositor: &dyn Compositor, name: &str) -> Result<()> {
    let snap = compositor.described_monitors(true)?;
    let desc = desc_of_name(&snap, name)
        .ok_or_else(|| AppError::UnknownMonitor {
            name: name.to_string(),
        })?
        .to_string();

    let mut rows = current_rows(&snap)?;
    if !cached_row_exists(&snap, &desc) {
        if let Some(row) = rows.iter_mut().find(|r| r.desc == desc) {
            row.mode = "preferred".to_string();
            row.position = "auto".to_string();
        }
    }
    if let Some(row) = rows.iter_mut().find(|r| r.desc == desc) {
        if row.scale == "0" {
            row.scale = "auto".to_string();
        }
        row.enabled = true;
    }

    let settled = apply_layout(compositor, &rows)?;
    if is_disabled(&settled, name) {
        bail!(AppError::DidNotSwitchOn {
            name: name.to_string()
        });
    }
    Ok(())
}

pub(crate) fn disable_monitor(compositor: &dyn Compositor, name: &str) -> Result<()> {
    let snap = compositor.described_monitors(true)?;
    let desc = desc_of_name(&snap, name)
        .ok_or_else(|| AppError::UnknownMonitor {
            name: name.to_string(),
        })?
        .to_string();

    let enabled = mon_enabled(&snap);
    if enabled.len() <= 1 {
        bail!(AppError::CannotDisableLastScreen);
    }

    let mut rows = current_rows(&snap)?;
    if let Some(row) = rows.iter_mut().find(|r| r.desc == desc) {
        row.enabled = false;
    }

    let settled = apply_layout(compositor, &rows)?;
    if !is_disabled(&settled, name) {
        bail!(AppError::DidNotSwitchOff {
            name: name.to_string()
        });
    }
    Ok(())
}

pub(crate) fn toggle_monitor(compositor: &dyn Compositor, name: &str) -> Result<()> {
    let snap = compositor.described_monitors(true)?;
    if is_disabled(&snap, name) {
        enable_monitor(compositor, name)
    } else {
        disable_monitor(compositor, name)
    }
}

pub(crate) fn set_mode(compositor: &dyn Compositor, name: &str, mode: &str) -> Result<()> {
    let snap = compositor.described_monitors(true)?;
    let desc = desc_of_name(&snap, name).unwrap_or("").to_string();
    let mut rows = current_rows(&snap)?;
    if let Some(row) = rows.iter_mut().find(|r| r.desc == desc) {
        row.mode = mode.to_string();
    }
    apply_layout(compositor, &rows)?;
    Ok(())
}

pub(crate) fn set_scale(compositor: &dyn Compositor, name: &str, scale: &str) -> Result<()> {
    let snap = compositor.described_monitors(true)?;
    let desc = desc_of_name(&snap, name).unwrap_or("").to_string();
    let mut rows = current_rows(&snap)?;
    if let Some(row) = rows.iter_mut().find(|r| r.desc == desc) {
        row.scale = scale.to_string();
    }
    apply_layout(compositor, &rows)?;
    Ok(())
}

pub(crate) fn set_primary(compositor: &dyn Compositor, name: &str) -> Result<()> {
    let snap = compositor.described_monitors(true)?;
    set_anchor(&snap, name)
}

pub(crate) fn anchor(compositor: &dyn Compositor) -> Result<Option<String>> {
    let snap = compositor.described_monitors(true)?;
    Ok(anchor_name(&snap))
}

pub(crate) fn save(compositor: &dyn Compositor) -> Result<()> {
    let snap = compositor.described_monitors(true)?;
    save_profile(&snap)
}

pub(crate) fn apply(compositor: &dyn Compositor) -> Result<()> {
    let (snap, _) = settle(compositor)?;
    apply_profile(compositor, &snap)?;
    Ok(())
}

pub(crate) fn reset(compositor: &dyn Compositor) -> Result<()> {
    let snap = compositor.described_monitors(true)?;
    let _ = fs::remove_file(profile_file(&snap));
    let _ = fs::remove_file(anchor_file(&snap));
    let _ = fs::remove_file(geometry_file(&snap));
    let _ = fs::remove_file(state_dir().join("current.lua"));
    compositor.reload()
}

/// Entry point for the monitor.added/removed and hyprland.start hooks.
pub(crate) fn auto(compositor: &dyn Compositor, notify: &dyn Notifier) -> Result<()> {
    let lock_path = lock_file();
    fs::create_dir_all(lock_path.parent().unwrap())?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&lock_path)?;
    let mut rw = fd_lock::RwLock::new(&mut file);

    let start = Instant::now();
    let _guard = loop {
        match rw.try_write() {
            Ok(guard) => break guard,
            Err(_) if start.elapsed() < Duration::from_secs(5) => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(_) => return Ok(()),
        }
    };

    let (snap, _) = settle(compositor)?;

    if let ApplyProfileOutcome::Applied = apply_profile(compositor, &snap)? {
        return Ok(());
    }

    if snap.len() <= 1 {
        save_profile(&snap)?;
        return Ok(());
    }

    extend(compositor, "right")?;

    let fresh = compositor.described_monitors(true)?;
    if let Some(name) = mon_enabled(&fresh).last() {
        let _ = notify.send(
            "Screen connected",
            &format!("{name} extended to the right — MOD+P for display options"),
            "video-display",
            Some(8000),
        );
    }
    Ok(())
}
