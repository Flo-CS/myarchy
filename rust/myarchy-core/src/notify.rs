use anyhow::Result;

pub enum Icon {
    BrightnessSymbolic,
    DialogError,
    VideoDisplay,
}

impl Icon {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BrightnessSymbolic => "display-brightness-symbolic",
            Self::DialogError => "dialog-error",
            Self::VideoDisplay => "video-display",
        }
    }
}

pub trait NotifierCtl {
    fn send(&self, summary: &str, body: &str, icon: &str, timeout_ms: Option<u32>) -> Result<()>;
}

pub trait OsdCtl {
    fn show_custom_progress(&self, monitor: &str, progress: f64, icon: &str) -> Result<()>;
}
