use crate::backend::nightlight::NightLightctl;
use crate::models::nightlight::NightlightState;
use anyhow::Result;

pub(crate) fn get(nightlight: &dyn NightLightctl) -> NightlightState {
    nightlight.get()
}

pub(crate) fn set(nightlight: &dyn NightLightctl, percent: i64) -> Result<()> {
    nightlight.set(percent)
}

pub(crate) fn off(nightlight: &dyn NightLightctl) -> Result<()> {
    nightlight.unset()
}
