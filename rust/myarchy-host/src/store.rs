pub mod display {
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

    /// Hotplug hooks and menu actions both mutate the layout, so every command serialises on one
    /// file. `flock` is per open file description, not per process, so the lock is taken once at
    /// the public entry point and the unlocked cores are composed underneath it.
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
}

pub mod wallpaper {
    use std::fs;
    use std::path::PathBuf;

    use anyhow::{Context, Result};

    use crate::{file, xdg};

    const CURRENT: &str = "current";

    pub fn dir() -> PathBuf {
        xdg::myarchy_dir().join("wallpapers")
    }

    pub fn current_link() -> PathBuf {
        dir().join(CURRENT)
    }

    fn preferred_file() -> PathBuf {
        xdg::state_dir().join("wallpaper").join(CURRENT)
    }

    pub fn list() -> Result<Vec<String>> {
        let mut names: Vec<String> = fs::read_dir(dir())
            .context("failed to read wallpapers directory")?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_ok_and(|t| !t.is_dir()))
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name != CURRENT && !name.starts_with('.'))
            .collect();
        names.sort();
        Ok(names)
    }

    pub fn current() -> Option<String> {
        let target = fs::read_link(current_link()).ok()?;
        target.file_name().map(|n| n.to_string_lossy().into_owned())
    }

    pub fn point_current_at(name: &str) -> Result<PathBuf> {
        let path = dir().join(name);
        file::symlink_atomic(&path, &current_link())?;
        Ok(path)
    }

    pub fn load_preferred() -> Option<String> {
        let text = fs::read_to_string(preferred_file()).ok()?;
        let name = text.trim();
        (!name.is_empty()).then(|| name.to_string())
    }

    pub fn save_preferred(name: &str) -> Result<()> {
        file::write_atomic(&preferred_file(), name)
    }

    pub fn forget_preferred() {
        let _ = fs::remove_file(preferred_file());
    }
}

pub mod cursor {
    use std::fs;
    use std::path::PathBuf;

    use anyhow::Result;
    use serde::{Deserialize, Serialize};

    use crate::{file, xdg};

    #[derive(Serialize, Deserialize)]
    pub struct Preferred {
        pub name: String,
        pub size: i64,
    }

    fn preferred_file() -> PathBuf {
        xdg::state_dir().join("cursor/current.json")
    }

    fn theme_roots() -> Vec<PathBuf> {
        let home = xdg::home();
        vec![
            PathBuf::from("/usr/share/icons"),
            home.join(".local/share/icons"),
            home.join(".icons"),
        ]
    }

    pub fn list() -> Result<Vec<String>> {
        let mut names = Vec::new();
        for root in theme_roots() {
            let Ok(entries) = fs::read_dir(&root) else {
                continue;
            };
            for entry in entries.filter_map(|e| e.ok()) {
                let is_cursor_theme = entry.file_type().is_ok_and(|t| t.is_dir())
                    && fs::symlink_metadata(entry.path().join("cursors")).is_ok_and(|m| m.is_dir());
                if is_cursor_theme {
                    names.push(entry.file_name().to_string_lossy().into_owned());
                }
            }
        }
        names.sort();
        names.dedup();
        Ok(names)
    }

    pub fn load_preferred() -> Option<Preferred> {
        let text = fs::read_to_string(preferred_file()).ok()?;
        serde_json::from_str(&text).ok()
    }

    pub fn save_preferred(name: &str, size: i64) -> Result<()> {
        let state = Preferred {
            name: name.to_string(),
            size,
        };
        file::write_atomic(&preferred_file(), &serde_json::to_string_pretty(&state)?)
    }

    pub fn forget_preferred() {
        let _ = fs::remove_file(preferred_file());
    }
}
