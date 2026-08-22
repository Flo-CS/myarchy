use anyhow::Result;

pub trait ThemeCtl {
    fn get_var(&self, key: &str) -> Result<Option<String>>;
    fn render(&self) -> Result<()>;
}
