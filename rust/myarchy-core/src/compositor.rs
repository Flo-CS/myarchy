use anyhow::Result;

use crate::layout::Layout;
use crate::resolution::Resolution;

#[derive(Debug, Clone)]
pub struct Monitor {
    pub name: String,
    pub description: Option<String>,
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
    pub fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("")
    }

    pub fn is_internal(&self) -> bool {
        self.name.starts_with("eDP-")
            || self.name.starts_with("LVDS-")
            || self.name.starts_with("DSI-")
    }
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
    fn monitors(&self, all: bool) -> Result<Vec<Monitor>>;
    fn described_monitors(&self, all: bool) -> Result<Vec<Monitor>> {
        Ok(self
            .monitors(all)?
            .into_iter()
            .filter(|m| !m.description().is_empty())
            .collect())
    }

    fn reload(&self) -> Result<()>;
    fn workspaces(&self) -> Result<Vec<Workspace>>;
    fn move_workspace_to_monitor(&self, workspace: &str, monitor: &str) -> Result<()>;
    fn render_rules(&self, layout: &Layout, monitors: &[Monitor]) -> String;
}
