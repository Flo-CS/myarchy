use crate::core::brightness::BrightnessCtl;
use crate::core::percent::Percent;
use anyhow::{Context, Result};

use crate::host::proc;

pub struct Backlight;

impl BrightnessCtl for Backlight {
    fn get(&self, _dirty: bool) -> Result<Percent> {
        let out = proc::output(
            "brightnessctl",
            &["--class=backlight", "--machine-readable"],
        )?;
        let text = String::from_utf8_lossy(&out.stdout);
        // "intel_backlight,backlight,60000,50%,120000"
        let line = text.lines().next().unwrap_or("");
        let pct = line.split(',').nth(3).unwrap_or("");
        pct.parse().context("could not read backlight brightness")
    }

    fn set(&self, percent: Percent, _dirty: bool) -> Result<()> {
        let target = format!("{percent}%");
        proc::run("brightnessctl", &["--class=backlight", "set", &target])
            .context("failed to set backlight brightness")?;
        Ok(())
    }

    fn step(&self, delta: i64) -> Result<Percent> {
        let target = self.get(false)?.offset(delta);
        self.set(target, false)?;
        Ok(target)
    }

    fn settle(&self) -> Result<()> {
        Ok(())
    }
}
