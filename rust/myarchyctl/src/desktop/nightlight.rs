use anyhow::Result;
use std::fmt;

pub(crate) enum NightlightState {
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

pub(crate) trait NightLightCtl {
    fn get(&self) -> NightlightState;
    fn set(&self, percent: i64) -> Result<()>;
    fn unset(&self) -> Result<()>;
}

pub(crate) fn get() -> NightlightState {
    let nightlight_ctl = crate::backend::nightlightctl();

    nightlight_ctl.get()
}

pub(crate) fn set(percent: i64) -> Result<()> {
    let nightlight_ctl = crate::backend::nightlightctl();

    nightlight_ctl.set(percent)
}

pub(crate) fn off() -> Result<()> {
    let nightlight_ctl = crate::backend::nightlightctl();

    nightlight_ctl.unset()
}
