use crate::backend::brightness;
use crate::backend::compositor::Compositorctl;
use crate::backend::osd::Osdctl;
use crate::icon::Icon;
use anyhow::Result;

pub(crate) fn get(compositor: &dyn Compositorctl, name: &str) -> Result<Option<i64>> {
    let monitors = compositor.monitors(false)?;
    let Some(backend) = brightness::try_resolve_backend(name, &monitors)? else {
        return Ok(None);
    };
    Ok(backend.get().ok())
}

pub(crate) fn set(compositor: &dyn Compositorctl, name: &str, percent: i64) -> Result<()> {
    let monitors = compositor.monitors(false)?;
    brightness::resolve_backend(name, &monitors)?.set(percent)
}

pub(crate) fn step(
    compositor: &dyn Compositorctl,
    osd: &dyn Osdctl,
    delta_percent: i64,
    name: &str,
) -> Result<()> {
    let monitors = compositor.monitors(false)?;
    let backend = brightness::resolve_backend(name, &monitors)?;
    let percent = backend.step(delta_percent)?;
    osd.show_custom_progress(
        name,
        percent as f64 / 100.0,
        Icon::BrightnessSymbolic.as_str(),
    )?;
    backend.settle()
}

pub(crate) fn monitors(compositor: &dyn Compositorctl) -> Result<Vec<String>> {
    let monitors = compositor.monitors(false)?;
    let mut names = Vec::new();
    for m in &monitors {
        if brightness::try_resolve_backend(&m.name, &monitors)?.is_some() {
            names.push(m.name.clone());
        }
    }
    Ok(names)
}
