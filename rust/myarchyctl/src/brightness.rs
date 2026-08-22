use std::ffi::OsStr;

use anyhow::Result;
use clap::Subcommand;
use clap_complete::engine::CompletionCandidate;
use myarchy_core::compositor::CompositorCtl;
use myarchy_core::notify::{Icon, OsdCtl};

use crate::backend;
use crate::cli::candidates;

#[derive(Subcommand)]
pub enum Command {
    Get {
        #[arg(add = clap_complete::engine::ArgValueCompleter::new(complete_monitor))]
        name: String,
    },
    Set {
        #[arg(add = clap_complete::engine::ArgValueCompleter::new(complete_monitor))]
        name: String,
        percent: i64,
    },
    Step {
        #[arg(allow_negative_numbers = true)]
        delta_percent: i64,
        #[arg(add = clap_complete::engine::ArgValueCompleter::new(complete_monitor))]
        name: String,
    },
    Monitors,
}

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Get { name } => {
            if let Some(pct) = get(&name)? {
                println!("{pct}");
            }
            Ok(())
        }
        Command::Set { name, percent } => set(&name, percent),
        Command::Step {
            delta_percent,
            name,
        } => step(delta_percent, &name),
        Command::Monitors => {
            for name in monitors()? {
                println!("{name}");
            }
            Ok(())
        }
    }
}

pub fn get(name: &str) -> Result<Option<i64>> {
    let monitors = backend::compositorctl().monitors(false)?;
    let Some(backend) = backend::try_brightnessctl(name, &monitors)? else {
        return Ok(None);
    };
    Ok(backend.get().ok())
}

pub fn set(name: &str, percent: i64) -> Result<()> {
    let monitors = backend::compositorctl().monitors(false)?;
    backend::brightnessctl(name, &monitors)?.set(percent)
}

pub fn step(delta_percent: i64, name: &str) -> Result<()> {
    let monitors = backend::compositorctl().monitors(false)?;
    let backend = backend::brightnessctl(name, &monitors)?;
    let percent = backend.step(delta_percent)?;
    backend::osdctl().show_custom_progress(
        name,
        percent as f64 / 100.0,
        Icon::BrightnessSymbolic.as_str(),
    )?;
    backend.settle()
}

pub fn monitors() -> Result<Vec<String>> {
    let monitors = backend::compositorctl().monitors(false)?;
    let mut names = Vec::new();
    for m in &monitors {
        if backend::try_brightnessctl(&m.name, &monitors)?.is_some() {
            names.push(m.name.clone());
        }
    }
    Ok(names)
}

fn complete_monitor(current: &OsStr) -> Vec<CompletionCandidate> {
    candidates(monitors().unwrap_or_default(), current)
}
