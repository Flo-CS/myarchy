use std::ffi::OsStr;
use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};

use crate::cli::candidates;
use crate::host::Host;

#[derive(Subcommand)]
pub enum Command {
    List,
    ApplyPreferred,
    Set {
        #[arg(add = ArgValueCompleter::new(complete_name))]
        name: String,
        size: i64,
    },
    SavePreferred {
        #[arg(add = ArgValueCompleter::new(complete_name))]
        name: String,
        size: i64,
    },
    Reset,
}

pub fn run(command: Command, host: &Host) -> Result<ExitCode> {
    let cursor = host.cursor();
    match command {
        Command::List => {
            for name in cursor.list()? {
                println!("{name}");
            }
        }
        Command::ApplyPreferred => cursor.apply_preferred()?,
        Command::Set { name, size } => cursor.set(&name, size)?,
        Command::SavePreferred { name, size } => cursor.save_preferred(&name, size)?,
        Command::Reset => cursor.reset()?,
    }
    Ok(ExitCode::SUCCESS)
}

fn complete_name(current: &OsStr) -> Vec<CompletionCandidate> {
    let names = Host::new(false, false).cursor().list().unwrap_or_default();
    candidates(names, current)
}
