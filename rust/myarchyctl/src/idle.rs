use std::io::Write;

use anyhow::Result;
use clap::Subcommand;
use myarchy_core::idle::IdleCtl;

use crate::backend;

#[derive(Subcommand)]
pub enum Command {
    Enable,
    Disable,
    Status,
    Watch,
}

pub fn run(command: Command) -> Result<()> {
    let idle = backend::idlectl();
    match command {
        Command::Enable => idle.enable(),
        Command::Disable => idle.disable(),
        Command::Status => {
            let inhibited = idle.status()?;
            println!("{}", label(inhibited));
            if inhibited {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
        Command::Watch => {
            let mut stdout = std::io::stdout().lock();
            idle.watch(&mut |inhibited| {
                writeln!(stdout, "{}", label(inhibited))?;
                stdout.flush()?;
                Ok(())
            })
        }
    }
}

fn label(inhibited: bool) -> &'static str {
    if inhibited {
        "inhibited"
    } else {
        "not inhibited"
    }
}
