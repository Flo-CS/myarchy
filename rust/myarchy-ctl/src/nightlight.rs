use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;
use myarchy_core::core::nightlight::NightlightCtl;
use myarchy_core::core::percent::Percent;

use crate::host::Host;

#[derive(Subcommand)]
pub enum Command {
    Get,
    Set { percent: Percent },
    Off,
}

pub fn run(command: Command, host: &Host) -> Result<ExitCode> {
    let nightlight = host.nightlight();
    match command {
        Command::Get => println!("{}", nightlight.get()),
        Command::Set { percent } => nightlight.set(percent)?,
        Command::Off => nightlight.unset()?,
    }
    Ok(ExitCode::SUCCESS)
}
