use std::fmt;
use std::str::FromStr;

use anyhow::{Result, anyhow};

const REFRESH_DECIMALS: f64 = 1000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Size {
    pub width: i64,
    pub height: i64,
}

impl Size {
    pub fn new(width: i64, height: i64) -> Self {
        Self { width, height }
    }
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl FromStr for Size {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        let (width, height) = s
            .split_once('x')
            .ok_or_else(|| anyhow!("malformed resolution: {s}"))?;
        Ok(Self {
            width: width.trim().parse()?,
            height: height.trim().parse()?,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resolution {
    pub size: Size,
    pub refresh: f64,
}

impl Resolution {
    /// Refresh rates arrive at several precisions — `60.00Hz` from a mode list, `60.003` from a
    /// live monitor — so they are rounded to one, and equality stays exact.
    pub fn new(size: Size, refresh: f64) -> Self {
        Self {
            size,
            refresh: (refresh * REFRESH_DECIMALS).round() / REFRESH_DECIMALS,
        }
    }

    /// Whole-number refresh, for lists a person reads rather than a rule the compositor parses.
    pub fn rounded(&self) -> String {
        format!("{}@{}", self.size, self.refresh.floor())
    }
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.size, self.refresh)
    }
}

impl FromStr for Resolution {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim();
        let (size, refresh) = s
            .split_once('@')
            .ok_or_else(|| anyhow!("resolution needs a refresh rate, e.g. 1920x1080@60: {s}"))?;
        let refresh = refresh.trim().trim_end_matches("Hz").trim();

        Ok(Self::new(size.parse()?, refresh.parse()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_reads_every_form_a_refresh_rate_arrives_in() {
        for form in ["1920x1080@60", "1920x1080@60.0", "1920x1080@60.00Hz"] {
            assert_eq!(
                form.parse::<Resolution>().unwrap(),
                Resolution::new(Size::new(1920, 1080), 60.0),
                "{form}"
            );
        }
        assert_ne!(
            Resolution::new(Size::new(1, 1), 59.94),
            Resolution::new(Size::new(1, 1), 60.0)
        );

        let exact = Resolution::new(Size::new(3440, 1440), 99.997);
        assert_eq!(exact.to_string(), "3440x1440@99.997");
        assert_eq!(exact.to_string().parse::<Resolution>().unwrap(), exact);
        assert_eq!(exact.rounded(), "3440x1440@99", "the form the menu lists");

        for bad in [
            "",
            "nonsense",
            "1920",
            "x1080",
            "1920x1080@fast",
            "1920x1080",
        ] {
            assert!(bad.parse::<Resolution>().is_err(), "{bad:?}");
        }
    }

    #[test]
    fn size_round_trips_a_bare_width_and_height() {
        for form in ["1920x1080", "3440x1440"] {
            assert_eq!(form.parse::<Size>().unwrap().to_string(), form);
        }
        for bad in ["", "nonsense", "1920", "x1080"] {
            assert!(bad.parse::<Size>().is_err(), "{bad:?}");
        }
    }
}
