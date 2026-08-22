use anyhow::Result;

pub trait BrightnessCtl {
    fn get(&self) -> Result<i64>;
    fn set(&self, percent: i64) -> Result<()>;
    fn step(&self, delta: i64) -> Result<i64>;
    fn settle(&self) -> Result<()>;
}
