use crate::core::cursor::CursorCtl;
use crate::core::error::UserError;
use crate::core::store::CursorStore;
use crate::core::theme::ThemeCtl;
use anyhow::{Context, Result};

pub struct Cursor<'a> {
    pub backend: &'a dyn CursorCtl,
    pub store: &'a dyn CursorStore,
    pub theme: &'a dyn ThemeCtl,
}

impl Cursor<'_> {
    pub fn list(&self) -> Result<Vec<String>> {
        self.store.list()
    }

    pub fn set(&self, name: &str, size: i64) -> Result<()> {
        let applied = self.backend.set(name, size);
        self.save_preferred(name, size)?;
        applied
    }

    pub fn apply_preferred(&self) -> Result<()> {
        let (name, size) = match self.store.load_preferred() {
            Some(preferred) => (preferred.name, preferred.size),
            None => {
                let name = self
                    .theme
                    .get_var("cursor-name")?
                    .ok_or(UserError::NoPreferredCursor)?;
                let size = self
                    .theme
                    .get_var("cursor-size")?
                    .ok_or(UserError::NoPreferredCursor)?
                    .parse::<i64>()
                    .context("cursor-size is not a number")
                    .context(UserError::InvalidThemeValue {
                        key: "cursor-size".into(),
                    })?;
                (name, size)
            }
        };
        self.backend.set(&name, size)
    }

    pub fn save_preferred(&self, name: &str, size: i64) -> Result<()> {
        self.store.save_preferred(name, size)?;
        self.theme.render()
    }

    pub fn reset(&self) -> Result<()> {
        self.store.forget_preferred();
        self.apply_preferred()?;
        self.theme.render()
    }
}
