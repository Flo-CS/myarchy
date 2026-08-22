mod ops;

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::core::compositor::Monitor;
use crate::core::error::UserError;
use crate::core::resolution::{Resolution, Size};

const SCALE_DECIMALS: f64 = 10000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Direction {
    Left,
    Right,
    Above,
    Below,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Side {
    LeftOf,
    RightOf,
    Above,
    Below,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Side {
    pub fn axis(self) -> Axis {
        match self {
            Self::LeftOf | Self::RightOf => Axis::Horizontal,
            Self::Above | Self::Below => Axis::Vertical,
        }
    }

    pub fn before_reference(self) -> bool {
        matches!(self, Self::LeftOf | Self::Above)
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::LeftOf => "left-of",
            Self::RightOf => "right-of",
            Self::Above => "above",
            Self::Below => "below",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum Mode {
    Preferred,
    Fixed(Resolution),
}

impl Mode {
    fn size(self) -> Option<Size> {
        match self {
            Self::Preferred => None,
            Self::Fixed(resolution) => Some(resolution.size),
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Preferred => f.write_str("preferred"),
            Self::Fixed(resolution) => resolution.fmt(f),
        }
    }
}

impl FromStr for Mode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "preferred" => Ok(Self::Preferred),
            _ => Ok(Self::Fixed(s.parse()?)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum Scale {
    Auto,
    Factor(f64),
}

impl Scale {
    /// A compositor reports `0` for a screen it has switched off, which is not a scale.
    pub fn factor(value: f64) -> Self {
        let rounded = (value * SCALE_DECIMALS).round() / SCALE_DECIMALS;
        if rounded > 0.0 {
            Self::Factor(rounded)
        } else {
            Self::Auto
        }
    }

    fn value(self) -> Option<f64> {
        match self {
            Self::Auto => None,
            Self::Factor(v) => Some(v),
        }
    }
}

impl fmt::Display for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => f.write_str("auto"),
            Self::Factor(v) => write!(f, "{v}"),
        }
    }
}

impl FromStr for Scale {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s == "auto" {
            return Ok(Self::Auto);
        }
        let invalid = || UserError::InvalidScale {
            value: s.to_string(),
        };
        let scale = Self::factor(s.parse().map_err(|_| invalid())?);
        if scale.value().is_none() {
            bail!(invalid());
        }
        Ok(scale)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum Position {
    Auto,
    Toward(Direction),
    At(Point),
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => f.write_str("auto"),
            Self::Toward(Direction::Left) => f.write_str("auto-left"),
            Self::Toward(Direction::Right) => f.write_str("auto-right"),
            Self::Toward(Direction::Above) => f.write_str("auto-up"),
            Self::Toward(Direction::Below) => f.write_str("auto-down"),
            Self::At(p) => write!(f, "{}x{}", p.x, p.y),
        }
    }
}

impl FromStr for Position {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "auto" => Self::Auto,
            "auto-left" => Self::Toward(Direction::Left),
            "auto-right" => Self::Toward(Direction::Right),
            "auto-up" => Self::Toward(Direction::Above),
            "auto-down" => Self::Toward(Direction::Below),
            _ => {
                let (x, y) = s
                    .split_once('x')
                    .ok_or_else(|| anyhow::anyhow!("malformed position: {s}"))?;
                Self::At(Point {
                    x: x.trim().parse()?,
                    y: y.trim().parse()?,
                })
            }
        })
    }
}

macro_rules! string_serde {
    ($($ty:ty),+) => {$(
        impl From<$ty> for String {
            fn from(v: $ty) -> String {
                v.to_string()
            }
        }

        impl TryFrom<String> for $ty {
            type Error = anyhow::Error;

            fn try_from(s: String) -> Result<Self> {
                s.parse()
            }
        }
    )+};
}

string_serde!(Mode, Scale, Position);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    On,
    Off,
    Mirroring(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Placement {
    pub mode: Mode,
    pub position: Position,
    pub scale: Scale,
}

impl Default for Placement {
    fn default() -> Self {
        Self {
            mode: Mode::Preferred,
            position: Position::Auto,
            scale: Scale::Auto,
        }
    }
}

impl Placement {
    /// Hyprland lays windows out in logical pixels, so a screen occupies its mode divided by scale.
    pub fn logical_size(&self) -> Option<Size> {
        let size = self.mode.size()?;
        let scale = self.scale.value().unwrap_or(1.0);
        Some(Size::new(
            (size.width as f64 / scale).round() as i64,
            (size.height as f64 / scale).round() as i64,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Screen {
    pub state: State,
    #[serde(flatten)]
    pub placement: Placement,
}

impl Screen {
    pub fn is_on(&self) -> bool {
        matches!(self.state, State::On)
    }

    pub fn is_off(&self) -> bool {
        matches!(self.state, State::Off)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
    #[serde(default)]
    pub screens: BTreeMap<String, Screen>,
}

impl Layout {
    pub fn observe(monitors: &[Monitor]) -> Self {
        let mut layout = Self::default();
        layout.sync(monitors);
        layout
    }

    /// The compositor is authoritative for a live screen; a screen it reports as off has no
    /// geometry to report (`0x0`, scale `0`), so the stored placement is all we will ever have.
    pub fn sync(&mut self, monitors: &[Monitor]) {
        self.screens
            .retain(|desc, _| monitors.iter().any(|m| &m.description == desc));

        for monitor in monitors {
            let screen = self
                .screens
                .entry(monitor.description.clone())
                .or_insert_with(|| Screen {
                    state: State::On,
                    placement: Placement::default(),
                });

            if monitor.disabled {
                screen.state = State::Off;
                continue;
            }

            screen.state = match monitor.mirror_of.as_deref().and_then(|name| {
                monitors
                    .iter()
                    .find(|m| m.name == name)
                    .map(|m| m.description.clone())
            }) {
                Some(target) => State::Mirroring(target),
                None => State::On,
            };
            screen.placement = Placement {
                mode: Mode::Fixed(monitor.resolution),
                position: Position::At(Point {
                    x: monitor.x,
                    y: monitor.y,
                }),
                scale: Scale::factor(monitor.scale),
            };
        }

        if let Some(anchor) = &self.anchor
            && !self.screens.contains_key(anchor)
        {
            self.anchor = None;
        }
    }

    /// The re-entrancy guard: our own writes fire the hotplug hooks, and a matching layout is what
    /// stops the `auto` they trigger from reloading again.
    pub fn matches(&self, monitors: &[Monitor]) -> bool {
        let live = Self::observe(monitors);
        if live.screens.len() != self.screens.len() {
            return false;
        }
        self.screens.iter().all(|(desc, screen)| {
            live.screens.get(desc).is_some_and(|other| {
                screen.state == other.state
                    && (screen.is_off() || screen.placement == other.placement)
            })
        })
    }

    pub fn on(&self) -> impl Iterator<Item = (&String, &Screen)> {
        self.screens.iter().filter(|(_, s)| s.is_on())
    }

    pub fn enabled_count(&self) -> usize {
        self.screens.values().filter(|s| !s.is_off()).count()
    }

    pub fn require(&mut self, desc: &str) -> Result<&mut Screen> {
        match self.screens.get_mut(desc) {
            Some(screen) => Ok(screen),
            None => bail!(UserError::NoStoredScreen {
                description: desc.to_string()
            }),
        }
    }
}

#[cfg(test)]
pub mod fixtures {
    use super::*;

    pub const BOE: &str = "BOE 0x08B9";
    pub const LG: &str = "LG Electronics LG HDR WQHD 303NTZN51357";
    pub const DELL: &str = "Dell Inc. DELL U2412M PMSXXXX";

    pub fn monitor(name: &str, description: &str, mode: &str, at: (i64, i64)) -> Monitor {
        Monitor {
            name: name.into(),
            description: description.into(),
            resolution: mode.parse().unwrap(),
            x: at.0,
            y: at.1,
            scale: 1.0,
            disabled: false,
            focused: false,
            mirror_of: None,
            resolutions: vec![mode.parse().unwrap()],
        }
    }

    /// hyprctl reports a switched-off screen with its geometry blanked, zeroes and all.
    pub fn switched_off(mut monitor: Monitor) -> Monitor {
        monitor.disabled = true;
        monitor.resolution = Resolution::new(Size::new(0, 0), 0.0);
        monitor.x = 0;
        monitor.y = 0;
        monitor.scale = 0.0;
        monitor
    }

    pub fn laptop() -> Monitor {
        monitor("eDP-1", BOE, "1920x1080@60.003", (0, 0))
    }

    pub fn ultrawide() -> Monitor {
        monitor("DP-3", LG, "3440x1440@99.997", (1920, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;

    #[test]
    fn sync_takes_geometry_from_a_live_screen_and_keeps_it_for_a_dark_one() {
        let mut layout = Layout::observe(&[laptop(), ultrawide()]);
        assert_eq!(
            layout.screens[LG].placement.position,
            "1920x0".parse().unwrap()
        );

        let mut moved = laptop();
        moved.x = 500;
        moved.y = 300;
        layout.sync(&[moved, switched_off(ultrawide())]);

        assert_eq!(
            layout.screens[BOE].placement.position,
            "500x300".parse().unwrap(),
            "adopted from the snapshot"
        );
        assert_eq!(layout.screens[LG].state, State::Off);
        assert_eq!(
            layout.screens[LG].placement.position,
            "1920x0".parse().unwrap(),
            "kept, not hyprctl's 0x0"
        );
        assert_eq!(
            layout.screens[LG].placement.mode,
            "3440x1440@99.997".parse().unwrap()
        );
        assert_eq!(layout.screens[LG].placement.scale, Scale::factor(1.0));
    }

    #[test]
    fn sync_follows_screens_arriving_and_leaving() {
        let mut layout = Layout::observe(&[laptop(), ultrawide()]);
        layout.anchor = Some(LG.into());

        let dell = monitor("DP-1", DELL, "1920x1200@59.95", (0, 0));
        layout.sync(&[laptop(), dell]);

        assert_eq!(layout.screens.keys().collect::<Vec<_>>(), vec![BOE, DELL]);
        assert_eq!(layout.anchor, None, "an anchor that left is not kept");
    }

    #[test]
    fn sync_records_which_screen_a_mirror_is_copying() {
        let mut mirroring = laptop();
        mirroring.mirror_of = Some("DP-3".into());

        let layout = Layout::observe(&[mirroring, ultrawide()]);

        assert_eq!(layout.screens[BOE].state, State::Mirroring(LG.into()));
    }

    #[test]
    fn matches_holds_while_the_screens_still_look_like_the_layout() {
        let live = [laptop(), switched_off(ultrawide())];
        let layout = Layout::observe(&live);

        assert!(layout.matches(&live));
        assert!(
            layout.matches(&[laptop(), switched_off(ultrawide())]),
            "a dark screen's geometry is not compared"
        );

        let mut moved = laptop();
        moved.x = 500;
        assert!(!layout.matches(&[moved, switched_off(ultrawide())]));

        assert!(
            !layout.matches(&[laptop(), ultrawide()]),
            "switched back on"
        );
    }

    #[test]
    fn a_scale_has_to_be_a_positive_number() {
        for bad in ["0", "-1", "-0.5", "nonsense", ""] {
            assert!(bad.parse::<Scale>().is_err(), "{bad:?}");
        }
        assert_eq!("auto".parse::<Scale>().unwrap(), Scale::Auto);
        assert_eq!("1.25".parse::<Scale>().unwrap(), Scale::factor(1.25));
    }
}
