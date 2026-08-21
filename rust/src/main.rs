mod backend;
mod brightness;
mod cursor;
mod display;
mod error;
mod icon;
mod models;
mod nightlight;
mod wallpaper;

use std::ffi::OsStr;

use anyhow::Result;
use backend::compositor::Compositorctl;
use backend::notifier::Notifierctl;
use clap::{Parser, Subcommand};
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use error::AppError;
use icon::Icon;
use models::layout::{Direction, Side};

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
    Brightness(BrightnessCommand),
    #[command(subcommand)]
    Nightlight(NightlightCommand),
    #[command(subcommand)]
    Wallpaper(WallpaperCommand),
    #[command(subcommand)]
    Cursor(CursorCommand),
}

#[derive(Subcommand)]
enum DisplayCommand {
    List,
    ListModes {
        #[arg(add = ArgValueCompleter::new(complete_monitor_name))]
        name: String,
    },
    Extend {
        direction: Direction,
    },
    Place {
        #[arg(add = ArgValueCompleter::new(complete_monitor_name))]
        name: String,
        side: Side,
        #[arg(add = ArgValueCompleter::new(complete_monitor_name))]
        r#ref: String,
    },
    Mirror,
    Only {
        #[arg(add = ArgValueCompleter::new(complete_monitor_name))]
        name: String,
    },
    Enable {
        #[arg(add = ArgValueCompleter::new(complete_monitor_name))]
        name: String,
    },
    Disable {
        #[arg(add = ArgValueCompleter::new(complete_monitor_name))]
        name: String,
    },
    Toggle {
        #[arg(add = ArgValueCompleter::new(complete_monitor_name))]
        name: String,
    },
    SetMode {
        #[arg(add = ArgValueCompleter::new(complete_monitor_name))]
        name: String,
        #[arg(add = ArgValueCompleter::new(complete_display_mode))]
        mode: String,
    },
    SetScale {
        #[arg(add = ArgValueCompleter::new(complete_monitor_name))]
        name: String,
        scale: String,
    },
    Primary {
        #[arg(add = ArgValueCompleter::new(complete_monitor_name))]
        name: String,
    },
    Anchor,
    Save,
    Apply,
    Auto,
    Reset,
}

#[derive(Subcommand)]
enum BrightnessCommand {
    Get {
        #[arg(add = ArgValueCompleter::new(complete_brightness_monitor))]
        name: String,
    },
    Set {
        #[arg(add = ArgValueCompleter::new(complete_brightness_monitor))]
        name: String,
        percent: i64,
    },
    Step {
        #[arg(allow_negative_numbers = true)]
        delta_percent: i64,
        #[arg(add = ArgValueCompleter::new(complete_brightness_monitor))]
        name: String,
    },
    Monitors,
}

#[derive(Subcommand)]
enum NightlightCommand {
    Get,
    Set { percent: i64 },
    Off,
}

#[derive(Subcommand)]
enum WallpaperCommand {
    Init,
    Get,
    List,
    Dir,
    Set {
        #[arg(add = ArgValueCompleter::new(complete_wallpaper_name))]
        name: String,
    },
    ApplyPreferred,
    SavePreferred,
    Reset,
}

#[derive(Subcommand)]
enum CursorCommand {
    List,
    ApplyPreferred,
    Set {
        #[arg(add = ArgValueCompleter::new(complete_cursor_name))]
        name: String,
        size: i64,
    },
    SavePreferred {
        #[arg(add = ArgValueCompleter::new(complete_cursor_name))]
        name: String,
        size: i64,
    },
    Reset,
}

fn candidates(names: Vec<String>, current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };
    names
        .into_iter()
        .filter(|name| name.starts_with(current))
        .map(CompletionCandidate::new)
        .collect()
}

fn complete_monitor_name(current: &OsStr) -> Vec<CompletionCandidate> {
    let names = backend::compositor::backend()
        .described_monitors(true)
        .map(|monitors| monitors.into_iter().map(|m| m.name).collect())
        .unwrap_or_default();
    candidates(names, current)
}

fn complete_display_mode(current: &OsStr) -> Vec<CompletionCandidate> {
    let mut modes: Vec<String> = backend::compositor::backend()
        .described_monitors(true)
        .map(|monitors| {
            monitors
                .into_iter()
                .flat_map(|m| m.resolutions)
                .map(|resolution| resolution.to_string())
                .collect()
        })
        .unwrap_or_default();
    modes.sort();
    modes.dedup();
    candidates(modes, current)
}

fn complete_brightness_monitor(current: &OsStr) -> Vec<CompletionCandidate> {
    let names = brightness::monitors(&backend::compositor::backend()).unwrap_or_default();
    candidates(names, current)
}

fn complete_wallpaper_name(current: &OsStr) -> Vec<CompletionCandidate> {
    candidates(wallpaper::list().unwrap_or_default(), current)
}

fn complete_cursor_name(current: &OsStr) -> Vec<CompletionCandidate> {
    candidates(cursor::list().unwrap_or_default(), current)
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        TopCommand::Display(cmd) => run_display(cmd),
        TopCommand::Brightness(cmd) => run_brightness(cmd),
        TopCommand::Nightlight(cmd) => run_nightlight(cmd),
        TopCommand::Wallpaper(cmd) => run_wallpaper(cmd),
        TopCommand::Cursor(cmd) => run_cursor(cmd),
    }
}

fn run_display(command: DisplayCommand) -> Result<()> {
    let compositor = backend::compositor::backend();
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
        DisplayCommand::Extend { direction } => display::extend(&compositor, direction),
        DisplayCommand::Place { name, side, r#ref } => {
            display::place(&compositor, &name, side, &r#ref)
        }
        DisplayCommand::Mirror => display::mirror(&compositor),
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
        DisplayCommand::Auto => display::auto(&compositor, &backend::notifier::backend()),
        DisplayCommand::Reset => display::reset(&compositor),
    }
}

fn run_brightness(command: BrightnessCommand) -> Result<()> {
    let compositor = backend::compositor::backend();
    match command {
        BrightnessCommand::Get { name } => {
            if let Some(pct) = brightness::get(&compositor, &name)? {
                println!("{pct}");
            }
            Ok(())
        }
        BrightnessCommand::Set { name, percent } => brightness::set(&compositor, &name, percent),
        BrightnessCommand::Step {
            delta_percent,
            name,
        } => brightness::step(&compositor, &backend::osd::backend(), delta_percent, &name),
        BrightnessCommand::Monitors => {
            for name in brightness::monitors(&compositor)? {
                println!("{name}");
            }
            Ok(())
        }
    }
}

fn run_nightlight(command: NightlightCommand) -> Result<()> {
    let nightlight = backend::nightlight::backend();
    match command {
        NightlightCommand::Get => {
            println!("{}", nightlight::get(&nightlight));
            Ok(())
        }
        NightlightCommand::Set { percent } => nightlight::set(&nightlight, percent),
        NightlightCommand::Off => nightlight::off(&nightlight),
    }
}

fn run_wallpaper(command: WallpaperCommand) -> Result<()> {
    let adapter = backend::wallpaper::backend();
    match command {
        WallpaperCommand::Init => wallpaper::init(&adapter),
        WallpaperCommand::Get => {
            if let Some(name) = wallpaper::get()? {
                println!("{name}");
            }
            Ok(())
        }
        WallpaperCommand::List => {
            for name in wallpaper::list()? {
                println!("{name}");
            }
            Ok(())
        }
        WallpaperCommand::Dir => {
            println!("{}", wallpaper::dir().display());
            Ok(())
        }
        WallpaperCommand::Set { name } => wallpaper::set(&adapter, &name),
        WallpaperCommand::ApplyPreferred => wallpaper::apply_preferred(&adapter),
        WallpaperCommand::SavePreferred => wallpaper::save_preferred(),
        WallpaperCommand::Reset => wallpaper::reset(&adapter),
    }
}

fn run_cursor(command: CursorCommand) -> Result<()> {
    let adapter = backend::cursor::backend();
    match command {
        CursorCommand::List => {
            for name in cursor::list()? {
                println!("{name}");
            }
            Ok(())
        }
        CursorCommand::ApplyPreferred => cursor::apply_preferred(&adapter),
        CursorCommand::Set { name, size } => {
            cursor::set(&adapter, &name, size)?;
            Ok(())
        }
        CursorCommand::SavePreferred { name, size } => {
            cursor::save_preferred(&name, size)?;
            Ok(())
        }
        CursorCommand::Reset => cursor::reset(&adapter),
    }
}

fn main() {
    clap_complete::CompleteEnv::with_factory(<Cli as clap::CommandFactory>::command).complete();

    if let Err(err) = run() {
        let should_notify = err
            .downcast_ref::<AppError>()
            .is_some_and(AppError::should_notify);
        if should_notify {
            let _ = backend::notifier::backend().send(
                "myarchyctl",
                &err.to_string(),
                Icon::DialogError.as_str(),
                None,
            );
        }
        eprintln!("{err}");
        std::process::exit(1);
    }
}
