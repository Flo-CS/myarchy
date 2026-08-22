use std::ffi::OsStr;
use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use myarchy_core::core::percent::Percent;

use crate::cli::candidates;
use crate::host::Host;

#[derive(Subcommand)]
pub enum Command {
    Get {
        #[arg(add = ArgValueCompleter::new(complete_monitor))]
        name: String,
        /// Return the last value set instead of querying the monitor
        #[arg(long)]
        dirty: bool,
    },
    Set {
        #[arg(add = ArgValueCompleter::new(complete_monitor))]
        name: String,
        percent: Percent,
        /// Fire the hardware write off in the background instead of waiting on the monitor to respond
        #[arg(long)]
        dirty: bool,
    },
    Step {
        #[arg(allow_negative_numbers = true)]
        delta_percent: i64,
        #[arg(add = ArgValueCompleter::new(complete_monitor))]
        name: String,
    },
    Monitors,
}

pub fn run(command: Command, host: &Host) -> Result<ExitCode> {
    let brightness = host.brightness();
    match command {
        Command::Get { name, dirty } => {
            if let Some(percent) = brightness.get(&name, dirty)? {
                println!("{percent}");
            }
        }
        Command::Set {
            name,
            percent,
            dirty,
        } => brightness.set(&name, percent, dirty)?,
        Command::Step {
            delta_percent,
            name,
        } => brightness.step(delta_percent, &name)?,
        Command::Monitors => {
            for name in brightness.monitors()? {
                println!("{name}");
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn complete_monitor(current: &OsStr) -> Vec<CompletionCandidate> {
    let names = Host::new(false, false)
        .brightness()
        .monitors()
        .unwrap_or_default();
    candidates(names, current)
}
