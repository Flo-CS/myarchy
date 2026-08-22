use crate::core::brightness::{BrightnessCtl, BrightnessSource};
use crate::core::compositor::Monitor;
use anyhow::Result;

use crate::host::backlight::Backlight;
use crate::host::ddc::Ddc;

pub struct Backends;

impl BrightnessSource for Backends {
    fn open(&self, name: &str, monitors: &[Monitor]) -> Result<Option<Box<dyn BrightnessCtl>>> {
        let Some(monitor) = monitors.iter().find(|m| m.name == name) else {
            return Ok(None);
        };

        if monitor.is_internal() {
            return Ok(Some(Box::new(Backlight)));
        }
        Ok(Ddc::display_for(name, monitors)?
            .map(|display| Box::new(Ddc::new(name.to_string(), display)) as Box<dyn BrightnessCtl>))
    }
}
