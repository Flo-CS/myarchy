#[derive(Debug, Clone)]
pub(crate) struct Monitor {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) width: i64,
    pub(crate) height: i64,
    pub(crate) refresh_rate: f64,
    pub(crate) x: i64,
    pub(crate) y: i64,
    pub(crate) scale: f64,
    pub(crate) disabled: bool,
    pub(crate) focused: bool,
    // TODO: Resolution should be an object
    pub(crate) resolutions: Vec<String>,
}

impl Monitor {
    pub(crate) fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("")
    }

    pub(crate) fn resolution(&self) -> String {
        format!(
            "{}x{}@{}",
            self.width,
            self.height,
            self.refresh_rate.floor()
        )
    }
}
