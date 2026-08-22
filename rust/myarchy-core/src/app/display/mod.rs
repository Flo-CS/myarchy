mod reconcile;
#[cfg(test)]
mod tests;

use std::time::Duration;

use crate::core::compositor::{CompositorCtl, Monitor, description_of, name_of};
use crate::core::error::UserError;
use crate::core::layout::{Direction, Layout, Mode, Scale, Side};
use crate::core::notify::{Icon, NotifierCtl};
use crate::core::resolution::{Resolution, Size};
use crate::core::store::LayoutStore;
use anyhow::{Result, bail};

/// Rules land asynchronously, so how long a reading is given to stop changing.
#[derive(Clone, Copy)]
pub struct Settle {
    pub tries: u32,
    pub interval: Duration,
}

impl Default for Settle {
    fn default() -> Self {
        Self {
            tries: 30,
            interval: Duration::from_millis(100),
        }
    }
}

pub struct Display<'a> {
    pub compositor: &'a dyn CompositorCtl,
    pub store: &'a dyn LayoutStore,
    pub notifier: &'a dyn NotifierCtl,
    pub settle: Settle,
}

impl Display<'_> {
    pub fn monitors(&self) -> Result<Vec<Monitor>> {
        self.compositor.monitors()
    }

    pub fn modes(&self, name: &str) -> Result<Vec<Resolution>> {
        let monitors = self.monitors()?;
        description_of(&monitors, name)?;
        Ok(monitors
            .into_iter()
            .find(|m| m.name == name)
            .map(|m| m.resolutions)
            .unwrap_or_default())
    }

    pub fn anchor(&self) -> Result<Option<String>> {
        let monitors = self.monitors()?;
        let Some(layout) = self.store.load(&monitors)? else {
            return Ok(None);
        };
        Ok(layout
            .anchor
            .and_then(|desc| name_of(&monitors, &desc).map(str::to_string)))
    }

    pub fn extend(&self, direction: Direction) -> Result<()> {
        self.store.locked(&mut || self.extend_now(direction))
    }

    pub fn place(&self, name: &str, side: Side, reference: &str) -> Result<()> {
        self.store.locked(&mut || {
            let (monitors, mut layout) = self.read()?;
            let moving = description_of(&monitors, name)?.to_string();
            let target = description_of(&monitors, reference)?.to_string();
            layout.place(&moving, side, &target)?;
            self.commit(&monitors, layout)
        })
    }

    pub fn mirror(&self) -> Result<()> {
        self.store.locked(&mut || {
            let (monitors, mut layout) = self.read()?;
            let anchor = self.anchor_description(&monitors, &layout)?;
            layout.mirror(&anchor);
            self.commit(&monitors, layout)
        })
    }

    pub fn only(&self, keep: &str) -> Result<()> {
        self.store.locked(&mut || {
            let (monitors, mut layout) = self.read()?;
            let keep_desc = description_of(&monitors, keep)?.to_string();
            layout.only(&keep_desc);
            self.evacuate_workspaces(&monitors, &layout)?;
            let settled = self.commit_settled(&monitors, layout)?;
            confirm_off(&settled, |name| name != keep)
        })
    }

    pub fn enable(&self, name: &str) -> Result<()> {
        self.store.locked(&mut || self.enable_now(name))
    }

    pub fn disable(&self, name: &str) -> Result<()> {
        self.store.locked(&mut || self.disable_now(name))
    }

    pub fn toggle(&self, name: &str) -> Result<()> {
        self.store.locked(&mut || {
            let monitors = self.monitors()?;
            let off = monitors
                .iter()
                .find(|m| m.name == name)
                .is_some_and(|m| m.disabled);
            if off {
                self.enable_now(name)
            } else {
                self.disable_now(name)
            }
        })
    }

    pub fn set_mode(&self, name: &str, mode: &str) -> Result<()> {
        self.store.locked(&mut || {
            let (monitors, mut layout) = self.read()?;
            let desc = description_of(&monitors, name)?.to_string();
            let resolved = resolve_mode(&monitors, name, mode)?;
            layout.set_mode(&desc, resolved)?;
            self.commit(&monitors, layout)
        })
    }

    pub fn set_scale(&self, name: &str, scale: &str) -> Result<()> {
        let scale: Scale = scale.parse()?;
        self.store.locked(&mut || {
            let (monitors, mut layout) = self.read()?;
            let desc = description_of(&monitors, name)?.to_string();
            layout.set_scale(&desc, scale)?;
            self.commit(&monitors, layout)
        })
    }

    pub fn set_primary(&self, name: &str) -> Result<()> {
        self.store.locked(&mut || {
            let (monitors, mut layout) = self.read()?;
            layout.anchor = Some(description_of(&monitors, name)?.to_string());
            self.store.save(&monitors, &layout)
        })
    }

    pub fn save(&self) -> Result<()> {
        self.store.locked(&mut || {
            let (monitors, layout) = self.read()?;
            self.store
                .render(&self.compositor.render_rules(&layout, &monitors))?;
            self.store.save(&monitors, &layout)
        })
    }

    pub fn apply(&self) -> Result<()> {
        self.store.locked(&mut || {
            let monitors = self.settle()?;
            match self.store.load(&monitors)? {
                Some(stored) => self.restore(&monitors, stored),
                None => Ok(()),
            }
        })
    }

    pub fn reset(&self) -> Result<()> {
        self.store.locked(&mut || {
            self.store.reset(&self.monitors()?)?;
            self.compositor.reload()
        })
    }

    /// Entry point for the monitor.added/removed and hyprland.start hooks.
    pub fn auto(&self) -> Result<()> {
        self.store.locked(&mut || {
            let monitors = self.settle()?;

            if let Some(stored) = self.store.load(&monitors)? {
                return self.restore(&monitors, stored);
            }

            if monitors.len() <= 1 {
                return self.store.save(&monitors, &Layout::observe(&monitors));
            }

            self.extend_now(Direction::Right)?;

            let fresh = self.monitors()?;
            if let Some(name) = fresh
                .iter()
                .filter(|m| !m.disabled)
                .map(|m| &m.name)
                .next_back()
            {
                let _ = self.notifier.send(
                    "Screen connected",
                    &format!("{name} extended to the right — MOD+P for display options"),
                    Icon::VideoDisplay,
                    Some(8000),
                );
            }
            Ok(())
        })
    }

    fn extend_now(&self, direction: Direction) -> Result<()> {
        let (monitors, mut layout) = self.read()?;
        let anchor = self.anchor_description(&monitors, &layout)?;
        layout.extend(&anchor, direction);
        self.commit(&monitors, layout)
    }

    fn enable_now(&self, name: &str) -> Result<()> {
        let (monitors, mut layout) = self.read()?;
        let desc = description_of(&monitors, name)?.to_string();
        layout.enable(&desc)?;
        let settled = self.commit_settled(&monitors, layout)?;
        if settled
            .iter()
            .any(|m| m.name == name && (m.disabled || m.mirror_of.is_some()))
        {
            bail!(UserError::DidNotSwitchOn {
                name: name.to_string()
            });
        }
        Ok(())
    }

    fn disable_now(&self, name: &str) -> Result<()> {
        let (monitors, mut layout) = self.read()?;
        let desc = description_of(&monitors, name)?.to_string();
        layout.disable(&desc)?;
        self.evacuate_workspaces(&monitors, &layout)?;
        let settled = self.commit_settled(&monitors, layout)?;
        confirm_off(&settled, |n| n == name)
    }
}

fn confirm_off(settled: &[Monitor], should_be_off: impl Fn(&str) -> bool) -> Result<()> {
    for monitor in settled {
        if should_be_off(&monitor.name) && !monitor.disabled {
            bail!(UserError::DidNotSwitchOff {
                name: monitor.name.clone()
            });
        }
    }
    Ok(())
}

fn resolve_mode(monitors: &[Monitor], name: &str, mode: &str) -> Result<Mode> {
    if mode == "preferred" {
        return Ok(Mode::Preferred);
    }
    if let Ok(resolution) = mode.parse::<Resolution>() {
        return Ok(Mode::Fixed(resolution));
    }
    let size: Size = mode.parse().map_err(|_| UserError::InvalidMode {
        value: mode.to_string(),
    })?;
    let resolution = monitors
        .iter()
        .find(|m| m.name == name)
        .into_iter()
        .flat_map(|m| m.resolutions.iter())
        .filter(|r| r.size == size)
        .max_by(|a, b| a.refresh.total_cmp(&b.refresh))
        .ok_or_else(|| UserError::NoSuchMode {
            name: name.to_string(),
            size,
        })?;
    Ok(Mode::Fixed(*resolution))
}
