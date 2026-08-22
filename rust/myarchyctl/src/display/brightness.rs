use anyhow::Result;

use crate::backend;
use crate::backend::icon::Icon;
use crate::display::CompositorCtl;

pub trait BrightnessCtl {
    fn get(&self) -> Result<i64>;
    fn set(&self, percent: i64) -> Result<()>;
    fn step(&self, delta: i64) -> Result<i64>;
    fn settle(&self) -> Result<()>;
}

pub(crate) trait OsdCtl {
    fn show_custom_progress(&self, monitor: &str, progress: f64, icon: &str) -> Result<()>;
}

pub fn get(name: &str) -> Result<Option<i64>> {
    let monitors = backend::compositorctl().monitors(false)?;
    let Some(backend) = backend::try_brightnessctl(name, &monitors)? else {
        return Ok(None);
    };
    Ok(backend.get().ok())
}

pub fn set(name: &str, percent: i64) -> Result<()> {
    let monitors = backend::compositorctl().monitors(false)?;
    backend::brightnessctl(name, &monitors)?.set(percent)
}

pub fn step(delta_percent: i64, name: &str) -> Result<()> {
    let monitors = backend::compositorctl().monitors(false)?;
    let backend = backend::brightnessctl(name, &monitors)?;
    let percent = backend.step(delta_percent)?;
    backend::osdctl().show_custom_progress(
        name,
        percent as f64 / 100.0,
        Icon::BrightnessSymbolic.as_str(),
    )?;
    backend.settle()
}

pub fn monitors() -> Result<Vec<String>> {
    let monitors = backend::compositorctl().monitors(false)?;
    let mut names = Vec::new();
    for m in &monitors {
        if backend::try_brightnessctl(&m.name, &monitors)?.is_some() {
            names.push(m.name.clone());
        }
    }
    Ok(names)
}
