use std::process::Command;

use anyhow::{bail, Result};

pub(crate) trait Osd {
    fn show_brightness_progress(&self, monitor: &str, delta: &str) -> Result<()>;
    fn show_custom_progress(&self, monitor: &str, progress: f64, icon: &str) -> Result<()>;
}

pub(crate) fn adapter() -> impl Osd {
    SwayOsd
}

struct SwayOsd;

impl Osd for SwayOsd {
    fn show_brightness_progress(&self, monitor: &str, delta: &str) -> Result<()> {
        let status = Command::new("swayosd-client")
            .args(["--monitor", monitor, "--brightness", delta])
            .status()?;
        if !status.success() {
            bail!("swayosd-client failed to step brightness");
        }
        Ok(())
    }

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
