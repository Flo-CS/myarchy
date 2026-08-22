use std::thread;

use crate::core::compositor::{Monitor, name_of};
use crate::core::error::UserError;
use crate::core::layout::Layout;
use anyhow::{Result, bail};

use super::Display;

impl Display<'_> {
    /// One snapshot per command, reconciled with the stored profile before anything is decided.
    pub(super) fn read(&self) -> Result<(Vec<Monitor>, Layout)> {
        let monitors = self.monitors()?;
        let mut layout = self.store.load(&monitors)?.unwrap_or_default();
        layout.sync(&monitors);
        Ok((monitors, layout))
    }

    pub(super) fn commit(&self, monitors: &[Monitor], layout: Layout) -> Result<()> {
        self.commit_settled(monitors, layout).map(|_| ())
    }

    /// Writing the rules is what applies them. The profile is saved once, afterwards, from the
    /// settled snapshot — so symbolic requests never reach disk and a crash leaves the previous
    /// profile intact.
    pub(super) fn commit_settled(
        &self,
        monitors: &[Monitor],
        mut layout: Layout,
    ) -> Result<Vec<Monitor>> {
        self.store
            .render(&self.compositor.render_rules(&layout, monitors))?;
        self.compositor.reload()?;

        let settled = self.settle()?;
        layout.sync(&settled);
        self.store.save(&settled, &layout)?;
        Ok(settled)
    }

    pub(super) fn restore(&self, monitors: &[Monitor], mut stored: Layout) -> Result<()> {
        if stored.matches(monitors) {
            return Ok(());
        }
        self.store
            .render(&self.compositor.render_rules(&stored, monitors))?;
        self.compositor.reload()?;

        let settled = self.settle()?;
        stored.sync(&settled);
        self.store.save(&settled, &stored)
    }

    /// A reading is only trusted once two consecutive ones agree.
    pub(super) fn settle(&self) -> Result<Vec<Monitor>> {
        let mut previous = self.monitors()?;
        for _ in 0..self.settle.tries {
            thread::sleep(self.settle.interval);
            let current = self.monitors()?;
            if Layout::observe(&current) == Layout::observe(&previous) {
                return Ok(current);
            }
            previous = current;
        }
        bail!(UserError::LayoutDidNotSettle)
    }

    /// Disabling a monitor does not move its workspaces off it (hyprwm/Hyprland#5052), leaving
    /// `MOD+<n>` pointing at a screen with no output. Called once the layout already says which
    /// screens are going off, and before the rules that switch them off are written.
    pub(super) fn evacuate_workspaces(&self, monitors: &[Monitor], layout: &Layout) -> Result<()> {
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

        for workspace in self.compositor.workspaces()? {
            if !workspace.is_special() && leaving.contains(&workspace.monitor.as_str()) {
                self.compositor
                    .move_workspace_to_monitor(&workspace.name, target)?;
            }
        }
        Ok(())
    }

    /// Wayland has no primary display, so the anchor is only what `extend` and `mirror` build
    /// around.
    pub(super) fn anchor_description(
        &self,
        monitors: &[Monitor],
        layout: &Layout,
    ) -> Result<String> {
        if let Some(desc) = layout.anchor.as_deref()
            && layout.screens.contains_key(desc)
        {
            return Ok(desc.to_string());
        }
        if let Some(monitor) = monitors.iter().find(|m| m.focused) {
            return Ok(monitor.description.clone());
        }
        match monitors.iter().find(|m| !m.disabled) {
            Some(monitor) => Ok(monitor.description.clone()),
            None => bail!(UserError::NoScreenToBuildAround),
        }
    }
}
