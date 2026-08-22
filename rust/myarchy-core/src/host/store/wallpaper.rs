use std::fs;
use std::path::PathBuf;

use crate::core::error::UserError;
use crate::core::store::WallpaperStore;
use anyhow::{Context, Result};

use crate::host::{file, xdg};

const CURRENT: &str = "current";

pub struct WallpaperFiles;

fn preferred_file() -> PathBuf {
    xdg::state_dir().join("wallpaper").join(CURRENT)
}

fn dir() -> PathBuf {
    xdg::myarchy_dir().join("wallpapers")
}

fn current_link() -> PathBuf {
    dir().join(CURRENT)
}

impl WallpaperStore for WallpaperFiles {
    fn dir(&self) -> PathBuf {
        dir()
    }

    fn list(&self) -> Result<Vec<String>> {
        let dir = self.dir();
        let mut names: Vec<String> = fs::read_dir(&dir)
            .with_context(|| format!("failed to read {}", dir.display()))
            .context(UserError::WallpaperDirUnreadable {
                path: dir.display().to_string(),
            })?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_ok_and(|t| !t.is_dir()))
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name != CURRENT && !name.starts_with('.'))
            .collect();
        names.sort();
        Ok(names)
    }

    fn contains(&self, name: &str) -> bool {
        self.dir().join(name).is_file()
    }

    fn current(&self) -> Option<String> {
        let target = fs::read_link(current_link()).ok()?;
        target.file_name().map(|n| n.to_string_lossy().into_owned())
    }

    fn has_current(&self) -> bool {
        current_link().is_file()
    }

    fn point_current_at(&self, name: &str) -> Result<PathBuf> {
        let path = self.dir().join(name);
        file::symlink_atomic(&path, &current_link())?;
        Ok(path)
    }

    fn load_preferred(&self) -> Option<String> {
        let text = fs::read_to_string(preferred_file()).ok()?;
        let name = text.trim();
        (!name.is_empty()).then(|| name.to_string())
    }

    fn save_preferred(&self, name: &str) -> Result<()> {
        file::write_atomic(&preferred_file(), name)
    }

    fn forget_preferred(&self) {
        let _ = fs::remove_file(preferred_file());
    }
}
