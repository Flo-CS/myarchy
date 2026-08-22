use std::ffi::OsStr;
use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use myarchy_core::core::compositor::{CompositorCtl, Monitor};
use myarchy_core::core::layout::{Direction, Side};
use myarchy_core::host::hyprctl::Hyprctl;

use crate::cli::candidates;
use crate::host::Host;

#[derive(Subcommand)]
pub enum Command {
    List,
    ListModes {
        #[arg(add = ArgValueCompleter::new(complete_monitor))]
        name: String,
    },
    Extend {
        direction: Direction,
    },
    Place {
        #[arg(add = ArgValueCompleter::new(complete_monitor))]
        name: String,
        side: Side,
        #[arg(add = ArgValueCompleter::new(complete_monitor))]
        r#ref: String,
    },
    Mirror,
    Only {
        #[arg(add = ArgValueCompleter::new(complete_monitor))]
        name: String,
    },
    Enable {
        #[arg(add = ArgValueCompleter::new(complete_monitor))]
        name: String,
    },
    Disable {
        #[arg(add = ArgValueCompleter::new(complete_monitor))]
        name: String,
    },
    Toggle {
        #[arg(add = ArgValueCompleter::new(complete_monitor))]
        name: String,
    },
    SetMode {
        #[arg(add = ArgValueCompleter::new(complete_monitor))]
        name: String,
        #[arg(add = ArgValueCompleter::new(complete_mode))]
        mode: String,
    },
    SetScale {
        #[arg(add = ArgValueCompleter::new(complete_monitor))]
        name: String,
        scale: String,
    },
    Primary {
        #[arg(add = ArgValueCompleter::new(complete_monitor))]
        name: String,
    },
    Anchor,
    Save,
    Apply,
    Auto,
    Reset,
}

pub fn run(command: Command, host: &Host) -> Result<ExitCode> {
    let display = host.display();
    match command {
        Command::List => {
            for monitor in display.monitors()? {
                println!("{}", row(&monitor));
            }
        }
        Command::ListModes { name } => {
            for resolution in display.modes(&name)? {
                println!("{resolution}");
            }
        }
        Command::Extend { direction } => display.extend(direction)?,
        Command::Place { name, side, r#ref } => display.place(&name, side, &r#ref)?,
        Command::Mirror => display.mirror()?,
        Command::Only { name } => display.only(&name)?,
        Command::Enable { name } => display.enable(&name)?,
        Command::Disable { name } => display.disable(&name)?,
        Command::Toggle { name } => display.toggle(&name)?,
        Command::SetMode { name, mode } => display.set_mode(&name, &mode)?,
        Command::SetScale { name, scale } => display.set_scale(&name, &scale)?,
        Command::Primary { name } => display.set_primary(&name)?,
        Command::Anchor => {
            if let Some(name) = display.anchor()? {
                println!("{name}");
            }
        }
        Command::Save => display.save()?,
        Command::Apply => display.apply()?,
        Command::Auto => display.auto()?,
        Command::Reset => display.reset()?,
    }
    Ok(ExitCode::SUCCESS)
}

fn row(monitor: &Monitor) -> String {
    let state = if monitor.disabled {
        "disabled"
    } else if monitor.mirror_of.is_some() {
        "mirroring"
    } else {
        "enabled"
    };
    format!(
        "{}\t{}\t{}\t{state}",
        monitor.name,
        monitor.description,
        monitor.resolution.rounded()
    )
}

fn complete_monitor(current: &OsStr) -> Vec<CompletionCandidate> {
    let names = Hyprctl
        .monitors()
        .map(|monitors| monitors.into_iter().map(|m| m.name).collect())
        .unwrap_or_default();
    candidates(names, current)
}

fn complete_mode(current: &OsStr) -> Vec<CompletionCandidate> {
    let mut modes: Vec<String> = Hyprctl
        .monitors()
        .map(|monitors| {
            monitors
                .into_iter()
                .flat_map(|m| m.resolutions)
                .map(|resolution| resolution.to_string())
                .collect()
        })
        .unwrap_or_default();
    modes.sort();
    modes.dedup();
    candidates(modes, current)
}
