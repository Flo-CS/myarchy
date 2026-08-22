use std::path::Path;

use anyhow::Result;
use myarchy_core::wallpaper::WallpaperCtl;

use crate::proc;

pub struct Hyprpaper;

impl WallpaperCtl for Hyprpaper {
    fn apply(&self, path: &Path) -> Result<()> {
        let target = format!(",{}", path.display());
        if proc::ok("hyprctl", &["hyprpaper", "wallpaper", &target]) {
            return Ok(());
        }
        proc::spawn_detached("hyprpaper", &[])
    }
}
