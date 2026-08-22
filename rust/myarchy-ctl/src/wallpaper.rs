use std::ffi::OsStr;
use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};

use crate::cli::candidates;
use crate::host::Host;

#[derive(Subcommand)]
pub enum Command {
    Init,
    Get,
    List,
    Dir,
    Set {
        #[arg(add = ArgValueCompleter::new(complete_name))]
        name: String,
    },
    ApplyPreferred,
    SavePreferred,
    Reset,
}

pub fn run(command: Command, host: &Host) -> Result<ExitCode> {
    let wallpaper = host.wallpaper();
    match command {
        Command::Init => wallpaper.init()?,
        Command::Get => {
            if let Some(name) = wallpaper.current() {
                println!("{name}");
            }
        }
        Command::List => {
            for name in wallpaper.list()? {
                println!("{name}");
            }
        }
        Command::Dir => println!("{}", wallpaper.dir().display()),
        Command::Set { name } => wallpaper.set(&name)?,
        Command::ApplyPreferred => wallpaper.apply_preferred()?,
        Command::SavePreferred => wallpaper.save_preferred()?,
        Command::Reset => wallpaper.reset()?,
    }
    Ok(ExitCode::SUCCESS)
}

fn complete_name(current: &OsStr) -> Vec<CompletionCandidate> {
    let names = Host::new(false, false)
        .wallpaper()
        .list()
        .unwrap_or_default();
    candidates(names, current)
}
