use super::Direction;
use super::FixedPoint;

/// Type of walker. This determines how fast they move
pub enum WalkerType {
    Mouse,
    Cat,
}

/// Used to indicate significant positions reached in a walk cycle
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum WalkResult {
    None,
    NewSquare, // TODO: Figure out other significant thresholds where cats and mice can collide
               // e.g. if walking towards each other, away from each other, at right angles etc
}

/// A walker. This can be a cat or a mouse
pub struct Walker {
    x: FixedPoint,
    y: FixedPoint,
    direction: Direction,
    walker_type: WalkerType,
}

impl Walker {
    /// Creates a new walker
    /// #examples
    /// ```
    /// use shoko_rocket_rust::{Walker, Direction, WalkerType};
    /// let walker = Walker::new(0, 0, Direction::Right, WalkerType::Mouse);
    /// ```
    pub fn new(x: i8, y: i8, direction: Direction, walker_type: WalkerType) -> Walker {
        Walker {
            x: FixedPoint::new(x, 0),
            y: FixedPoint::new(y, 0),
            direction,
            walker_type,
        }
    }

    /// Advances the position of a walker
    /// #examples
    /// ```
    /// use shoko_rocket_rust::{Walker, Direction, WalkerType};
    /// let mut walker = Walker::new(0, 0, Direction::Right, WalkerType::Mouse);
    /// let walk_result = walker.walk();
    /// ```
    pub fn walk(&mut self) -> WalkResult {
        // Determine the speed of the walker
        let speed = match self.walker_type {
            WalkerType::Cat => FixedPoint::new(0, 4),
            WalkerType::Mouse => FixedPoint::new(0, 6),
        };

        // Advance the position and determine if a new grid was reached
        let reached_new_square = match self.direction {
            Direction::Up => {
                let start = self.y;
                self.y -= speed;
                self.y.did_overflow(start)
            }
            Direction::Down => {
                let start = self.y;
                self.y += speed;
                self.y.did_overflow(start)
            }
            Direction::Left => {
                let start = self.x;
                self.x -= speed;
                self.x.did_overflow(start)
            }
            Direction::Right => {
                let start = self.x;
                self.x += speed;
                self.x.did_overflow(start)
            }
        };

        return if reached_new_square {
            WalkResult::NewSquare
        } else {
            WalkResult::None
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// GIVEN a mouse at 0,0
    /// WHEN it walks right 180 times
    /// THEN it reaches a new square on the 60th, 120th and 180th walk cycle
    #[test]
    fn mouse_walker_indicates_new_square() {
        let mut walker = Walker::new(0, 0, Direction::Right, WalkerType::Mouse);

        for step in 1..=180 {
            if step % 60 == 0 {
                assert_eq!(WalkResult::NewSquare, walker.walk());
            } else {
                assert_eq!(WalkResult::None, walker.walk());
            }
        }
    }

    /// GIVEN a cat at 0,0
    /// WHEN it walks right 180 times
    /// THEN it reaches a new square on the 90th and 180th walk cycle
    #[test]
    fn cat_walker_indicates_new_square() {
        let mut walker = Walker::new(0, 0, Direction::Right, WalkerType::Cat);

        for step in 1..=180 {
            if step % 90 == 0 {
                assert_eq!(WalkResult::NewSquare, walker.walk());
            } else {
                assert_eq!(WalkResult::None, walker.walk());
            }
        }
    }
}
