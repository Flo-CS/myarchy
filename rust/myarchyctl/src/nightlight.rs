use anyhow::Result;
use clap::Subcommand;
use myarchy_core::nightlight::NightLightCtl;

use crate::backend;

#[derive(Subcommand)]
pub enum Command {
    Get,
    Set { percent: i64 },
    Off,
}

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Get => {
            println!("{}", backend::nightlightctl().get());
            Ok(())
        }
        Command::Set { percent } => backend::nightlightctl().set(percent),
        Command::Off => backend::nightlightctl().unset(),
    }
}
