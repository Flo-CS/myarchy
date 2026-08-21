use std::process::Command;

use anyhow::{bail, Context, Result};

use super::BrightnessCtl;

pub(super) struct Backlight;

impl BrightnessCtl for Backlight {
    fn get(&self) -> Result<i64> {
        let out = Command::new("brightnessctl")
            .args(["--class=backlight", "--machine-readable"])
            .output()?;
        let text = String::from_utf8_lossy(&out.stdout);
        // "intel_backlight,backlight,60000,50%,120000"
        let line = text.lines().next().unwrap_or("");
        let pct = line.split(',').nth(3).unwrap_or("").trim_end_matches('%');
        pct.parse::<i64>()
            .context("could not read backlight brightness")
    }

    fn set(&self, percent: i64) -> Result<()> {
        let out = Command::new("brightnessctl")
            .args(["--class=backlight", "set", &format!("{percent}%")])
            .output()?;
        if !out.status.success() {
            bail!("failed to set backlight brightness");
        }
        Ok(())
    }

    fn step(&self, delta: i64) -> Result<i64> {
        let target = (self.get()? + delta).clamp(0, 100);
        self.set(target)?;
        Ok(target)
    }

    fn settle(&self) -> Result<()> {
        Ok(())
    }
}
