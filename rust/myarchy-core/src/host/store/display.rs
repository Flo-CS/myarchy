use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use crate::core::compositor::Monitor;
use crate::core::error::UserError;
use crate::core::layout::Layout;
use crate::core::store::LayoutStore;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use crate::host::{file, xdg};

const LOCK_WAIT: Duration = Duration::from_secs(5);

pub struct LayoutFiles;

fn dir() -> PathBuf {
    xdg::state_dir().join("display")
}

fn rules_file() -> PathBuf {
    dir().join("current.lua")
}

/// Connector names renumber across re-plugs, so a profile is addressed by its screens'
/// descriptions.
fn key(monitors: &[Monitor]) -> String {
    let mut descriptions: Vec<&str> = monitors.iter().map(|m| m.description.as_str()).collect();
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

impl LayoutStore for LayoutFiles {
    fn load(&self, monitors: &[Monitor]) -> Result<Option<Layout>> {
        let path = profile_file(monitors);
        let Ok(text) = fs::read_to_string(&path) else {
            return Ok(None);
        };
        serde_json::from_str(&text)
            .map(Some)
            .with_context(|| format!("{} is not valid JSON", path.display()))
            .context(UserError::ProfileUnreadable {
                path: path.display().to_string(),
            })
    }

    fn save(&self, monitors: &[Monitor], layout: &Layout) -> Result<()> {
        file::write_atomic(
            &profile_file(monitors),
            &serde_json::to_string_pretty(layout)?,
        )
    }

    fn render(&self, rules: &str) -> Result<()> {
        file::write_atomic(&rules_file(), rules)
    }

    fn reset(&self, monitors: &[Monitor]) -> Result<()> {
        let _ = fs::remove_file(profile_file(monitors));
        let _ = fs::remove_file(rules_file());
        Ok(())
    }

    /// `flock` is per open file description, not per process, so the lock is taken once at the
    /// public entry point and the unlocked cores are composed underneath it.
    fn locked(&self, f: &mut dyn FnMut() -> Result<()>) -> Result<()> {
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
                Err(err) => {
                    return Err(
                        anyhow::Error::new(err).context(UserError::AnotherDisplayCommandRunning)
                    );
                }
            }
        }
    }
}
