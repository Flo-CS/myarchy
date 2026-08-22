use std::fmt;
use std::str::FromStr;

use anyhow::Result;

/// Clamped on the way in, so nothing downstream has to ask whether a reading is in range.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Percent(u8);

impl Percent {
    pub fn new(value: i64) -> Self {
        Self(value.clamp(0, 100) as u8)
    }

    pub fn get(self) -> i64 {
        self.0 as i64
    }

    pub fn offset(self, delta: i64) -> Self {
        Self::new(self.get().saturating_add(delta))
    }

    pub fn fraction(self) -> f64 {
        self.get() as f64 / 100.0
    }
}

impl fmt::Display for Percent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Percent {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::new(s.trim().trim_end_matches('%').parse()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_percentage_never_leaves_its_range_however_it_is_built() {
        for value in [-300, -1, 0, 1, 50, 100, 101, 9000] {
            let percent = Percent::new(value);
            assert!((0..=100).contains(&percent.get()), "{value}");
            assert_eq!(percent.get(), value.clamp(0, 100));
        }

        for (start, delta) in [
            (0, -10),
            (100, 10),
            (50, -80),
            (50, 80),
            (50, i64::MIN),
            (50, i64::MAX),
        ] {
            let moved = Percent::new(start).offset(delta);
            assert!((0..=100).contains(&moved.get()), "{start} {delta}");
        }

        assert_eq!("60%".parse::<Percent>().unwrap(), Percent::new(60));
        assert_eq!("150".parse::<Percent>().unwrap(), Percent::new(100));
        assert_eq!(Percent::new(50).fraction(), 0.5);
    }
}
