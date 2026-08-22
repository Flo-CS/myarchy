use anyhow::Result;

pub trait IdleCtl {
    fn enable(&self) -> Result<()>;
    fn disable(&self) -> Result<()>;
    fn status(&self) -> Result<bool>;
    /// Streams until the connection closes, calling `on_change` with each new state.
    fn watch(&self, on_change: &mut dyn FnMut(bool) -> Result<()>) -> Result<()>;
}
