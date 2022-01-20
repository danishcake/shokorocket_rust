use super::Direction;
use std::convert::TryFrom;

/// Represents a tile
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TileType {
    Empty,
    Rocket,
    Hole,
    Up,
    UpHalf,
    Down,
    DownHalf,
    Left,
    LeftHalf,
    Right,
    RightHalf,
}

impl TileType {
    /// Diminishes the size of an arrow. Other tile types are
    /// unaffected
    /// #examples
    /// ```
    /// use shoko_rocket_rust::{TileType};
    /// TileType::Up.diminish();
    /// ```
    pub fn diminish(self) -> TileType {
        match self {
            TileType::Up => TileType::UpHalf,
            TileType::Down => TileType::DownHalf,
            TileType::Left => TileType::LeftHalf,
            TileType::Right => TileType::RightHalf,
            TileType::UpHalf | TileType::DownHalf | TileType::LeftHalf | TileType::RightHalf => {
                TileType::Empty
            }
            _ => self,
        }
    }
}

impl TryFrom<TileType> for Direction {
    type Error = ();

    /// Converts the arrow type to the corresponding Direction
    /// #examples
    /// ```
    /// use shoko_rocket_rust::{TileType, Direction};
    /// use std::convert::TryInto;
    /// let direction: Direction = TileType::Up.try_into().unwrap();
    /// ```
    fn try_from(tile_type: TileType) -> Result<Self, Self::Error> {
        match tile_type {
            TileType::Up | TileType::UpHalf => Ok(Direction::Up),
            TileType::Down | TileType::DownHalf => Ok(Direction::Down),
            TileType::Left | TileType::LeftHalf => Ok(Direction::Left),
            TileType::Right | TileType::RightHalf => Ok(Direction::Right),
            TileType::Empty | TileType::Rocket | TileType::Hole => Err(()),
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
        assert_eq!(TileType::UpHalf, TileType::Up.diminish());
        assert_eq!(TileType::DownHalf, TileType::Down.diminish());
        assert_eq!(TileType::LeftHalf, TileType::Left.diminish());
        assert_eq!(TileType::RightHalf, TileType::Right.diminish());
    }

    /// GIVEN A half arrow
    /// WHEN we diminish it
    /// THEN an empty is output
    #[test]
    fn diminish_half_arrow() {
        assert_eq!(TileType::Empty, TileType::UpHalf.diminish());
        assert_eq!(TileType::Empty, TileType::DownHalf.diminish());
        assert_eq!(TileType::Empty, TileType::LeftHalf.diminish());
        assert_eq!(TileType::Empty, TileType::RightHalf.diminish());
    }

    /// GIVEN an empty/hole/rocket
    /// WHEN we diminish it
    /// THEN an empty is output
    #[test]
    fn diminish_other() {
        assert_eq!(TileType::Empty, TileType::Empty.diminish());
        assert_eq!(TileType::Hole, TileType::Hole.diminish());
        assert_eq!(TileType::Rocket, TileType::Rocket.diminish());
    }
}
