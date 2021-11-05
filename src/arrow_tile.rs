use super::Direction;
use std::convert::TryFrom;

/// Represents an arrow tile
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ArrowType {
    Empty,
    Up,
    UpHalf,
    Down,
    DownHalf,
    Left,
    LeftHalf,
    Right,
    RightHalf,
}

impl ArrowType {
    /// Diminishes the size of an arrow. Other tile types are
    /// unaffected
    /// #examples
    /// ```
    /// use shoko_rocket_rust::{ArrowType};
    /// ArrowType::Up.diminish();
    /// ```
    pub fn diminish(self) -> ArrowType {
        match self {
            ArrowType::Up => ArrowType::UpHalf,
            ArrowType::Down => ArrowType::DownHalf,
            ArrowType::Left => ArrowType::LeftHalf,
            ArrowType::Right => ArrowType::RightHalf,
            ArrowType::UpHalf
            | ArrowType::DownHalf
            | ArrowType::LeftHalf
            | ArrowType::RightHalf => ArrowType::Empty,
            _ => self,
        }
    }
}

impl TryFrom<ArrowType> for Direction {
    type Error = ();

    /// Converts the arrow type to the corresponding
    /// #examples
    /// ```
    /// use shoko_rocket_rust::{ArrowType, Direction};
    /// use std::convert::TryInto;
    /// let direction: Direction = ArrowType::Up.try_into().unwrap();
    /// ```
    fn try_from(arrow_type: ArrowType) -> Result<Self, Self::Error> {
        match arrow_type {
            ArrowType::Up | ArrowType::UpHalf => Ok(Direction::Up),
            ArrowType::Down | ArrowType::DownHalf => Ok(Direction::Down),
            ArrowType::Left | ArrowType::LeftHalf => Ok(Direction::Left),
            ArrowType::Right | ArrowType::RightHalf => Ok(Direction::Right),
            ArrowType::Empty => Err(()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// GIVEN A full arrow
    /// WHEN we diminish it
    /// THEN a half arrow is output
    #[test]
    fn diminish_full_arrow() {
        assert_eq!(ArrowType::UpHalf, ArrowType::Up.diminish());
        assert_eq!(ArrowType::DownHalf, ArrowType::Down.diminish());
        assert_eq!(ArrowType::LeftHalf, ArrowType::Left.diminish());
        assert_eq!(ArrowType::RightHalf, ArrowType::Right.diminish());
    }

    /// GIVEN A half arrow
    /// WHEN we diminish it
    /// THEN an empty is output
    #[test]
    fn diminish_half_arrow() {
        assert_eq!(ArrowType::Empty, ArrowType::UpHalf.diminish());
        assert_eq!(ArrowType::Empty, ArrowType::DownHalf.diminish());
        assert_eq!(ArrowType::Empty, ArrowType::LeftHalf.diminish());
        assert_eq!(ArrowType::Empty, ArrowType::RightHalf.diminish());
    }

    /// GIVEN an empty
    /// WHEN we diminish it
    /// THEN an empty is output
    #[test]
    fn diminish_empty() {
        assert_eq!(ArrowType::Empty, ArrowType::Empty.diminish());
    }
}
