use std::path::PathBuf;

use anyhow::Result;

use crate::core::compositor::Monitor;
use crate::core::layout::Layout;

pub trait LayoutStore {
    fn load(&self, monitors: &[Monitor]) -> Result<Option<Layout>>;
    fn save(&self, monitors: &[Monitor], layout: &Layout) -> Result<()>;
    fn render(&self, rules: &str) -> Result<()>;
    fn reset(&self, monitors: &[Monitor]) -> Result<()>;

    /// Hotplug hooks and menu actions both mutate the layout, so a whole command runs under one
    /// exclusive lock rather than each read and write taking its own.
    fn locked(&self, f: &mut dyn FnMut() -> Result<()>) -> Result<()>;
}

pub trait WallpaperStore {
    fn dir(&self) -> PathBuf;
    fn list(&self) -> Result<Vec<String>>;
    fn contains(&self, name: &str) -> bool;
    fn current(&self) -> Option<String>;
    fn has_current(&self) -> bool;
    fn point_current_at(&self, name: &str) -> Result<PathBuf>;
    fn load_preferred(&self) -> Option<String>;
    fn save_preferred(&self, name: &str) -> Result<()>;
    fn forget_preferred(&self);
}

pub struct PreferredCursor {
    pub name: String,
    pub size: i64,
}

pub trait CursorStore {
    fn list(&self) -> Result<Vec<String>>;
    fn load_preferred(&self) -> Option<PreferredCursor>;
    fn save_preferred(&self, name: &str, size: i64) -> Result<()>;
    fn forget_preferred(&self);
}
