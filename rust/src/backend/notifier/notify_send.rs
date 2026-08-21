use std::process::Command;

use anyhow::{bail, Result};

use super::Notifierctl;

pub(super) struct NotifySend;

impl Notifierctl for NotifySend {
    fn send(&self, summary: &str, body: &str, icon: &str, timeout_ms: Option<u32>) -> Result<()> {
        let mut cmd = Command::new("notify-send");
        cmd.arg(summary).arg(body).arg("-i").arg(icon);
        if let Some(t) = timeout_ms {
            cmd.arg("-t").arg(t.to_string());
        }
        let status = cmd.status()?;
        if !status.success() {
            bail!("notify-send failed");
        }
        Ok(())
    }
}
