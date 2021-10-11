/// Represents the four ordinal directions
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Rotates a direction to the right
    fn turn_right(self) -> Direction {
        match self {
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
        }
    }

    /// Rotates a direction to the left
    fn turn_left(self) -> Direction {
        match self {
            Direction::Up => Direction::Left,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
            Direction::Right => Direction::Up,
        }
    }

    /// Rotates a direction 180 degrees
    fn turn_around(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// GIVEN the four ordinal directions
    /// WHEN the are turned left
    /// THEN we get the correct result
    #[test]
    fn turn_left() {
        assert_eq!(Direction::Up.turn_left(), Direction::Left);
        assert_eq!(Direction::Down.turn_left(), Direction::Right);
        assert_eq!(Direction::Left.turn_left(), Direction::Down);
        assert_eq!(Direction::Right.turn_left(), Direction::Up);
    }

    /// GIVEN the four ordinal directions
    /// WHEN the are turned right
    /// THEN we get the correct result
    #[test]
    fn turn_right() {
        assert_eq!(Direction::Up.turn_right(), Direction::Right);
        assert_eq!(Direction::Down.turn_right(), Direction::Left);
        assert_eq!(Direction::Left.turn_right(), Direction::Up);
        assert_eq!(Direction::Right.turn_right(), Direction::Down);
    }

    /// GIVEN the four ordinal directions
    /// WHEN the are turned around
    /// THEN we get the correct result
    #[test]
    fn turn_around() {
        assert_eq!(Direction::Up.turn_around(), Direction::Down);
        assert_eq!(Direction::Down.turn_around(), Direction::Up);
        assert_eq!(Direction::Left.turn_around(), Direction::Right);
        assert_eq!(Direction::Right.turn_around(), Direction::Left);
    }
}
