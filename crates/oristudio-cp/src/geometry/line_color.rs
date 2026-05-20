use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Oriedita line color/type codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LineColor {
    Angle = -2,
    None = -1,
    Black0 = 0,
    Red1 = 1,
    Blue2 = 2,
    Cyan3 = 3,
    Orange4 = 4,
    Magenta5 = 5,
    Green6 = 6,
    Yellow7 = 7,
    Purple8 = 8,
    Other9 = 9,
    Grey10 = 10,
}

impl LineColor {
    pub const fn number(self) -> i32 {
        self as i32
    }

    pub fn from_number(number: i32) -> Result<Self, LineColorParseError> {
        match number {
            -2 => Ok(Self::Angle),
            -1 => Ok(Self::None),
            0 => Ok(Self::Black0),
            1 => Ok(Self::Red1),
            2 => Ok(Self::Blue2),
            3 => Ok(Self::Cyan3),
            4 => Ok(Self::Orange4),
            5 => Ok(Self::Magenta5),
            6 => Ok(Self::Green6),
            7 => Ok(Self::Yellow7),
            8 => Ok(Self::Purple8),
            9 => Ok(Self::Other9),
            10 => Ok(Self::Grey10),
            _ => Err(LineColorParseError::UnknownNumber(number)),
        }
    }

    pub fn advance_folding(self) -> Result<Self, LineColorParseError> {
        if !self.is_folding_line() {
            return Err(LineColorParseError::CannotAdvanceNonFolding(self));
        }

        if self == Self::Blue2 {
            Ok(Self::Black0)
        } else {
            Self::from_number(self.number() + 1)
        }
    }

    pub const fn change_mv(self) -> Self {
        match self {
            Self::Red1 => Self::Blue2,
            Self::Blue2 => Self::Red1,
            _ => self,
        }
    }

    pub const fn change_aux_color(self) -> Self {
        match self {
            Self::Orange4 => Self::Yellow7,
            Self::Yellow7 => Self::Orange4,
            _ => self,
        }
    }

    pub const fn is_folding_line(self) -> bool {
        matches!(self, Self::Black0 | Self::Red1 | Self::Blue2)
    }
}

impl fmt::Display for LineColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.number())
    }
}

impl FromStr for LineColor {
    type Err = LineColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let number = s
            .parse::<i32>()
            .map_err(|_| LineColorParseError::InvalidNumber(s.to_owned()))?;
        Self::from_number(number)
    }
}

/// Errors for parsing or cycling Oriedita line colors.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LineColorParseError {
    #[error("line color number {0} is unknown")]
    UnknownNumber(i32),
    #[error("line color value {0:?} is not an integer")]
    InvalidNumber(String),
    #[error("cannot advance folding on non-folding line color {0}")]
    CannotAdvanceNonFolding(LineColor),
}

#[cfg(test)]
mod tests {
    use super::{LineColor, LineColorParseError};

    #[test]
    fn color_codes_match_oriedita() {
        assert_eq!(LineColor::Angle.number(), -2);
        assert_eq!(LineColor::None.number(), -1);
        assert_eq!(LineColor::Grey10.number(), 10);
        assert_eq!("7".parse::<LineColor>(), Ok(LineColor::Yellow7));
    }

    #[test]
    fn folding_color_cycle_matches_oriedita() {
        assert_eq!(LineColor::Black0.advance_folding(), Ok(LineColor::Red1));
        assert_eq!(LineColor::Red1.advance_folding(), Ok(LineColor::Blue2));
        assert_eq!(LineColor::Blue2.advance_folding(), Ok(LineColor::Black0));
        assert_eq!(
            LineColor::Cyan3.advance_folding(),
            Err(LineColorParseError::CannotAdvanceNonFolding(
                LineColor::Cyan3
            ))
        );
    }
}
