use crate::core::theme::ThemeCtl;
use anyhow::Result;

use crate::host::proc;

pub struct MyarchyTheme;

impl ThemeCtl for MyarchyTheme {
    fn get_var(&self, key: &str) -> Result<Option<String>> {
        let out = proc::output("myarchy-theme", &["get-var", key])?;
        let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Ok((!value.is_empty()).then_some(value))
    }

    fn render(&self) -> Result<()> {
        proc::status("myarchy-theme", &["render"])?;
        Ok(())
    }
}
