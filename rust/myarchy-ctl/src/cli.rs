use std::ffi::OsStr;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_complete::engine::CompletionCandidate;
use myarchy_core::core::notify::Icon;

use crate::host::Host;
use crate::{brightness, cursor, display, idle, nightlight, wallpaper};

#[derive(Parser)]
#[command(name = "myarchy-ctl")]
pub struct Cli {
    /// Report a failure as a desktop notification
    #[arg(long, global = true)]
    pub notify: bool,
    /// Show the value a command lands on as an on-screen display
    #[arg(long, global = true)]
    pub osd: bool,
    #[command(subcommand)]
    pub command: TopCommand,
}

#[derive(Subcommand)]
pub enum TopCommand {
    #[command(subcommand)]
    Display(display::Command),
    #[command(subcommand)]
    Brightness(brightness::Command),
    #[command(subcommand)]
    Nightlight(nightlight::Command),
    #[command(subcommand)]
    Wallpaper(wallpaper::Command),
    #[command(subcommand)]
    Cursor(cursor::Command),
    #[command(subcommand)]
    Idle(idle::Command),
}

impl TopCommand {
    /// What a `--notify` toast is headed, once the wording underneath it is already settled.
    pub fn subject(&self) -> (&'static str, Icon) {
        match self {
            Self::Display(_) => ("Display", Icon::VideoDisplay),
            Self::Brightness(_) => ("Brightness", Icon::BrightnessSymbolic),
            Self::Nightlight(_) => ("Night light", Icon::DialogError),
            Self::Wallpaper(_) => ("Wallpaper", Icon::DialogError),
            Self::Cursor(_) => ("Cursor", Icon::DialogError),
            Self::Idle(_) => ("Idle inhibitor", Icon::DialogError),
        }
    }

    pub fn run(self, host: &Host) -> Result<ExitCode> {
        match self {
            Self::Display(cmd) => display::run(cmd, host),
            Self::Brightness(cmd) => brightness::run(cmd, host),
            Self::Nightlight(cmd) => nightlight::run(cmd, host),
            Self::Wallpaper(cmd) => wallpaper::run(cmd, host),
            Self::Cursor(cmd) => cursor::run(cmd, host),
            Self::Idle(cmd) => idle::run(cmd, host),
        }
    }
}

pub fn candidates(names: Vec<String>, current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };
    names
        .into_iter()
        .filter(|name| name.starts_with(current))
        .map(CompletionCandidate::new)
        .collect()
}
