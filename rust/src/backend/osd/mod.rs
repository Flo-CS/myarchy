mod swayosd;

use anyhow::Result;

pub(crate) trait Osdctl {
    fn show_custom_progress(&self, monitor: &str, progress: f64, icon: &str) -> Result<()>;
}

pub(crate) fn backend() -> impl Osdctl {
    swayosd::SwayOsd
}
