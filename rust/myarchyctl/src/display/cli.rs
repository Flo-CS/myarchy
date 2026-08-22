use std::ffi::OsStr;

use anyhow::Result;
use clap::Subcommand;
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use myarchy_core::compositor::CompositorCtl;
use myarchy_core::layout::{Direction, Side};

use crate::backend;
use crate::cli::candidates;
use crate::display;

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

pub fn run(command: Command) -> Result<()> {
    let compositor = backend::compositorctl();
    match command {
        Command::List => {
            let out = display::list(&compositor)?;
            if !out.is_empty() {
                println!("{out}");
            }
            Ok(())
        }
        Command::ListModes { name } => {
            let out = display::list_modes(&compositor, &name)?;
            if !out.is_empty() {
                println!("{out}");
            }
            Ok(())
        }
        Command::Extend { direction } => display::extend(&compositor, direction),
        Command::Place { name, side, r#ref } => display::place(&compositor, &name, side, &r#ref),
        Command::Mirror => display::mirror(&compositor),
        Command::Only { name } => display::only(&compositor, &name),
        Command::Enable { name } => display::enable_monitor(&compositor, &name),
        Command::Disable { name } => display::disable_monitor(&compositor, &name),
        Command::Toggle { name } => display::toggle_monitor(&compositor, &name),
        Command::SetMode { name, mode } => display::set_mode(&compositor, &name, &mode),
        Command::SetScale { name, scale } => display::set_scale(&compositor, &name, &scale),
        Command::Primary { name } => display::set_primary(&compositor, &name),
        Command::Anchor => {
            if let Some(name) = display::anchor(&compositor)? {
                println!("{name}");
            }
            Ok(())
        }
        Command::Save => display::save(&compositor),
        Command::Apply => display::apply(&compositor),
        Command::Auto => display::auto(&compositor, &backend::notifierctl()),
        Command::Reset => display::reset(&compositor),
    }
}

fn complete_monitor(current: &OsStr) -> Vec<CompletionCandidate> {
    let names = backend::compositorctl()
        .described_monitors(true)
        .map(|monitors| monitors.into_iter().map(|m| m.name).collect())
        .unwrap_or_default();
    candidates(names, current)
}

fn complete_mode(current: &OsStr) -> Vec<CompletionCandidate> {
    let mut modes: Vec<String> = backend::compositorctl()
        .described_monitors(true)
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
