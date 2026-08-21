mod backlight;
mod ddc;

use anyhow::Result;

use crate::error::AppError;
use crate::models::monitor::Monitor;

use backlight::Backlight;
use ddc::Ddc;

pub(crate) trait BrightnessCtl {
    fn get(&self) -> Result<i64>;
    fn set(&self, percent: i64) -> Result<()>;
    fn step(&self, delta: i64) -> Result<i64>;
    fn settle(&self) -> Result<()>;
}

fn is_internal(name: &str) -> bool {
    name.starts_with("eDP-") || name.starts_with("LVDS-") || name.starts_with("DSI-")
}

pub(crate) fn resolve_backend(name: &str, monitors: &[Monitor]) -> Result<Box<dyn BrightnessCtl>> {
    try_resolve_backend(name, monitors)?.ok_or_else(|| {
        AppError::DdcNotResponding {
            name: name.to_string(),
        }
        .into()
    })
}

pub(crate) fn try_resolve_backend(
    name: &str,
    monitors: &[Monitor],
) -> Result<Option<Box<dyn BrightnessCtl>>> {
    if is_internal(name) {
        return Ok(Some(Box::new(Backlight)));
    }
    Ok(Ddc::display_for(name, monitors)?
        .map(|display| Box::new(Ddc::new(name.to_string(), display)) as Box<dyn BrightnessCtl>))
}
