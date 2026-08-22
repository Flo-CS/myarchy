use anyhow::{bail, Result};
use myarchy_core::notify::NotifierCtl;

use crate::proc;

pub struct NotifySend;

impl NotifierCtl for NotifySend {
    fn send(&self, summary: &str, body: &str, icon: &str, timeout_ms: Option<u32>) -> Result<()> {
        let timeout = timeout_ms.map(|t| t.to_string());
        let mut args = vec![summary, body, "-i", icon];
        if let Some(timeout) = timeout.as_deref() {
            args.extend(["-t", timeout]);
        }
        if !proc::status("notify-send", &args)? {
            bail!("notify-send failed");
        }
        Ok(())
    }
}
