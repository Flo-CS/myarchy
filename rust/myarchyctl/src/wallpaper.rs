use std::ffi::OsStr;

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use myarchy_core::error::AppError;
use myarchy_core::wallpaper::WallpaperCtl;
use myarchy_host::store::wallpaper as state;
use myarchy_host::theme;

use crate::backend;
use crate::cli::candidates;

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

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Init => init(),
        Command::Get => {
            if let Some(name) = state::current() {
                println!("{name}");
            }
            Ok(())
        }
        Command::List => {
            for name in state::list()? {
                println!("{name}");
            }
            Ok(())
        }
        Command::Dir => {
            println!("{}", state::dir().display());
            Ok(())
        }
        Command::Set { name } => set(&name),
        Command::ApplyPreferred => apply_preferred(),
        Command::SavePreferred => save_preferred(),
        Command::Reset => reset(),
    }
}

fn apply(name: &str) -> Result<()> {
    if !state::dir().join(name).is_file() {
        bail!(AppError::UnknownWallpaper {
            name: name.to_string()
        });
    }
    let path = state::point_current_at(name)?;

    // hyprpaper.conf preloads this symlink on its own startup, before myarchyctl ever runs; the
    // path passed below only tells an already-running hyprpaper to switch live over IPC.
    backend::wallpaperctl().apply(&path)
}

fn set(name: &str) -> Result<()> {
    apply(name)?;
    save_preferred()
}

fn init() -> Result<()> {
    if state::current_link().is_file() {
        return Ok(());
    }
    let name = state::list()?
        .into_iter()
        .next()
        .context("no wallpapers found")?;
    set(&name)
}

fn apply_preferred() -> Result<()> {
    let name = match state::load_preferred() {
        Some(name) => name,
        None => theme::get_var("wallpaper")?.ok_or(AppError::NoPreferredWallpaper)?,
    };
    apply(&name)
}

fn save_preferred() -> Result<()> {
    let name = state::current().context("no current wallpaper set")?;
    state::save_preferred(&name)?;
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
