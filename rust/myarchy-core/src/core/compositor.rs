use anyhow::Result;

use crate::core::error::UserError;
use crate::core::layout::Layout;
use crate::core::resolution::Resolution;

#[derive(Debug, Clone)]
pub struct Monitor {
    pub name: String,
    pub description: String,
    pub resolution: Resolution,
    pub x: i64,
    pub y: i64,
    pub scale: f64,
    pub disabled: bool,
    pub focused: bool,
    pub mirror_of: Option<String>,
    pub resolutions: Vec<Resolution>,
}

impl Monitor {
    pub fn is_internal(&self) -> bool {
        self.name.starts_with("eDP-")
            || self.name.starts_with("LVDS-")
            || self.name.starts_with("DSI-")
    }
}

/// Connector names are what a person types; descriptions are what a profile is keyed on.
pub fn description_of<'a>(monitors: &'a [Monitor], name: &str) -> Result<&'a str> {
    monitors
        .iter()
        .find(|m| m.name == name)
        .map(|m| m.description.as_str())
        .ok_or_else(|| {
            UserError::UnknownMonitor {
                name: name.to_string(),
            }
            .into()
        })
}

pub fn name_of<'a>(monitors: &'a [Monitor], description: &str) -> Option<&'a str> {
    monitors
        .iter()
        .find(|m| m.description == description)
        .map(|m| m.name.as_str())
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub name: String,
    pub monitor: String,
}

impl Workspace {
    pub fn is_special(&self) -> bool {
        self.name.starts_with("special")
    }
}

pub trait CompositorCtl {
    /// Switched-off screens included; they still carry the description a profile is keyed on.
    fn monitors(&self) -> Result<Vec<Monitor>>;
    fn reload(&self) -> Result<()>;
    fn workspaces(&self) -> Result<Vec<Workspace>>;
    fn move_workspace_to_monitor(&self, workspace: &str, monitor: &str) -> Result<()>;
    fn render_rules(&self, layout: &Layout, monitors: &[Monitor]) -> String;
}
