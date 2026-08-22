use anyhow::Result;

#[derive(Clone, Copy)]
pub enum Icon {
    BrightnessSymbolic,
    DialogError,
    VideoDisplay,
}

impl Icon {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BrightnessSymbolic => "display-brightness-symbolic",
            Self::DialogError => "dialog-error",
            Self::VideoDisplay => "video-display",
        }
    }
}

pub trait NotifierCtl {
    fn send(&self, summary: &str, body: &str, icon: Icon, timeout_ms: Option<u32>) -> Result<()>;
}

pub trait OsdCtl {
    fn show_custom_progress(&self, monitor: &str, progress: f64, icon: Icon) -> Result<()>;
}

/// What a command wired without `--notify` or `--osd` talks to.
pub struct Silent;

impl NotifierCtl for Silent {
    fn send(&self, _: &str, _: &str, _: Icon, _: Option<u32>) -> Result<()> {
        Ok(())
    }
}

impl OsdCtl for Silent {
    fn show_custom_progress(&self, _: &str, _: f64, _: Icon) -> Result<()> {
        Ok(())
    }
}
