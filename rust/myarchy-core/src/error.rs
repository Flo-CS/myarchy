use std::fmt;

use crate::notify::Icon;

#[derive(Debug)]
pub enum AppError {
    UnknownMonitor {
        name: String,
    },
    CannotPlaceSelfRelative,
    CannotPlace {
        name: String,
        side: String,
        refname: String,
    },
    DidNotSwitchOff {
        name: String,
    },
    DidNotSwitchOn {
        name: String,
    },
    CannotDisableLastScreen,
    LayoutDidNotSettle,
    DdcNotResponding {
        name: String,
    },
    NightlightFailed {
        message: String,
    },
    UnknownWallpaper {
        name: String,
    },
    NoPreferredWallpaper,
    NoPreferredCursor,
    CursorFailed {
        message: String,
    },
}

pub struct Notification {
    pub summary: &'static str,
    pub icon: Icon,
}

impl AppError {
    /// A mistyped name is already on stderr in front of whoever typed it; only what a keybinding or
    /// a hotplug hook can hit with nobody watching is worth a toast.
    pub fn notification(&self) -> Option<Notification> {
        let (summary, icon) = match self {
            Self::UnknownMonitor { .. }
            | Self::CannotPlaceSelfRelative
            | Self::CannotPlace { .. }
            | Self::DidNotSwitchOff { .. }
            | Self::DidNotSwitchOn { .. }
            | Self::UnknownWallpaper { .. } => return None,
            Self::LayoutDidNotSettle | Self::CannotDisableLastScreen => {
                ("Display", Icon::VideoDisplay)
            }
            Self::DdcNotResponding { .. } => ("Brightness", Icon::BrightnessSymbolic),
            Self::NightlightFailed { .. } => ("Night light", Icon::DialogError),
            Self::NoPreferredWallpaper => ("Wallpaper", Icon::DialogError),
            Self::NoPreferredCursor | Self::CursorFailed { .. } => ("Cursor", Icon::DialogError),
        };
        Some(Notification { summary, icon })
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownMonitor { name } => write!(f, "Unknown monitor: {name}"),
            Self::CannotPlaceSelfRelative => {
                write!(f, "Cannot place a monitor relative to itself")
            }
            Self::CannotPlace {
                name,
                side,
                refname,
            } => {
                write!(f, "Cannot place {name} {side} {refname}")
            }
            Self::DidNotSwitchOff { name } => write!(f, "{name} did not switch off"),
            Self::DidNotSwitchOn { name } => write!(f, "{name} did not switch on"),
            Self::CannotDisableLastScreen => write!(f, "Cannot disable the last enabled screen"),
            Self::LayoutDidNotSettle => write!(f, "The screen layout did not settle"),
            Self::DdcNotResponding { name } => write!(f, "{name} does not respond to DDC/CI"),
            Self::NightlightFailed { message } => write!(f, "{message}"),
            Self::UnknownWallpaper { name } => write!(f, "Unknown wallpaper: {name}"),
            Self::NoPreferredWallpaper => {
                write!(f, "No wallpaper name found in the current theme")
            }
            Self::NoPreferredCursor => write!(f, "No cursor config found for current theme"),
            Self::CursorFailed { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for AppError {}
