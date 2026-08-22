use std::io::Write;
use std::process::ExitCode;

use anyhow::Result;
use clap::Subcommand;
use myarchy_core::core::idle::IdleCtl;

use crate::host::Host;

#[derive(Subcommand)]
pub enum Command {
    Enable,
    Disable,
    Status,
    Watch,
}

pub fn run(command: Command, host: &Host) -> Result<ExitCode> {
    let idle = host.idle();
    match command {
        Command::Enable => idle.enable()?,
        Command::Disable => idle.disable()?,
        Command::Status => {
            let inhibited = idle.status()?;
            println!("{}", label(inhibited));
            if !inhibited {
                return Ok(ExitCode::FAILURE);
            }
        }
        Command::Watch => {
            let mut stdout = std::io::stdout().lock();
            idle.watch(&mut |inhibited| {
                writeln!(stdout, "{}", label(inhibited))?;
                stdout.flush()?;
                Ok(())
            })?;
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn label(inhibited: bool) -> &'static str {
    if inhibited {
        "inhibited"
    } else {
        "not inhibited"
    }
}
