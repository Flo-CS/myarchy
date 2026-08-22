use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use myarchy_core::compositor::Monitor;
use myarchy_core::layout::Layout;
use sha2::{Digest, Sha256};

use crate::{file, xdg};

const LOCK_WAIT: Duration = Duration::from_secs(5);

fn dir() -> PathBuf {
    xdg::state_dir().join("display")
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

pub fn load(monitors: &[Monitor]) -> Option<Layout> {
    let text = fs::read_to_string(profile_file(monitors)).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save(monitors: &[Monitor], layout: &Layout) -> Result<()> {
    file::write_atomic(
        &profile_file(monitors),
        &serde_json::to_string_pretty(layout)?,
    )
}

pub fn render(rules: &str) -> Result<()> {
    file::write_atomic(&rules_file(), rules)
}

pub fn reset(monitors: &[Monitor]) {
    let _ = fs::remove_file(profile_file(monitors));
    let _ = fs::remove_file(rules_file());
}

/// Hotplug hooks and menu actions both mutate the layout, so every command serialises on one file.
/// `flock` is per open file description, not per process, so the lock is taken once at the public
/// entry point and the unlocked cores are composed underneath it.
pub fn locked<T>(f: impl FnOnce() -> Result<T>) -> Result<T> {
    let path = xdg::runtime_dir().join("display").join("lock");
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
