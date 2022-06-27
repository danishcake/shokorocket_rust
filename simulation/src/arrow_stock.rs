use crate::Direction;
use core::ops::{Index, IndexMut};

/// Represents a stock of unused arrows
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ArrowStock {
    up: u8,
    down: u8,
    left: u8,
    right: u8,
}

impl ArrowStock {
    /// Creates an empty ArrowStock
    /// #examples
    /// ```
    /// use shoko_rocket_rust::{ArrowStock, Direction};
    /// let mut arrow_stock = ArrowStock::new();
    /// arrow_stock[Direction::Left] = 10
    /// assert_eq!(0,  arrow_stock[Direction::Up]);
    /// assert_eq!(0,  arrow_stock[Direction::Down]);
    /// assert_eq!(10, arrow_stock[Direction::Left]);
    /// assert_eq!(0,  arrow_stock[Direction::Right]);
    /// ```
    pub fn new() -> ArrowStock {
        ArrowStock {
            up: 0,
            down: 0,
            left: 0,
            right: 0,
        }
    }
}

/// Immutable indexing of ArrowStock by direction
impl Index<Direction> for ArrowStock {
    type Output = u8;

    fn index(&self, direction: Direction) -> &Self::Output {
        match direction {
            Direction::Up => &self.up,
            Direction::Down => &self.down,
            Direction::Left => &self.left,
            Direction::Right => &self.right,
        }
    }
}

/// Mutable indexing of ArrowStock by direction
impl IndexMut<Direction> for ArrowStock {
    fn index_mut(&mut self, direction: Direction) -> &mut Self::Output {
        match direction {
            Direction::Up => &mut self.up,
            Direction::Down => &mut self.down,
            Direction::Left => &mut self.left,
            Direction::Right => &mut self.right,
        }
    }
}
