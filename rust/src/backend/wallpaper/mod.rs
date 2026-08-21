mod hyprpaper;

use std::path::Path;

use anyhow::Result;

pub(crate) trait Wallpaperctl {
    fn apply(&self, path: &Path) -> Result<()>;
}

pub(crate) fn backend() -> impl Wallpaperctl {
    hyprpaper::Hyprpaper
}
