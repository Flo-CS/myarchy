use std::path::Path;

use anyhow::Result;

pub trait WallpaperCtl {
    fn apply(&self, path: &Path) -> Result<()>;
}
