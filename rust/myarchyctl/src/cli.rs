use std::ffi::OsStr;

use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_complete::engine::CompletionCandidate;

use crate::{brightness, cursor, display, idle, nightlight, wallpaper};

#[derive(Parser)]
#[command(name = "myarchyctl")]
pub struct Cli {
    #[command(subcommand)]
    command: TopCommand,
}

#[derive(Subcommand)]
enum TopCommand {
    #[command(subcommand)]
    Display(display::cli::Command),
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

impl Cli {
    pub fn run(self) -> Result<()> {
        match self.command {
            TopCommand::Display(cmd) => display::cli::run(cmd),
            TopCommand::Brightness(cmd) => brightness::run(cmd),
            TopCommand::Nightlight(cmd) => nightlight::run(cmd),
            TopCommand::Wallpaper(cmd) => wallpaper::run(cmd),
            TopCommand::Cursor(cmd) => cursor::run(cmd),
            TopCommand::Idle(cmd) => idle::run(cmd),
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
