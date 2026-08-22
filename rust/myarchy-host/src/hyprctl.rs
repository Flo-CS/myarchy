use std::fmt::Write as _;

use anyhow::{bail, Context, Result};
use myarchy_core::compositor::{CompositorCtl, Monitor, Workspace};
use myarchy_core::layout::{Layout, Screen, State};
use myarchy_core::resolution::Resolution;
use serde::Deserialize;

use crate::proc;

const NO_MIRROR: &str = "none";

#[derive(Debug, Deserialize)]
struct HyprctlMonitor {
    name: String,
    description: Option<String>,
    width: i64,
    height: i64,
    #[serde(rename = "refreshRate")]
    refresh_rate: f64,
    x: i64,
    y: i64,
    scale: f64,
    #[serde(default)]
    disabled: bool,
    #[serde(default)]
    focused: bool,
    #[serde(rename = "mirrorOf", default)]
    mirror_of: Option<String>,
    #[serde(rename = "availableModes", default)]
    available_modes: Vec<String>,
}

impl From<HyprctlMonitor> for Monitor {
    fn from(m: HyprctlMonitor) -> Self {
        Monitor {
            name: m.name,
            description: m.description,
            resolution: Resolution::new(m.width, m.height, m.refresh_rate),
            x: m.x,
            y: m.y,
            scale: m.scale,
            disabled: m.disabled,
            focused: m.focused,
            mirror_of: m.mirror_of.filter(|s| s != NO_MIRROR && !s.is_empty()),
            resolutions: m
                .available_modes
                .iter()
                .filter_map(|mode| mode.parse().ok())
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct HyprctlWorkspace {
    name: String,
    monitor: String,
}

impl From<HyprctlWorkspace> for Workspace {
    fn from(w: HyprctlWorkspace) -> Self {
        Workspace {
            name: w.name,
            monitor: w.monitor,
        }
    }
}

pub struct HyprctlCli;

/// A missing compositor is reported on stdout with a zero exit status, not as a failure.
fn hyprctl(args: &[&str]) -> Result<String> {
    let out = proc::output("hyprctl", args)?;
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    if !out.status.success() || stdout.starts_with("Couldn't open a socket") {
        bail!("hyprctl {} failed: {}", args.join(" "), proc::detail(&out));
    }
    Ok(stdout)
}

fn lua_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c == '\\' || c == '"' {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

impl CompositorCtl for HyprctlCli {
    fn monitors(&self, all: bool) -> Result<Vec<Monitor>> {
        let args: &[&str] = if all {
            &["monitors", "all", "-j"]
        } else {
            &["monitors", "-j"]
        };
        let raw: Vec<HyprctlMonitor> = serde_json::from_str(&hyprctl(args)?)
            .context("failed to parse hyprctl monitors output")?;
        Ok(raw.into_iter().map(Monitor::from).collect())
    }

    fn reload(&self) -> Result<()> {
        hyprctl(&["reload"])?;
        Ok(())
    }

    fn workspaces(&self) -> Result<Vec<Workspace>> {
        let raw: Vec<HyprctlWorkspace> = serde_json::from_str(&hyprctl(&["workspaces", "-j"])?)
            .context("failed to parse hyprctl workspaces output")?;
        Ok(raw.into_iter().map(Workspace::from).collect())
    }

    fn move_workspace_to_monitor(&self, workspace: &str, monitor: &str) -> Result<()> {
        hyprctl(&["dispatch", "moveworkspacetomonitor", workspace, monitor])?;
        Ok(())
    }

    fn render_rules(&self, layout: &Layout, monitors: &[Monitor]) -> String {
        let name_of = |desc: &str| {
            monitors
                .iter()
                .find(|m| m.description() == desc)
                .map(|m| m.name.as_str())
                .unwrap_or(NO_MIRROR)
        };

        let mut out = String::new();
        for (desc, Screen { state, placement }) in &layout.screens {
            let output = lua_string(desc);
            match state {
                State::Off => {
                    let _ = writeln!(
                        out,
                        "hl.monitor({{ output = \"desc:{output}\", disabled = true }})"
                    );
                }
                State::On => {
                    let _ = writeln!(
                        out,
                        "hl.monitor({{ output = \"desc:{output}\", mode = \"{}\", position = \"{}\", scale = \"{}\", mirror = \"{NO_MIRROR}\", disabled = false }})",
                        placement.mode, placement.position, placement.scale
                    );
                }
                State::Mirroring(target) => {
                    let _ = writeln!(
                        out,
                        "hl.monitor({{ output = \"desc:{output}\", mode = \"{}\", scale = \"{}\", mirror = \"{}\", disabled = false }})",
                        placement.mode,
                        placement.scale,
                        lua_string(name_of(target)),
                    );
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use myarchy_core::layout::fixtures::{laptop, ultrawide, BOE, DELL, LG};
    use myarchy_core::layout::{Placement, Screen};

    fn screen(state: State, mode: &str, position: &str, scale: &str) -> Screen {
        Screen {
            state,
            placement: Placement {
                mode: mode.parse().unwrap(),
                position: position.parse().unwrap(),
                scale: scale.parse().unwrap(),
            },
        }
    }

    #[test]
    fn renders_a_profile_spelling_out_both_sticky_fields_on_every_rule() {
        let mut layout = Layout::default();
        layout
            .screens
            .insert(LG.into(), screen(State::On, "3440x1440@99.997", "0x0", "1"));
        layout.screens.insert(
            BOE.into(),
            screen(
                State::Mirroring(LG.into()),
                "1920x1080@60.003",
                "0x0",
                "1.25",
            ),
        );
        layout.screens.insert(
            DELL.into(),
            screen(State::Off, "1920x1200@59.95", "5360x0", "1"),
        );

        assert_eq!(
            HyprctlCli.render_rules(&layout, &[laptop(), ultrawide()]),
            concat!(
                "hl.monitor({ output = \"desc:BOE 0x08B9\", mode = \"1920x1080@60.003\", scale = \"1.25\", mirror = \"DP-3\", disabled = false })\n",
                "hl.monitor({ output = \"desc:Dell Inc. DELL U2412M PMSXXXX\", disabled = true })\n",
                "hl.monitor({ output = \"desc:LG Electronics LG HDR WQHD 303NTZN51357\", mode = \"3440x1440@99.997\", position = \"0x0\", scale = \"1\", mirror = \"none\", disabled = false })\n",
            )
        );
    }

    #[test]
    fn a_mirror_of_an_unplugged_screen_falls_back_to_no_mirror() {
        let mut layout = Layout::default();
        layout.screens.insert(
            BOE.into(),
            screen(
                State::Mirroring(DELL.into()),
                "1920x1080@60.003",
                "0x0",
                "1",
            ),
        );

        assert!(HyprctlCli
            .render_rules(&layout, &[laptop()])
            .contains("mirror = \"none\""));
    }

    #[test]
    fn a_quote_in_a_description_cannot_break_the_generated_lua() {
        let mut layout = Layout::default();
        layout.screens.insert(
            "Acme 24\" \\ Pro".into(),
            screen(State::Off, "1920x1080@60", "0x0", "1"),
        );

        assert_eq!(
            HyprctlCli.render_rules(&layout, &[]),
            "hl.monitor({ output = \"desc:Acme 24\\\" \\\\ Pro\", disabled = true })\n"
        );
    }
}
