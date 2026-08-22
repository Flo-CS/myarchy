use anyhow::Result;

use crate::proc;

pub fn get_var(key: &str) -> Result<Option<String>> {
    let out = proc::output("myarchy-theme", &["get-var", key])?;
    let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok((!value.is_empty()).then_some(value))
}

pub fn render() -> Result<()> {
    proc::status("myarchy-theme", &["render"])?;
    Ok(())
}
