pub mod brightness;
mod engine;
pub(crate) mod layout;
pub(crate) mod monitor;
pub mod resolution;
mod store;

use std::thread;
use std::time::Duration;

use anyhow::{anyhow, bail, Result};

use crate::desktop::workspace::Workspace;
use crate::display::layout::{Direction, Layout, Mode, Scale, Side};
use crate::display::monitor::Monitor;
use crate::display::resolution::{Resolution, Size};
use crate::error::AppError;

const SETTLE_TRIES: u32 = 30;
const SETTLE_INTERVAL: Duration = Duration::from_millis(100);

pub(crate) trait CompositorCtl {
    fn monitors(&self, all: bool) -> Result<Vec<Monitor>>;
    fn described_monitors(&self, all: bool) -> Result<Vec<Monitor>> {
        Ok(self
            .monitors(all)?
            .into_iter()
            .filter(|m| !m.description().is_empty())
            .collect())
    }

    fn reload(&self) -> Result<()>;
    fn workspaces(&self) -> Result<Vec<Workspace>>;
    fn move_workspace_to_monitor(&self, workspace: &str, monitor: &str) -> Result<()>;
    fn render_rules(&self, layout: &Layout, monitors: &[Monitor]) -> String;
}

pub(crate) trait NotifierCtl {
    fn send(&self, summary: &str, body: &str, icon: &str, timeout_ms: Option<u32>) -> Result<()>;
}

pub(crate) fn list(compositor: &dyn CompositorCtl) -> Result<String> {
    Ok(compositor
        .described_monitors(true)?
        .iter()
        .map(|m| {
            let state = if m.disabled {
                "disabled"
            } else if m.mirror_of.is_some() {
                "mirroring"
            } else {
                "enabled"
            };
            format!(
                "{}\t{}\t{}\t{state}",
                m.name,
                m.description(),
                m.resolution.rounded()
            )
        })
        .collect::<Vec<_>>()
        .join("\n"))
}

pub(crate) fn list_modes(compositor: &dyn CompositorCtl, name: &str) -> Result<String> {
    Ok(compositor
        .described_monitors(true)?
        .iter()
        .find(|m| m.name == name)
        .map(|m| {
            m.resolutions
                .iter()
                .map(Resolution::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default())
}

pub(crate) fn extend(compositor: &dyn CompositorCtl, direction: Direction) -> Result<()> {
    store::locked(|| extend_core(compositor, direction))
}

pub(crate) fn place(
    compositor: &dyn CompositorCtl,
    name: &str,
    side: Side,
    reference: &str,
) -> Result<()> {
    store::locked(|| {
        let (monitors, mut layout) = read(compositor)?;
        let moving = describe(&monitors, name)?;
        let target = describe(&monitors, reference)?;
        engine::place(&mut layout, &moving, side, &target)?;
        commit(compositor, &monitors, layout)
    })
}

pub(crate) fn mirror(compositor: &dyn CompositorCtl) -> Result<()> {
    store::locked(|| {
        let (monitors, mut layout) = read(compositor)?;
        let anchor = anchor_description(&monitors, &layout)?;
        engine::mirror(&mut layout, &anchor);
        commit(compositor, &monitors, layout)
    })
}

pub(crate) fn only(compositor: &dyn CompositorCtl, keep: &str) -> Result<()> {
    store::locked(|| {
        let (monitors, mut layout) = read(compositor)?;
        let keep_desc = describe(&monitors, keep)?;
        engine::only(&mut layout, &keep_desc);
        evacuate_workspaces(compositor, &monitors, &layout)?;
        let settled = commit_settled(compositor, &monitors, layout)?;
        confirm_off(&settled, |name| name != keep)
    })
}

pub(crate) fn enable_monitor(compositor: &dyn CompositorCtl, name: &str) -> Result<()> {
    store::locked(|| enable_core(compositor, name))
}

pub(crate) fn disable_monitor(compositor: &dyn CompositorCtl, name: &str) -> Result<()> {
    store::locked(|| disable_core(compositor, name))
}

pub(crate) fn toggle_monitor(compositor: &dyn CompositorCtl, name: &str) -> Result<()> {
    store::locked(|| {
        let monitors = compositor.described_monitors(true)?;
        let off = monitors
            .iter()
            .find(|m| m.name == name)
            .is_some_and(|m| m.disabled);
        if off {
            enable_core(compositor, name)
        } else {
            disable_core(compositor, name)
        }
    })
}

pub(crate) fn set_mode(compositor: &dyn CompositorCtl, name: &str, mode: &str) -> Result<()> {
    store::locked(|| {
        let (monitors, mut layout) = read(compositor)?;
        let desc = describe(&monitors, name)?;
        let resolved = resolve_mode(&monitors, name, mode)?;
        engine::set_mode(&mut layout, &desc, resolved)?;
        commit(compositor, &monitors, layout)
    })
}

fn resolve_mode(monitors: &[Monitor], name: &str, mode: &str) -> Result<Mode> {
    if mode == "preferred" {
        return Ok(Mode::Preferred);
    }
    if let Ok(resolution) = mode.parse::<Resolution>() {
        return Ok(Mode::Fixed(resolution));
    }
    let size: Size = mode.parse()?;
    let resolution = monitors
        .iter()
        .find(|m| m.name == name)
        .into_iter()
        .flat_map(|m| m.resolutions.iter())
        .filter(|r| r.width == size.width && r.height == size.height)
        .max_by(|a, b| a.refresh.total_cmp(&b.refresh))
        .ok_or_else(|| anyhow!("no {size} mode for {name}"))?;
    Ok(Mode::Fixed(*resolution))
}

pub(crate) fn set_scale(compositor: &dyn CompositorCtl, name: &str, scale: &str) -> Result<()> {
    let scale: Scale = scale.parse()?;
    store::locked(|| {
        let (monitors, mut layout) = read(compositor)?;
        engine::set_scale(&mut layout, &describe(&monitors, name)?, scale)?;
        commit(compositor, &monitors, layout)
    })
}

pub(crate) fn set_primary(compositor: &dyn CompositorCtl, name: &str) -> Result<()> {
    store::locked(|| {
        let (monitors, mut layout) = read(compositor)?;
        layout.anchor = Some(describe(&monitors, name)?);
        store::save(&monitors, &layout)
    })
}

pub(crate) fn anchor(compositor: &dyn CompositorCtl) -> Result<Option<String>> {
    let monitors = compositor.described_monitors(true)?;
    let Some(layout) = store::load(&monitors) else {
        return Ok(None);
    };
    Ok(layout
        .anchor
        .and_then(|desc| name_of(&monitors, &desc).map(str::to_string)))
}

pub(crate) fn save(compositor: &dyn CompositorCtl) -> Result<()> {
    store::locked(|| {
        let (monitors, layout) = read(compositor)?;
        store::render(&compositor.render_rules(&layout, &monitors))?;
        store::save(&monitors, &layout)
    })
}

pub(crate) fn apply(compositor: &dyn CompositorCtl) -> Result<()> {
    store::locked(|| {
        let monitors = settle(compositor)?;
        match store::load(&monitors) {
            Some(stored) => restore(compositor, &monitors, stored),
            None => Ok(()),
        }
    })
}

pub(crate) fn reset(compositor: &dyn CompositorCtl) -> Result<()> {
    store::locked(|| {
        store::reset(&compositor.described_monitors(true)?);
        compositor.reload()
    })
}

/// Entry point for the monitor.added/removed and hyprland.start hooks.
pub(crate) fn auto(compositor: &dyn CompositorCtl, notify: &dyn NotifierCtl) -> Result<()> {
    store::locked(|| {
        let monitors = settle(compositor)?;

        if let Some(stored) = store::load(&monitors) {
            return restore(compositor, &monitors, stored);
        }

        if monitors.len() <= 1 {
            return store::save(&monitors, &Layout::observe(&monitors));
        }

        extend_core(compositor, Direction::Right)?;

        let fresh = compositor.described_monitors(true)?;
        if let Some(name) = fresh
            .iter()
            .filter(|m| !m.disabled)
            .map(|m| &m.name)
            .next_back()
        {
            let _ = notify.send(
                "Screen connected",
                &format!("{name} extended to the right — MOD+P for display options"),
                "video-display",
                Some(8000),
            );
        }
        Ok(())
    })
}

fn extend_core(compositor: &dyn CompositorCtl, direction: Direction) -> Result<()> {
    let (monitors, mut layout) = read(compositor)?;
    let anchor = anchor_description(&monitors, &layout)?;
    engine::extend(&mut layout, &anchor, direction);
    commit(compositor, &monitors, layout)
}

fn enable_core(compositor: &dyn CompositorCtl, name: &str) -> Result<()> {
    let (monitors, mut layout) = read(compositor)?;
    engine::enable(&mut layout, &describe(&monitors, name)?)?;
    let settled = commit_settled(compositor, &monitors, layout)?;
    if settled
        .iter()
        .any(|m| m.name == name && (m.disabled || m.mirror_of.is_some()))
    {
        bail!(AppError::DidNotSwitchOn {
            name: name.to_string()
        });
    }
    Ok(())
}

fn disable_core(compositor: &dyn CompositorCtl, name: &str) -> Result<()> {
    let (monitors, mut layout) = read(compositor)?;
    let desc = describe(&monitors, name)?;
    engine::disable(&mut layout, &desc)?;
    evacuate_workspaces(compositor, &monitors, &layout)?;
    let settled = commit_settled(compositor, &monitors, layout)?;
    confirm_off(&settled, |n| n == name)
}

/// One snapshot per command, reconciled with the stored profile before anything is decided.
fn read(compositor: &dyn CompositorCtl) -> Result<(Vec<Monitor>, Layout)> {
    let monitors = compositor.described_monitors(true)?;
    let mut layout = store::load(&monitors).unwrap_or_default();
    layout.sync(&monitors);
    Ok((monitors, layout))
}

fn commit(compositor: &dyn CompositorCtl, monitors: &[Monitor], layout: Layout) -> Result<()> {
    commit_settled(compositor, monitors, layout).map(|_| ())
}

/// Writing the rules is what applies them. The profile is saved once, afterwards, from the settled
/// snapshot — so symbolic requests never reach disk and a crash leaves the previous profile intact.
fn commit_settled(
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

fn restore(compositor: &dyn CompositorCtl, monitors: &[Monitor], mut stored: Layout) -> Result<()> {
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
fn settle(compositor: &dyn CompositorCtl) -> Result<Vec<Monitor>> {
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
fn evacuate_workspaces(
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

fn confirm_off(settled: &[Monitor], should_be_off: impl Fn(&str) -> bool) -> Result<()> {
    for monitor in settled {
        if should_be_off(&monitor.name) && !monitor.disabled {
            bail!(AppError::DidNotSwitchOff {
                name: monitor.name.clone()
            });
        }
    }
    Ok(())
}

fn describe(monitors: &[Monitor], name: &str) -> Result<String> {
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

fn name_of<'a>(monitors: &'a [Monitor], description: &str) -> Option<&'a str> {
    monitors
        .iter()
        .find(|m| m.description() == description)
        .map(|m| m.name.as_str())
}

/// Wayland has no primary display, so the anchor is only what `extend` and `mirror` build around.
fn anchor_description(monitors: &[Monitor], layout: &Layout) -> Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::compositorctl as backend;
    use crate::display::layout::fixtures::{laptop, switched_off, ultrawide, BOE, LG};

    fn rules_for(monitors: &[Monitor], apply: impl FnOnce(&mut Layout)) -> String {
        let mut layout = Layout::observe(monitors);
        apply(&mut layout);
        backend().render_rules(&layout, monitors)
    }

    #[test]
    fn plugging_in_the_ultrawide_and_extending_right() {
        let monitors = [laptop(), ultrawide()];
        let rules = rules_for(&monitors, |layout| {
            engine::extend(layout, LG, Direction::Right)
        });

        assert_eq!(
            rules,
            concat!(
                "hl.monitor({ output = \"desc:BOE 0x08B9\", mode = \"preferred\", position = \"auto-right\", scale = \"1\", mirror = \"none\", disabled = false })\n",
                "hl.monitor({ output = \"desc:LG Electronics LG HDR WQHD 303NTZN51357\", mode = \"preferred\", position = \"0x0\", scale = \"1\", mirror = \"none\", disabled = false })\n",
            )
        );
    }

    #[test]
    fn putting_the_laptop_under_the_ultrawide() {
        let monitors = [laptop(), ultrawide()];
        let rules = rules_for(&monitors, |layout| {
            engine::place(layout, BOE, Side::Below, LG).unwrap()
        });

        assert_eq!(
            rules,
            concat!(
                "hl.monitor({ output = \"desc:BOE 0x08B9\", mode = \"1920x1080@60.003\", position = \"760x1440\", scale = \"1\", mirror = \"none\", disabled = false })\n",
                "hl.monitor({ output = \"desc:LG Electronics LG HDR WQHD 303NTZN51357\", mode = \"3440x1440@99.997\", position = \"0x0\", scale = \"1\", mirror = \"none\", disabled = false })\n",
            )
        );
    }

    #[test]
    fn resolve_mode_picks_the_highest_refresh_matching_a_bare_size() {
        let mut dual_refresh = laptop();
        dual_refresh.resolutions = vec![
            "1920x1080@60.003".parse().unwrap(),
            "1920x1080@59.951".parse().unwrap(),
        ];
        let monitors = [dual_refresh];

        assert_eq!(
            resolve_mode(&monitors, "eDP-1", "1920x1080").unwrap(),
            Mode::Fixed("1920x1080@60.003".parse().unwrap())
        );
        assert_eq!(
            resolve_mode(&monitors, "eDP-1", "preferred").unwrap(),
            Mode::Preferred
        );
        assert_eq!(
            resolve_mode(&monitors, "eDP-1", "3440x1440")
                .unwrap_err()
                .to_string(),
            "no 3440x1440 mode for eDP-1"
        );
    }

    #[test]
    fn a_profile_survives_the_screen_being_switched_off_and_reloaded() {
        let mut layout = Layout::observe(&[laptop(), ultrawide()]);
        engine::disable(&mut layout, LG).unwrap();

        layout.sync(&[laptop(), switched_off(ultrawide())]);

        assert_eq!(
            backend().render_rules(&layout, &[laptop(), ultrawide()]),
            concat!(
                "hl.monitor({ output = \"desc:BOE 0x08B9\", mode = \"1920x1080@60.003\", position = \"0x0\", scale = \"1\", mirror = \"none\", disabled = false })\n",
                "hl.monitor({ output = \"desc:LG Electronics LG HDR WQHD 303NTZN51357\", disabled = true })\n",
            )
        );
        assert_eq!(
            layout.screens[LG].placement.position,
            "1920x0".parse().unwrap()
        );
    }
}
