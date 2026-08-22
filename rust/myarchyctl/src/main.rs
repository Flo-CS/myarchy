mod backend;
pub mod desktop;
mod display;
mod error;

use std::ffi::OsStr;

use anyhow::Result;
use backend::icon::Icon;
use clap::{Parser, Subcommand};
use clap_complete::engine::{ArgValueCompleter, CompletionCandidate};
use desktop::idle;
use display::layout::{Direction, Side};
use display::{CompositorCtl, NotifierCtl};
use error::AppError;

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
    #[command(subcommand)]
    Idle(IdleCommand),
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

#[derive(Subcommand)]
enum IdleCommand {
    Enable,
    Disable,
    Status,
    Watch,
}

fn main() {
    clap_complete::CompleteEnv::with_factory(<Cli as clap::CommandFactory>::command).complete();

    if let Err(err) = run() {
        let should_notify = err
            .downcast_ref::<AppError>()
            .is_some_and(AppError::should_notify);
        if should_notify {
            let _ = backend::notifierctl().send(
                "myarchyctl", // TODO: should depend on the error
                &err.to_string(),
                Icon::DialogError.as_str(), // TODO: should depend on the error
                None,
            );
        }
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        TopCommand::Display(cmd) => run_display(cmd),
        TopCommand::Brightness(cmd) => run_brightness(cmd),
        TopCommand::Nightlight(cmd) => run_nightlight(cmd),
        TopCommand::Wallpaper(cmd) => run_wallpaper(cmd),
        TopCommand::Cursor(cmd) => run_cursor(cmd),
        TopCommand::Idle(cmd) => run_idle(cmd),
    }
}

fn run_display(command: DisplayCommand) -> Result<()> {
    let compositor = backend::compositorctl();
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
        DisplayCommand::Auto => display::auto(&compositor, &backend::notifierctl()),
        DisplayCommand::Reset => display::reset(&compositor),
    }
}

fn run_brightness(command: BrightnessCommand) -> Result<()> {
    match command {
        BrightnessCommand::Get { name } => {
            if let Some(pct) = display::brightness::get(&name)? {
                println!("{pct}");
            }
            Ok(())
        }
        BrightnessCommand::Set { name, percent } => display::brightness::set(&name, percent),
        BrightnessCommand::Step {
            delta_percent,
            name,
        } => display::brightness::step(delta_percent, &name),
        BrightnessCommand::Monitors => {
            for name in display::brightness::monitors()? {
                println!("{name}");
            }
            Ok(())
        }
    }
}

fn run_nightlight(command: NightlightCommand) -> Result<()> {
    match command {
        NightlightCommand::Get => {
            println!("{}", desktop::nightlight::get());
            Ok(())
        }
        NightlightCommand::Set { percent } => desktop::nightlight::set(percent),
        NightlightCommand::Off => desktop::nightlight::off(),
    }
}

fn run_wallpaper(command: WallpaperCommand) -> Result<()> {
    match command {
        WallpaperCommand::Init => desktop::wallpaper::init(),
        WallpaperCommand::Get => {
            if let Some(name) = desktop::wallpaper::get()? {
                println!("{name}");
            }
            Ok(())
        }
        WallpaperCommand::List => {
            for name in desktop::wallpaper::list()? {
                println!("{name}");
            }
            Ok(())
        }
        WallpaperCommand::Dir => {
            println!("{}", desktop::wallpaper::dir().display());
            Ok(())
        }
        WallpaperCommand::Set { name } => desktop::wallpaper::set(&name),
        WallpaperCommand::ApplyPreferred => desktop::wallpaper::apply_preferred(),
        WallpaperCommand::SavePreferred => desktop::wallpaper::save_preferred(),
        WallpaperCommand::Reset => desktop::wallpaper::reset(),
    }
}

fn run_cursor(command: CursorCommand) -> Result<()> {
    match command {
        CursorCommand::List => {
            for name in desktop::cursor::list()? {
                println!("{name}");
            }
            Ok(())
        }
        CursorCommand::ApplyPreferred => desktop::cursor::apply_preferred(),
        CursorCommand::Set { name, size } => {
            desktop::cursor::set(&name, size)?;
            Ok(())
        }
        CursorCommand::SavePreferred { name, size } => {
            desktop::cursor::save_preferred(&name, size)?;
            Ok(())
        }
        CursorCommand::Reset => desktop::cursor::reset(),
    }
}

fn run_idle(command: IdleCommand) -> Result<()> {
    match command {
        IdleCommand::Enable => idle::enable(),
        IdleCommand::Disable => idle::disable(),
        IdleCommand::Status => {
            let inhibited = idle::status()?;
            println!("{}", idle::label(inhibited));
            if inhibited {
                Ok(())
            } else {
                std::process::exit(1);
            }
        }
        IdleCommand::Watch => idle::watch(),
    }
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
    let names = backend::compositorctl()
        .described_monitors(true)
        .map(|monitors| monitors.into_iter().map(|m| m.name).collect())
        .unwrap_or_default();
    candidates(names, current)
}

fn complete_display_mode(current: &OsStr) -> Vec<CompletionCandidate> {
    let mut modes: Vec<String> = backend::compositorctl()
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
    candidates(display::brightness::monitors().unwrap_or_default(), current)
}

fn complete_wallpaper_name(current: &OsStr) -> Vec<CompletionCandidate> {
    candidates(desktop::wallpaper::list().unwrap_or_default(), current)
}

fn complete_cursor_name(current: &OsStr) -> Vec<CompletionCandidate> {
    candidates(desktop::cursor::list().unwrap_or_default(), current)
}
