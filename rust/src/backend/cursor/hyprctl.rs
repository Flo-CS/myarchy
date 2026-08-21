use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::error::AppError;

use super::Cursorctl;

pub(super) struct HyprctlCursor;

impl Cursorctl for HyprctlCursor {
    fn set(&self, name: &str, size: i64) -> Result<()> {
        let out = Command::new("hyprctl")
            .args(["setcursor", name, &size.to_string()])
            .output()
            .context("failed to run hyprctl setcursor")?;
        if !out.status.success() {
            bail!(AppError::CursorFailed {
                message: String::from_utf8_lossy(&out.stdout).trim().to_string()
            });
        }
        Ok(())
    }
}
