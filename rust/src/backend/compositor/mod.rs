mod hyprctl;

use anyhow::Result;

use crate::models::layout::Layout;
use crate::models::monitor::Monitor;
use crate::models::workspace::Workspace;

pub(crate) trait Compositorctl {
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

pub(crate) fn backend() -> impl Compositorctl {
    hyprctl::HyprctlCli
}
