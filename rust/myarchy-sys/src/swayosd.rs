use anyhow::{bail, Result};
use myarchy_core::notify::OsdCtl;

use crate::proc;

pub struct SwayOsd;

impl OsdCtl for SwayOsd {
    fn show_custom_progress(&self, monitor: &str, progress: f64, icon: &str) -> Result<()> {
        let progress = format!("{progress:.2}");
        let args = [
            "--monitor",
            monitor,
            "--custom-progress",
            &progress,
            "--custom-icon",
            icon,
        ];
        if !proc::status("swayosd-client", &args)? {
            bail!("swayosd-client failed to show progress");
        }
        Ok(())
    }
}
