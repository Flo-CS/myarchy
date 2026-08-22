use crate::core::notify::{Icon, OsdCtl};
use anyhow::{Result, bail};

use crate::host::proc;

pub struct SwayOsd;

impl OsdCtl for SwayOsd {
    fn show_custom_progress(&self, monitor: &str, progress: f64, icon: Icon) -> Result<()> {
        let progress = format!("{progress:.2}");
        let args = [
            "--monitor",
            monitor,
            "--custom-progress",
            &progress,
            "--custom-icon",
            icon.as_str(),
        ];
        if !proc::status("swayosd-client", &args)? {
            bail!("swayosd-client failed to show progress");
        }
        Ok(())
    }
}
