use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use crate::core::error::UserError;
use crate::core::nightlight::{NightlightCtl, NightlightState};
use crate::core::percent::Percent;
use anyhow::{Result, anyhow};

use crate::host::{file, proc, xdg};

const MIN_KELVIN: i64 = 2500;
const MAX_KELVIN: i64 = 6500;

fn kelvin_from_percent(percent: Percent) -> i64 {
    MAX_KELVIN - percent.get() * (MAX_KELVIN - MIN_KELVIN) / 100
}

fn percent_from_kelvin(kelvin: i64) -> Percent {
    Percent::new((MAX_KELVIN - kelvin) * 100 / (MAX_KELVIN - MIN_KELVIN))
}

fn state_file() -> PathBuf {
    xdg::runtime_dir()
        .join("screen")
        .join("nightlight-temperature")
}

pub struct Hyprsunset;

impl Hyprsunset {
    fn running() -> bool {
        proc::ok("pgrep", &["-x", "hyprsunset"])
    }

    fn ensure_daemon() -> Result<()> {
        if Self::running() {
            return Ok(());
        }
        proc::spawn_detached("uwsm-app", &["--", "hyprsunset"])?;

        // let it claim its socket before the first request
        thread::sleep(Duration::from_millis(500));
        Ok(())
    }
}

impl NightlightCtl for Hyprsunset {
    fn get(&self) -> NightlightState {
        fs::read_to_string(state_file())
            .ok()
            .and_then(|s| s.trim().parse::<i64>().ok())
            .map(|kelvin| NightlightState::On(percent_from_kelvin(kelvin)))
            .unwrap_or(NightlightState::Off)
    }

    fn set(&self, percent: Percent) -> Result<()> {
        Self::ensure_daemon()?;

        let kelvin = kelvin_from_percent(percent);
        if !proc::ok(
            "hyprctl",
            &["hyprsunset", "temperature", &kelvin.to_string()],
        ) {
            return Err(
                anyhow!("hyprsunset did not accept a temperature of {kelvin}K")
                    .context(UserError::NightlightNotApplied),
            );
        }
        file::write_atomic(&state_file(), &format!("{kelvin}\n"))
    }

    fn unset(&self) -> Result<()> {
        if Self::running() {
            let _ = proc::output("hyprctl", &["hyprsunset", "identity"]);
        }
        let _ = fs::remove_file(state_file());
        Ok(())
    }
}
