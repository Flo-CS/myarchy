use std::fmt;

use anyhow::Result;

use crate::core::percent::Percent;

pub enum NightlightState {
    Off,
    On(Percent),
}

impl fmt::Display for NightlightState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NightlightState::Off => write!(f, "off"),
            NightlightState::On(percent) => write!(f, "{percent}"),
        }
    }
}

pub trait NightlightCtl {
    fn get(&self) -> NightlightState;
    fn set(&self, percent: Percent) -> Result<()>;
    fn unset(&self) -> Result<()>;
}
