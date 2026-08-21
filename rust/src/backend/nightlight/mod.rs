mod hyprsunset;

use anyhow::Result;

use crate::models::nightlight::NightlightState;

pub(crate) trait NightLightctl {
    fn get(&self) -> NightlightState;
    fn set(&self, percent: i64) -> Result<()>;
    fn unset(&self) -> Result<()>;
}

pub(crate) fn backend() -> impl NightLightctl {
    hyprsunset::Hyprsunset
}
