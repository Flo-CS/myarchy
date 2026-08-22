use std::process::Command;

use anyhow::{bail, Result};

use crate::display::brightness::OsdCtl;

pub(super) struct SwayOsd;

impl OsdCtl for SwayOsd {
    fn show_custom_progress(&self, monitor: &str, progress: f64, icon: &str) -> Result<()> {
        let status = Command::new("swayosd-client")
            .args([
                "--monitor",
                monitor,
                "--custom-progress",
                &format!("{progress:.2}"),
                "--custom-icon",
                icon,
            ])
            .status()?;
        if !status.success() {
            bail!("swayosd-client failed to show progress");
        }
        Ok(())
    }
}
