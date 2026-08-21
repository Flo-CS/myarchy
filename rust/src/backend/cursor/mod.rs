mod hyprctl;

use anyhow::Result;

pub(crate) trait Cursorctl {
    fn set(&self, name: &str, size: i64) -> Result<()>;
}

pub(crate) fn backend() -> impl Cursorctl {
    hyprctl::HyprctlCursor
}
