mod apply;
pub mod cli;

use anyhow::{anyhow, bail, Result};
use myarchy_core::compositor::{CompositorCtl, Monitor};
use myarchy_core::engine;
use myarchy_core::error::AppError;
use myarchy_core::layout::{Direction, Layout, Mode, Scale, Side};
use myarchy_core::notify::{Icon, NotifierCtl};
use myarchy_core::resolution::{Resolution, Size};
use myarchy_host::store::display as store;

use apply::{
    anchor_description, commit, commit_settled, confirm_off, describe, evacuate_workspaces,
    name_of, read, restore, settle,
};

pub fn list(compositor: &dyn CompositorCtl) -> Result<String> {
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

pub fn list_modes(compositor: &dyn CompositorCtl, name: &str) -> Result<String> {
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

pub fn extend(compositor: &dyn CompositorCtl, direction: Direction) -> Result<()> {
    store::locked(|| extend_core(compositor, direction))
}

pub fn place(
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

pub fn mirror(compositor: &dyn CompositorCtl) -> Result<()> {
    store::locked(|| {
        let (monitors, mut layout) = read(compositor)?;
        let anchor = anchor_description(&monitors, &layout)?;
        engine::mirror(&mut layout, &anchor);
        commit(compositor, &monitors, layout)
    })
}

pub fn only(compositor: &dyn CompositorCtl, keep: &str) -> Result<()> {
    store::locked(|| {
        let (monitors, mut layout) = read(compositor)?;
        let keep_desc = describe(&monitors, keep)?;
        engine::only(&mut layout, &keep_desc);
        evacuate_workspaces(compositor, &monitors, &layout)?;
        let settled = commit_settled(compositor, &monitors, layout)?;
        confirm_off(&settled, |name| name != keep)
    })
}

pub fn enable_monitor(compositor: &dyn CompositorCtl, name: &str) -> Result<()> {
    store::locked(|| enable_core(compositor, name))
}

pub fn disable_monitor(compositor: &dyn CompositorCtl, name: &str) -> Result<()> {
    store::locked(|| disable_core(compositor, name))
}

pub fn toggle_monitor(compositor: &dyn CompositorCtl, name: &str) -> Result<()> {
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

pub fn set_mode(compositor: &dyn CompositorCtl, name: &str, mode: &str) -> Result<()> {
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

pub fn set_scale(compositor: &dyn CompositorCtl, name: &str, scale: &str) -> Result<()> {
    let scale: Scale = scale.parse()?;
    store::locked(|| {
        let (monitors, mut layout) = read(compositor)?;
        engine::set_scale(&mut layout, &describe(&monitors, name)?, scale)?;
        commit(compositor, &monitors, layout)
    })
}

pub fn set_primary(compositor: &dyn CompositorCtl, name: &str) -> Result<()> {
    store::locked(|| {
        let (monitors, mut layout) = read(compositor)?;
        layout.anchor = Some(describe(&monitors, name)?);
        store::save(&monitors, &layout)
    })
}

pub fn anchor(compositor: &dyn CompositorCtl) -> Result<Option<String>> {
    let monitors = compositor.described_monitors(true)?;
    let Some(layout) = store::load(&monitors) else {
        return Ok(None);
    };
    Ok(layout
        .anchor
        .and_then(|desc| name_of(&monitors, &desc).map(str::to_string)))
}

pub fn save(compositor: &dyn CompositorCtl) -> Result<()> {
    store::locked(|| {
        let (monitors, layout) = read(compositor)?;
        store::render(&compositor.render_rules(&layout, &monitors))?;
        store::save(&monitors, &layout)
    })
}

pub fn apply(compositor: &dyn CompositorCtl) -> Result<()> {
    store::locked(|| {
        let monitors = settle(compositor)?;
        match store::load(&monitors) {
            Some(stored) => restore(compositor, &monitors, stored),
            None => Ok(()),
        }
    })
}

pub fn reset(compositor: &dyn CompositorCtl) -> Result<()> {
    store::locked(|| {
        store::reset(&compositor.described_monitors(true)?);
        compositor.reload()
    })
}

/// Entry point for the monitor.added/removed and hyprland.start hooks.
pub fn auto(compositor: &dyn CompositorCtl, notify: &dyn NotifierCtl) -> Result<()> {
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
                Icon::VideoDisplay.as_str(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::compositorctl as backend;
    use myarchy_core::layout::fixtures::{laptop, switched_off, ultrawide, BOE, LG};

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
