mod notify_send;

use anyhow::Result;

pub(crate) trait Notifierctl {
    fn send(&self, summary: &str, body: &str, icon: &str, timeout_ms: Option<u32>) -> Result<()>;
}

pub(crate) fn backend() -> impl Notifierctl {
    notify_send::NotifySend
}
