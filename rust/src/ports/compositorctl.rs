use std::process::Command;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::models::monitor::Monitor;

#[derive(Debug, Deserialize)]
struct HyprctlMonitor {
    name: String,
    description: Option<String>,
    width: i64,
    height: i64,
    #[serde(rename = "refreshRate")]
    refresh_rate: f64,
    x: i64,
    y: i64,
    scale: f64,
    #[serde(default)]
    disabled: bool,
    #[serde(default)]
    focused: bool,
    #[serde(rename = "availableModes", default)]
    available_modes: Vec<String>,
}

impl From<HyprctlMonitor> for Monitor {
    fn from(m: HyprctlMonitor) -> Self {
        Monitor {
            name: m.name,
            description: m.description,
            width: m.width,
            height: m.height,
            refresh_rate: m.refresh_rate,
            x: m.x,
            y: m.y,
            scale: m.scale,
            disabled: m.disabled,
            focused: m.focused,
            resolutions: m.available_modes,
        }
    }
}

pub(crate) trait Compositor {
    fn monitors(&self, all: bool) -> Result<Vec<Monitor>>;
    fn reload(&self) -> Result<()>;

    fn described_monitors(&self, all: bool) -> Result<Vec<Monitor>> {
        Ok(self
            .monitors(all)?
            .into_iter()
            .filter(|m| !m.description().is_empty())
            .collect())
    }
}

pub(crate) fn adapter() -> impl Compositor {
    HyprctlCli
}

struct HyprctlCli;

impl Compositor for HyprctlCli {
    fn monitors(&self, all: bool) -> Result<Vec<Monitor>> {
        let mut cmd = Command::new("hyprctl");
        cmd.arg("monitors");
        if all {
            cmd.arg("all");
        }
        cmd.arg("-j");

        let output = cmd.output().context("failed to run hyprctl monitors")?;
        let raw: Vec<HyprctlMonitor> = serde_json::from_slice(&output.stdout)
            .context("failed to parse hyprctl monitors output")?;
        Ok(raw.into_iter().map(Monitor::from).collect())
    }

    fn reload(&self) -> Result<()> {
        Command::new("hyprctl")
            .arg("reload")
            .output()
            .context("failed to run hyprctl reload")?;
        Ok(())
    }
}
