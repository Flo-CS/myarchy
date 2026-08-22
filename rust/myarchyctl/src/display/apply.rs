use std::thread;
use std::time::Duration;

use anyhow::{bail, Result};
use myarchy_core::compositor::{CompositorCtl, Monitor};
use myarchy_core::error::AppError;
use myarchy_core::layout::Layout;
use myarchy_sys::store;

const SETTLE_TRIES: u32 = 30;
const SETTLE_INTERVAL: Duration = Duration::from_millis(100);

/// One snapshot per command, reconciled with the stored profile before anything is decided.
pub(super) fn read(compositor: &dyn CompositorCtl) -> Result<(Vec<Monitor>, Layout)> {
    let monitors = compositor.described_monitors(true)?;
    let mut layout = store::load(&monitors).unwrap_or_default();
    layout.sync(&monitors);
    Ok((monitors, layout))
}

pub(super) fn commit(
    compositor: &dyn CompositorCtl,
    monitors: &[Monitor],
    layout: Layout,
) -> Result<()> {
    commit_settled(compositor, monitors, layout).map(|_| ())
}

/// Writing the rules is what applies them. The profile is saved once, afterwards, from the settled
/// snapshot — so symbolic requests never reach disk and a crash leaves the previous profile intact.
pub(super) fn commit_settled(
    compositor: &dyn CompositorCtl,
    monitors: &[Monitor],
    mut layout: Layout,
) -> Result<Vec<Monitor>> {
    store::render(&compositor.render_rules(&layout, monitors))?;
    compositor.reload()?;

    let settled = settle(compositor)?;
    layout.sync(&settled);
    store::save(&settled, &layout)?;
    Ok(settled)
}

pub(super) fn restore(
    compositor: &dyn CompositorCtl,
    monitors: &[Monitor],
    mut stored: Layout,
) -> Result<()> {
    if stored.matches(monitors) {
        return Ok(());
    }
    store::render(&compositor.render_rules(&stored, monitors))?;
    compositor.reload()?;

    let settled = settle(compositor)?;
    stored.sync(&settled);
    store::save(&settled, &stored)
}

/// Rules land asynchronously, so a reading is only trusted once two consecutive ones agree.
pub(super) fn settle(compositor: &dyn CompositorCtl) -> Result<Vec<Monitor>> {
    let mut previous = compositor.described_monitors(true)?;
    for _ in 0..SETTLE_TRIES {
        thread::sleep(SETTLE_INTERVAL);
        let current = compositor.described_monitors(true)?;
        if Layout::observe(&current) == Layout::observe(&previous) {
            return Ok(current);
        }
        previous = current;
    }
    bail!(AppError::LayoutDidNotSettle)
}

/// Disabling a monitor does not move its workspaces off it (hyprwm/Hyprland#5052), leaving
/// `MOD+<n>` pointing at a screen with no output. Called once the layout already says which screens
/// are going off, and before the rules that switch them off are written.
pub(super) fn evacuate_workspaces(
    compositor: &dyn CompositorCtl,
    monitors: &[Monitor],
    layout: &Layout,
) -> Result<()> {
    let staying: Vec<&str> = layout
        .screens
        .iter()
        .filter(|(_, screen)| !screen.is_off())
        .filter_map(|(desc, _)| name_of(monitors, desc))
        .collect();

    let Some(target) = layout
        .anchor
        .as_deref()
        .and_then(|desc| name_of(monitors, desc))
        .filter(|name| staying.contains(name))
        .or_else(|| staying.first().copied())
    else {
        return Ok(());
    };

    let leaving: Vec<&str> = layout
        .screens
        .iter()
        .filter(|(_, screen)| screen.is_off())
        .filter_map(|(desc, _)| name_of(monitors, desc))
        .collect();

    for workspace in compositor.workspaces()? {
        if !workspace.is_special() && leaving.contains(&workspace.monitor.as_str()) {
            compositor.move_workspace_to_monitor(&workspace.name, target)?;
        }
    }
    Ok(())
}

pub(super) fn confirm_off(settled: &[Monitor], should_be_off: impl Fn(&str) -> bool) -> Result<()> {
    for monitor in settled {
        if should_be_off(&monitor.name) && !monitor.disabled {
            bail!(AppError::DidNotSwitchOff {
                name: monitor.name.clone()
            });
        }
    }
    Ok(())
}

pub(super) fn describe(monitors: &[Monitor], name: &str) -> Result<String> {
    monitors
        .iter()
        .find(|m| m.name == name)
        .map(|m| m.description().to_string())
        .ok_or_else(|| {
            AppError::UnknownMonitor {
                name: name.to_string(),
            }
            .into()
        })
}

pub(super) fn name_of<'a>(monitors: &'a [Monitor], description: &str) -> Option<&'a str> {
    monitors
        .iter()
        .find(|m| m.description() == description)
        .map(|m| m.name.as_str())
}

/// Wayland has no primary display, so the anchor is only what `extend` and `mirror` build around.
pub(super) fn anchor_description(monitors: &[Monitor], layout: &Layout) -> Result<String> {
    if let Some(desc) = layout.anchor.as_deref() {
        if layout.screens.contains_key(desc) {
            return Ok(desc.to_string());
        }
    }
    if let Some(monitor) = monitors.iter().find(|m| m.focused) {
        return Ok(monitor.description().to_string());
    }
    match monitors.iter().find(|m| !m.disabled) {
        Some(monitor) => Ok(monitor.description().to_string()),
        None => bail!("no enabled screen to build the layout around"),
    }
}
