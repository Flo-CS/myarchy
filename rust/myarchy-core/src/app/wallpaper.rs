use std::path::PathBuf;

use crate::core::error::UserError;
use crate::core::store::WallpaperStore;
use crate::core::theme::ThemeCtl;
use crate::core::wallpaper::WallpaperCtl;
use anyhow::{Result, bail};

pub struct Wallpaper<'a> {
    pub backend: &'a dyn WallpaperCtl,
    pub store: &'a dyn WallpaperStore,
    pub theme: &'a dyn ThemeCtl,
}

impl Wallpaper<'_> {
    pub fn dir(&self) -> PathBuf {
        self.store.dir()
    }

    pub fn list(&self) -> Result<Vec<String>> {
        self.store.list()
    }

    pub fn current(&self) -> Option<String> {
        self.store.current()
    }

    pub fn init(&self) -> Result<()> {
        if self.store.has_current() {
            return Ok(());
        }
        let name = self
            .store
            .list()?
            .into_iter()
            .next()
            .ok_or(UserError::NoWallpapers)?;
        self.set(&name)
    }

    pub fn set(&self, name: &str) -> Result<()> {
        self.apply(name)?;
        self.save_preferred()
    }

    pub fn apply_preferred(&self) -> Result<()> {
        let name = match self.store.load_preferred() {
            Some(name) => name,
            None => self
                .theme
                .get_var("wallpaper")?
                .ok_or(UserError::NoPreferredWallpaper)?,
        };
        self.apply(&name)
    }

    pub fn save_preferred(&self) -> Result<()> {
        let name = self.store.current().ok_or(UserError::NoCurrentWallpaper)?;
        self.store.save_preferred(&name)?;
        self.theme.render()
    }

    pub fn reset(&self) -> Result<()> {
        self.store.forget_preferred();
        self.apply_preferred()?;
        self.theme.render()
    }

    fn apply(&self, name: &str) -> Result<()> {
        if !self.store.contains(name) {
            bail!(UserError::UnknownWallpaper {
                name: name.to_string()
            });
        }
        let path = self.store.point_current_at(name)?;

        // hyprpaper.conf preloads this symlink on its own startup, before myarchy-ctl ever runs; the
        // path passed below only tells an already-running hyprpaper to switch live over IPC.
        self.backend.apply(&path)
    }
}
