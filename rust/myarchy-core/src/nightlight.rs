use std::fmt;

use anyhow::Result;

pub enum NightlightState {
    Off,
    On(i64),
}

impl fmt::Display for NightlightState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NightlightState::Off => write!(f, "off"),
            NightlightState::On(percent) => write!(f, "{percent}"),
        }
    }
}

pub trait NightLightCtl {
    fn get(&self) -> NightlightState;
    fn set(&self, percent: i64) -> Result<()>;
    fn unset(&self) -> Result<()>;
}
