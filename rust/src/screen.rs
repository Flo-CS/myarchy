use crate::models::nightlight::NightlightState;
use crate::ports::brightnessctl;
use crate::ports::compositorctl::Compositor;
use crate::ports::nightlightctl::NightLight;
use crate::ports::notifierctl::Notifier;
use crate::ports::osdctl::Osd;
use anyhow::Result;

fn resolve_backend(
    compositor: &dyn Compositor,
    name: &str,
) -> Result<Box<dyn brightnessctl::Brightness>> {
    let monitors = compositor.monitors(false)?;
    brightnessctl::resolve_adapter(name, &monitors)
}

fn try_resolve_backend(
    compositor: &dyn Compositor,
    name: &str,
) -> Result<Option<Box<dyn brightnessctl::Brightness>>> {
    let monitors = compositor.monitors(false)?;
    brightnessctl::try_resolve_adapter(name, &monitors)
}

pub(crate) fn brightness_get(compositor: &dyn Compositor, name: &str) -> Result<Option<i64>> {
    let Some(backend) = try_resolve_backend(compositor, name)? else {
        return Ok(None);
    };
    Ok(backend.get().ok())
}

pub(crate) fn brightness_set(compositor: &dyn Compositor, name: &str, percent: i64) -> Result<()> {
    resolve_backend(compositor, name)?.set(percent)
}

pub(crate) fn brightness_step(
    compositor: &dyn Compositor,
    osd: &dyn Osd,
    delta_percent: &str,
    name: &str,
) -> Result<()> {
    resolve_backend(compositor, name)?.step(delta_percent, osd)
}

pub(crate) fn brightness_monitors(compositor: &dyn Compositor) -> Result<Vec<String>> {
    let monitors = compositor.monitors(false)?;
    let mut names = Vec::new();
    for m in &monitors {
        if brightnessctl::try_resolve_adapter(&m.name, &monitors)?.is_some() {
            names.push(m.name.clone());
        }
    }
    Ok(names)
}

pub(crate) fn brightness_worker(
    compositor: &dyn Compositor,
    notify: &dyn Notifier,
    name: &str,
) -> Result<()> {
    let monitors = compositor.monitors(false)?;
    brightnessctl::run_worker(notify, name, &monitors)
}

pub(crate) fn nightlight_get(nightlight: &dyn NightLight) -> NightlightState {
    nightlight.get()
}

pub(crate) fn nightlight_set(nightlight: &dyn NightLight, percent: i64) -> Result<()> {
    nightlight.set(percent)
}

pub(crate) fn nightlight_off(nightlight: &dyn NightLight) -> Result<()> {
    nightlight.unset()
}
