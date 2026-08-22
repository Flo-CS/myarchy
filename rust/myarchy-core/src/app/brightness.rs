use crate::core::brightness::{BrightnessCtl, BrightnessSource};
use crate::core::compositor::{CompositorCtl, Monitor, description_of};
use crate::core::error::UserError;
use crate::core::notify::{Icon, OsdCtl};
use crate::core::percent::Percent;
use anyhow::Result;

pub struct Brightness<'a> {
    pub compositor: &'a dyn CompositorCtl,
    pub source: &'a dyn BrightnessSource,
    pub osd: &'a dyn OsdCtl,
}

impl Brightness<'_> {
    /// `None` when the screen is known but has no brightness control.
    pub fn get(&self, name: &str, dirty: bool) -> Result<Option<Percent>> {
        match self.open(name)? {
            Some(control) => control.get(dirty).map(Some),
            None => Ok(None),
        }
    }

    pub fn set(&self, name: &str, percent: Percent, dirty: bool) -> Result<()> {
        self.require(name)?.set(percent, dirty)
    }

    pub fn step(&self, delta_percent: i64, name: &str) -> Result<()> {
        let control = self.require(name)?;
        let percent = control.step(delta_percent)?;
        self.osd
            .show_custom_progress(name, percent.fraction(), Icon::BrightnessSymbolic)?;
        control.settle()
    }

    pub fn monitors(&self) -> Result<Vec<String>> {
        let monitors = self.lit()?;
        let mut names = Vec::new();
        for monitor in &monitors {
            if self.source.open(&monitor.name, &monitors)?.is_some() {
                names.push(monitor.name.clone());
            }
        }
        Ok(names)
    }

    /// A screen that is switched off has no brightness to read or write.
    fn lit(&self) -> Result<Vec<Monitor>> {
        Ok(self
            .compositor
            .monitors()?
            .into_iter()
            .filter(|m| !m.disabled)
            .collect())
    }

    fn open(&self, name: &str) -> Result<Option<Box<dyn BrightnessCtl>>> {
        let monitors = self.lit()?;
        description_of(&monitors, name)?;
        self.source.open(name, &monitors)
    }

    fn require(&self, name: &str) -> Result<Box<dyn BrightnessCtl>> {
        self.open(name)?.ok_or_else(|| {
            UserError::DdcNotResponding {
                name: name.to_string(),
            }
            .into()
        })
    }
}
