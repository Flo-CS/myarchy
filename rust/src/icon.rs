pub(crate) enum Icon {
    BrightnessSymbolic,
    DialogError,
}

impl Icon {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::BrightnessSymbolic => "display-brightness-symbolic",
            Self::DialogError => "dialog-error",
        }
    }
}
