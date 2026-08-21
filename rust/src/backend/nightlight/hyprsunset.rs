use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use anyhow::{bail, Result};

use crate::backend::xdg;
use crate::error::AppError;
use crate::models::nightlight::NightlightState;

use super::NightLightctl;

const MIN_KELVIN: i64 = 2500;
const MAX_KELVIN: i64 = 6500;

fn kelvin_from_percent(percent: i64) -> i64 {
    let percent = percent.clamp(0, 100);
    MAX_KELVIN - percent * (MAX_KELVIN - MIN_KELVIN) / 100
}

fn percent_from_kelvin(kelvin: i64) -> i64 {
    (MAX_KELVIN - kelvin) * 100 / (MAX_KELVIN - MIN_KELVIN)
}

fn state_file() -> PathBuf {
    xdg::runtime_dir()
        .join("screen")
        .join("nightlight-temperature")
}

pub(super) struct Hyprsunset;

impl Hyprsunset {
    fn running() -> bool {
        Command::new("pgrep")
            .args(["-x", "hyprsunset"])
            .output()
            .map(|out| out.status.success())
            .unwrap_or(false)
    }

    fn installed() -> bool {
        Command::new("sh")
            .args(["-c", "command -v hyprsunset >/dev/null 2>&1"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    fn ensure_daemon() -> Result<()> {
        if Self::running() {
            return Ok(());
        }
        if !Self::installed() {
            bail!(AppError::NightlightFailed {
                message: "hyprsunset is not installed".to_string()
            });
        }

        Command::new("uwsm-app")
            .args(["--", "hyprsunset"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        // let it claim its socket before the first request
        thread::sleep(Duration::from_millis(500));
        Ok(())
    }
}

impl NightLightctl for Hyprsunset {
    fn get(&self) -> NightlightState {
        fs::read_to_string(state_file())
            .ok()
            .and_then(|s| s.trim().parse::<i64>().ok())
            .map(|kelvin| NightlightState::On(percent_from_kelvin(kelvin)))
            .unwrap_or(NightlightState::Off)
    }

    fn set(&self, percent: i64) -> Result<()> {
        Self::ensure_daemon()?;

        let kelvin = kelvin_from_percent(percent);
        let out = Command::new("hyprctl")
            .args(["hyprsunset", "temperature", &kelvin.to_string()])
            .output()?;
        if !out.status.success() {
            bail!(AppError::NightlightFailed {
                message: format!("could not set temperature to {kelvin}K")
            });
        }

        fs::create_dir_all(state_file().parent().unwrap())?;
        fs::write(state_file(), format!("{kelvin}\n"))?;
        Ok(())
    }

    fn unset(&self) -> Result<()> {
        if Self::running() {
            let _ = Command::new("hyprctl")
                .args(["hyprsunset", "identity"])
                .output();
        }
        let _ = fs::remove_file(state_file());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_percentage_survives_the_trip_through_kelvin() {
        assert_eq!(kelvin_from_percent(0), MAX_KELVIN, "none is daylight");
        assert_eq!(kelvin_from_percent(100), MIN_KELVIN, "full is warmest");
        assert_eq!(kelvin_from_percent(50), 4500);

        for percent in 0..=100 {
            assert_eq!(
                percent_from_kelvin(kelvin_from_percent(percent)),
                percent,
                "percent {percent}"
            );
        }
    }

    #[test]
    fn a_percentage_out_of_range_is_clamped() {
        assert_eq!(kelvin_from_percent(-40), MAX_KELVIN);
        assert_eq!(kelvin_from_percent(500), MIN_KELVIN);
    }
}
