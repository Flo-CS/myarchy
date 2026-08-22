use std::ffi::OsStr;

use anyhow::{Context, Result};
use clap::Subcommand;
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use myarchy_core::cursor::CursorCtl;
use myarchy_core::error::AppError;
use myarchy_sys::state::cursor as state;
use myarchy_sys::theme;

use crate::backend;
use crate::cli::candidates;

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

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::List => {
            for name in state::list()? {
                println!("{name}");
            }
            Ok(())
        }
        Command::ApplyPreferred => apply_preferred(),
        Command::Set { name, size } => set(&name, size),
        Command::SavePreferred { name, size } => save_preferred(&name, size),
        Command::Reset => reset(),
    }
}

fn set(name: &str, size: i64) -> Result<()> {
    let applied = backend::cursorctl().set(name, size);
    save_preferred(name, size)?;
    applied
}

fn apply_preferred() -> Result<()> {
    let (name, size) = match state::load_preferred() {
        Some(state) => (state.name, state.size),
        None => {
            let name = theme::get_var("cursor-name")?.ok_or(AppError::NoPreferredCursor)?;
            let size = theme::get_var("cursor-size")?
                .context("no cursor-size found for current theme")?
                .parse::<i64>()
                .context("cursor-size in current theme is not a number")?;
            (name, size)
        }
    };
    backend::cursorctl().set(&name, size)
}

fn save_preferred(name: &str, size: i64) -> Result<()> {
    state::save_preferred(name, size)?;
    theme::render()
}

fn reset() -> Result<()> {
    state::forget_preferred();
    apply_preferred()?;
    theme::render()
}

fn complete_name(current: &OsStr) -> Vec<CompletionCandidate> {
    candidates(state::list().unwrap_or_default(), current)
}
