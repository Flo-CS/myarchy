use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};

use super::Wallpaperctl;

pub(super) struct Hyprpaper;

impl Wallpaperctl for Hyprpaper {
    fn apply(&self, path: &Path) -> Result<()> {
        let live = Command::new("hyprctl")
            .args(["hyprpaper", "wallpaper", &format!(",{}", path.display())])
            .output()
            .context("failed to run hyprctl hyprpaper")?
            .status
            .success();

        if !live {
            Command::new("hyprpaper")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .context("failed to start hyprpaper")?;
        }
        Ok(())
    }
}
