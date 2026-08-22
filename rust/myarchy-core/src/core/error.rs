use std::fmt;

use crate::core::layout::Side;
use crate::core::resolution::Size;

/// Failures worth putting in front of whoever is at the keyboard. Every variant keeps its
/// parameters apart from its wording, so a catalogue lookup can replace the `Display` impl
/// below without touching a single call site.
///
/// A technical failure keeps its English `anyhow` chain and carries the variant that says what it
/// means to a person as context, so the chain reaches the log and the wording reaches the screen.
/// Anything with no such meaning surfaces as `Unexpected`.
// TODO: please prefix everything with the context, DidNotSwitchOn or Off is very vague for example
#[derive(Debug)]
pub enum UserError {
    UnknownMonitor {
        name: String,
    },
    NoStoredScreen {
        description: String,
    },
    CannotPlaceSelfRelative,
    CannotPlace {
        name: String,
        side: Side,
        reference: String,
    },
    CannotDisableLastScreen,
    DidNotSwitchOff {
        name: String,
    },
    DidNotSwitchOn {
        name: String,
    },
    LayoutDidNotSettle,
    NoScreenToBuildAround,
    NoSuchMode {
        name: String,
        size: Size,
    },
    InvalidMode {
        value: String,
    },
    InvalidScale {
        value: String,
    },
    CompositorNotRunning,
    CompositorRejected,
    AnotherDisplayCommandRunning,
    ProfileUnreadable {
        path: String,
    },
    DdcNotResponding {
        name: String,
    },
    UnknownWallpaper {
        name: String,
    },
    NoWallpapers,
    NoCurrentWallpaper,
    WallpaperDirUnreadable {
        path: String,
    },
    NoPreferredWallpaper,
    NoPreferredCursor,
    CursorNotApplied {
        name: String,
    },
    InvalidThemeValue {
        key: String,
    },
    NightlightNotApplied,
    IdleDaemonNotRunning,
    ToolMissing {
        tool: String,
    },
    Unexpected,
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownMonitor { name } => write!(f, "Unknown monitor: {name}"),
            Self::NoStoredScreen { description } => {
                write!(f, "No stored screen for {description}")
            }
            Self::CannotPlaceSelfRelative => {
                write!(f, "Cannot place a monitor relative to itself")
            }
            Self::CannotPlace {
                name,
                side,
                reference,
            } => write!(f, "Cannot place {name} {side} {reference}"),
            Self::CannotDisableLastScreen => write!(f, "Cannot disable the last enabled screen"),
            Self::DidNotSwitchOff { name } => write!(f, "{name} did not switch off"),
            Self::DidNotSwitchOn { name } => write!(f, "{name} did not switch on"),
            Self::LayoutDidNotSettle => write!(f, "The screen layout did not settle"),
            Self::NoScreenToBuildAround => {
                write!(f, "No enabled screen to build the layout around")
            }
            Self::NoSuchMode { name, size } => write!(f, "{name} has no {size} mode"),
            Self::InvalidMode { value } => {
                write!(f, "Invalid mode: {value} (like 1920x1080 or 1920x1080@60)")
            }
            Self::InvalidScale { value } => {
                write!(f, "Invalid scale: {value} (a positive number, or auto)")
            }
            Self::CompositorNotRunning => write!(f, "Hyprland is not running"), // TODO: make this more generic, so it can be used with other compositors
            Self::CompositorRejected => write!(f, "Hyprland refused the command"),
            Self::AnotherDisplayCommandRunning => {
                write!(f, "Another display command is still running")
            }
            Self::ProfileUnreadable { path } => {
                write!(f, "The saved display profile cannot be read: {path}")
            }
            Self::DdcNotResponding { name } => write!(f, "{name} does not respond to DDC/CI"),
            Self::UnknownWallpaper { name } => write!(f, "Unknown wallpaper: {name}"),
            Self::NoWallpapers => write!(f, "No wallpapers found"),
            Self::NoCurrentWallpaper => write!(f, "No wallpaper is currently set"),
            Self::WallpaperDirUnreadable { path } => {
                write!(f, "The wallpapers directory cannot be read: {path}")
            }
            Self::NoPreferredWallpaper => {
                write!(f, "No wallpaper name found in the current theme")
            }
            Self::NoPreferredCursor => write!(f, "No cursor config found for the current theme"),
            Self::CursorNotApplied { name } => write!(f, "The cursor theme {name} was not applied"),
            Self::InvalidThemeValue { key } => {
                write!(f, "The current theme has an unusable {key}")
            }
            Self::NightlightNotApplied => write!(f, "The night light was not changed"),
            Self::IdleDaemonNotRunning => write!(f, "The idle inhibitor daemon is not running"),
            Self::ToolMissing { tool } => write!(f, "{tool} is not installed"),
            Self::Unexpected => write!(f, "Something went wrong"),
        }
    }
}

impl std::error::Error for UserError {}
