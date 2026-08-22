use anyhow::Result;

use crate::core::compositor::Monitor;
use crate::core::percent::Percent;

pub trait BrightnessCtl {
    /// `dirty` returns the last value set instead of confirming it against the monitor.
    fn get(&self, dirty: bool) -> Result<Percent>;
    /// `dirty` fires the write off without waiting for the monitor to confirm it landed.
    fn set(&self, percent: Percent, dirty: bool) -> Result<()>;
    fn step(&self, delta: i64) -> Result<Percent>;
    fn settle(&self) -> Result<()>;
}

pub trait BrightnessSource {
    /// `None` when the screen has no brightness control at all, which is not a failure.
    fn open(&self, name: &str, monitors: &[Monitor]) -> Result<Option<Box<dyn BrightnessCtl>>>;
}
