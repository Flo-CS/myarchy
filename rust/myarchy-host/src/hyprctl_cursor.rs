use anyhow::{bail, Result};
use myarchy_core::cursor::CursorCtl;
use myarchy_core::error::AppError;

use crate::proc;

pub struct HyprctlCursor;

impl CursorCtl for HyprctlCursor {
    fn set(&self, name: &str, size: i64) -> Result<()> {
        let out = proc::output("hyprctl", &["setcursor", name, &size.to_string()])?;
        if !out.status.success() {
            bail!(AppError::CursorFailed {
                message: String::from_utf8_lossy(&out.stdout).trim().to_string()
            });
        }
        Ok(())
    }
}
