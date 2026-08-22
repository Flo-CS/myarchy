use anyhow::Result;

pub trait CursorCtl {
    fn set(&self, name: &str, size: i64) -> Result<()>;
}
