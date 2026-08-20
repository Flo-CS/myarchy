use std::fmt;

pub(crate) enum NightlightState {
    Off,
    On(i64),
}

impl fmt::Display for NightlightState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NightlightState::Off => write!(f, "off"),
            NightlightState::On(percent) => write!(f, "{percent}"),
        }
    }
}
