use std::fmt;

#[derive(Debug)]
pub(crate) enum AppError {
    UnknownMonitor {
        name: String,
    },
    UnknownDirection {
        direction: String,
    },
    UnknownSide {
        side: String,
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
    DdcNotResponding {
        name: String,
    },
    NightlightFailed {
        message: String,
    },
}

impl AppError {
    pub(crate) fn should_notify(&self) -> bool {
        match self {
            Self::UnknownMonitor { .. }
            | Self::UnknownDirection { .. }
            | Self::UnknownSide { .. }
            | Self::CannotPlaceSelfRelative
            | Self::CannotPlace { .. }
            | Self::DidNotSwitchOff { .. }
            | Self::DidNotSwitchOn { .. } => false,
            Self::CannotDisableLastScreen
            | Self::DdcNotResponding { .. }
            | Self::NightlightFailed { .. } => true,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownMonitor { name } => write!(f, "Unknown monitor: {name}"),
            Self::UnknownDirection { direction } => write!(f, "Unknown direction: {direction}"),
            Self::UnknownSide { side } => write!(f, "Unknown side: {side}"),
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
            Self::DdcNotResponding { name } => write!(f, "{name} does not respond to DDC/CI"),
            Self::NightlightFailed { message } => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for AppError {}
