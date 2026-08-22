use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use crate::display::layout::Layout;
use crate::display::monitor::Monitor;

const LOCK_WAIT: Duration = Duration::from_secs(5);

fn dir() -> PathBuf {
    crate::backend::xdg::state_dir().join("display")
}

fn rules_file() -> PathBuf {
    dir().join("current.lua")
}

/// Connector names renumber across re-plugs, so a profile is addressed by its screens'
/// descriptions.
fn key(monitors: &[Monitor]) -> String {
    let mut descriptions: Vec<&str> = monitors.iter().map(|m| m.description()).collect();
    descriptions.sort_unstable();
    let mut joined = descriptions.join("\n");
    joined.push('\n');

    let mut hasher = Sha256::new();
    hasher.update(joined.as_bytes());
    hasher.finalize()[..6]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn profile_file(monitors: &[Monitor]) -> PathBuf {
    dir().join(format!("{}.json", key(monitors)))
}

fn write_atomic(file: &Path, contents: &str) -> Result<()> {
    fs::create_dir_all(dir())?;
    let tmp = PathBuf::from(format!("{}.tmp{}", file.display(), std::process::id()));
    fs::write(&tmp, contents).with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, file).with_context(|| format!("failed to replace {}", file.display()))
}

pub(super) fn load(monitors: &[Monitor]) -> Option<Layout> {
    let text = fs::read_to_string(profile_file(monitors)).ok()?;
    serde_json::from_str(&text).ok()
}

pub(super) fn save(monitors: &[Monitor], layout: &Layout) -> Result<()> {
    write_atomic(
        &profile_file(monitors),
        &serde_json::to_string_pretty(layout)?,
    )
}

pub(super) fn render(rules: &str) -> Result<()> {
    write_atomic(&rules_file(), rules)
}

pub(super) fn reset(monitors: &[Monitor]) {
    let _ = fs::remove_file(profile_file(monitors));
    let _ = fs::remove_file(rules_file());
}

/// Hotplug hooks and menu actions both mutate the layout, so every command serialises on one file.
/// `flock` is per open file description, not per process, so the lock is taken once at the public
/// entry point and the unlocked cores are composed underneath it.
pub(super) fn locked<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    let path = crate::backend::xdg::runtime_dir()
        .join("display")
        .join("lock");
    fs::create_dir_all(path.parent().unwrap())?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)?;
    let mut lock = fd_lock::RwLock::new(&mut file);

    let start = Instant::now();
    loop {
        match lock.try_write() {
            Ok(_guard) => return f(),
            Err(_) if start.elapsed() < LOCK_WAIT => thread::sleep(Duration::from_millis(100)),
            Err(err) => return Err(err).context("another display command is still running"),
        }
    }
}
