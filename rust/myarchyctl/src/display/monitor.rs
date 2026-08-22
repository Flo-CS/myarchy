use crate::display::resolution::Resolution;

#[derive(Debug, Clone)]
pub(crate) struct Monitor {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) resolution: Resolution,
    pub(crate) x: i64,
    pub(crate) y: i64,
    pub(crate) scale: f64,
    pub(crate) disabled: bool,
    pub(crate) focused: bool,
    pub(crate) mirror_of: Option<String>,
    pub(crate) resolutions: Vec<Resolution>,
}

impl Monitor {
    pub(crate) fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("")
    }

    pub(crate) fn is_internal(&self) -> bool {
        self.name.starts_with("eDP-")
            || self.name.starts_with("LVDS-")
            || self.name.starts_with("DSI-")
    }
}
