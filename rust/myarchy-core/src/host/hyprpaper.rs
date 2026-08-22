use std::path::Path;

use crate::core::wallpaper::WallpaperCtl;
use anyhow::Result;

use crate::host::proc;

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
