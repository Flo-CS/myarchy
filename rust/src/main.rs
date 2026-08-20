mod display;
mod error;
mod models;
mod ports;
mod screen;

use anyhow::Result;
use clap::{Parser, Subcommand};
use error::AppError;
use ports::notifierctl::Notifier;

#[derive(Parser)]
#[command(name = "myarchyctl")]
struct Cli {
    #[command(subcommand)]
    command: TopCommand,
}

#[derive(Subcommand)]
enum TopCommand {
    #[command(subcommand)]
    Display(DisplayCommand),
    #[command(subcommand)]
    Screen(ScreenCommand),
}

#[derive(Subcommand)]
enum DisplayCommand {
    List,
    ListModes {
        name: String,
    },
    Extend {
        direction: String,
    },
    Place {
        name: String,
        side: String,
        r#ref: String,
    },
    Only {
        name: String,
    },
    Enable {
        name: String,
    },
    Disable {
        name: String,
    },
    Toggle {
        name: String,
    },
    SetMode {
        name: String,
        mode: String,
    },
    SetScale {
        name: String,
        scale: String,
    },
    Primary {
        name: String,
    },
    Anchor,
    Save,
    Apply,
    Auto,
    Reset,
}

#[derive(Subcommand)]
pub(crate) enum ScreenCommand {
    BrightnessGet {
        name: String,
    },
    BrightnessSet {
        name: String,
        percent: i64,
    },
    BrightnessStep {
        #[arg(allow_hyphen_values = true)]
        delta_percent: String,
        name: String,
    },
    BrightnessMonitors,
    NightlightGet,
    NightlightSet {
        percent: i64,
    },
    NightlightOff,
    #[command(hide = true)]
    BrightnessWorker {
        name: String,
    },
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        TopCommand::Display(cmd) => run_display(cmd),
        TopCommand::Screen(cmd) => run_screen(cmd),
    }
}

fn run_display(command: DisplayCommand) -> Result<()> {
    let compositor = ports::compositorctl::adapter();
    match command {
        DisplayCommand::List => {
            let out = display::list(&compositor)?;
            if !out.is_empty() {
                println!("{out}");
            }
            Ok(())
        }
        DisplayCommand::ListModes { name } => {
            let out = display::list_modes(&compositor, &name)?;
            if !out.is_empty() {
                println!("{out}");
            }
            Ok(())
        }
        DisplayCommand::Extend { direction } => display::extend(&compositor, &direction),
        DisplayCommand::Place { name, side, r#ref } => {
            display::place(&compositor, &name, &side, &r#ref)
        }
        DisplayCommand::Only { name } => display::only(&compositor, &name),
        DisplayCommand::Enable { name } => display::enable_monitor(&compositor, &name),
        DisplayCommand::Disable { name } => display::disable_monitor(&compositor, &name),
        DisplayCommand::Toggle { name } => display::toggle_monitor(&compositor, &name),
        DisplayCommand::SetMode { name, mode } => display::set_mode(&compositor, &name, &mode),
        DisplayCommand::SetScale { name, scale } => display::set_scale(&compositor, &name, &scale),
        DisplayCommand::Primary { name } => display::set_primary(&compositor, &name),
        DisplayCommand::Anchor => {
            if let Some(name) = display::anchor(&compositor)? {
                println!("{name}");
            }
            Ok(())
        }
        DisplayCommand::Save => display::save(&compositor),
        DisplayCommand::Apply => display::apply(&compositor),
        DisplayCommand::Auto => display::auto(&compositor, &ports::notifierctl::adapter()),
        DisplayCommand::Reset => display::reset(&compositor),
    }
}

fn run_screen(command: ScreenCommand) -> Result<()> {
    let compositor = ports::compositorctl::adapter();
    match command {
        ScreenCommand::BrightnessGet { name } => {
            if let Some(pct) = screen::brightness_get(&compositor, &name)? {
                println!("{pct}");
            }
            Ok(())
        }
        ScreenCommand::BrightnessSet { name, percent } => {
            screen::brightness_set(&compositor, &name, percent)
        }
        ScreenCommand::BrightnessStep {
            delta_percent,
            name,
        } => screen::brightness_step(
            &compositor,
            &ports::osdctl::adapter(),
            &delta_percent,
            &name,
        ),
        ScreenCommand::BrightnessMonitors => {
            for name in screen::brightness_monitors(&compositor)? {
                println!("{name}");
            }
            Ok(())
        }
        ScreenCommand::NightlightGet => {
            println!(
                "{}",
                screen::nightlight_get(&ports::nightlightctl::adapter())
            );
            Ok(())
        }
        ScreenCommand::NightlightSet { percent } => {
            screen::nightlight_set(&ports::nightlightctl::adapter(), percent)
        }
        ScreenCommand::NightlightOff => screen::nightlight_off(&ports::nightlightctl::adapter()),
        ScreenCommand::BrightnessWorker { name } => {
            screen::brightness_worker(&compositor, &ports::notifierctl::adapter(), &name)
        }
    }
}

fn main() {
    if let Err(err) = run() {
        let should_notify = err
            .downcast_ref::<AppError>()
            .is_some_and(AppError::should_notify);
        if should_notify {
            let _ = ports::notifierctl::adapter().send(
                "myarchyctl",
                &err.to_string(),
                "dialog-error",
                None,
            );
        }
        eprintln!("{err}");
        std::process::exit(1);
    }
}
