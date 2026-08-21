use std::process::Command;

use anyhow::{Context, Result};

pub(crate) fn get_var(key: &str) -> Result<Option<String>> {
    let out = Command::new("myarchy-theme")
        .args(["get-var", key])
        .output()
        .context("failed to run myarchy-theme get-var")?;
    let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok((!value.is_empty()).then_some(value))
}

pub(crate) fn render() -> Result<()> {
    Command::new("myarchy-theme")
        .arg("render")
        .status()
        .context("failed to run myarchy-theme render")?;
    Ok(())
}
