use arrayvec::ArrayVec;

use crate::{ArrowTile, Walker, WalkerType, walker::WalkResult};

use super::direction::Direction;

pub const WORLD_WIDTH: usize = 12;
pub const WORLD_HEIGHT: usize = 9;
const MAX_WALKERS: usize = WORLD_WIDTH * WORLD_HEIGHT;
const MAX_ARROWS: usize = 32;
const HEADER_SIZE: usize = 64;

const LEFT_WALL_MASK: [u8; 4] = [0b00000010, 0b00001000, 0b00100000, 0b10000000];
const TOP_WALL_MASK: [u8; 4] = [0b00000001, 0b00000100, 0b00010000, 0b01000000];

/// Represents the entire state of a world
/// This is a 12x9 array of squares. Each square controls the top and left walls,
/// and can have one of a cat, mouse or rocket in it. This information is packed
/// to minimise space
/// struct Header_t
/// {
///   char[32] name;
///   char[32] author;
/// }
///
/// Followed by 27 of the following
/// {
///   uint8_t: 1 wall_up_0;
///   uint8_t: 1 wall_left_0;
///   uint8_t: 1 wall_up_1;
///   uint8_t: 1 wall_left_1;
///   uint8_t: 1 wall_up_2;
///   uint8_t: 1 wall_left_2;
///   uint8_t: 1 wall_up_3;
///   uint8_t: 1 wall_left_3;
/// }[27]
///
/// Followed by 108 of the following
/// {
///   uint8_t: 3 entity; // 0 -> empty, 1 -> mouse, 2 -> cat, 3 -> rocket, 4 -> hole, 5-7 -> unused
///   uint8_t: 2 entity_direction; // 0 -> up, 1 -> down, 2 -> left, 3 -> right
///   uint8_t: 1 arrow; // 0 -> empty, 1 -> arrow
///   uint8_t: 2 arrow_direction; // 0 -> up, 1 -> down, 2 -> left, 3 -> right
/// }
/// For a total of 64 + 27 + 108 bytes = 199 bytes
/// TODO: More constants!
pub struct World {
    data: [u8; 199],
    mice: ArrayVec<Walker, MAX_WALKERS>,
    cats: ArrayVec<Walker, MAX_WALKERS>,
    arrows: ArrayVec<ArrowTile, MAX_ARROWS>
}

impl World {
    /// Creates a new world with walls around the edge
    /// #examples
    /// ```
    /// use shoko_rocket_rust::{World, Direction};
    /// let world = World::new();
    /// assert_eq!(true,  world.get_wall(0, 0, Direction::Up));
    /// assert_eq!(false, world.get_wall(0, 0, Direction::Down));
    /// assert_eq!(true,  world.get_wall(0, 0, Direction::Left));
    /// assert_eq!(false, world.get_wall(0, 0, Direction::Right));
    /// ```
    pub fn new() -> World {
        // Create the world
        let mut world = World {
            data: [0; 199],
            mice: ArrayVec::new(),
            cats: ArrayVec::new(),
            arrows: ArrayVec::new()
        };

        // Set the walls along the top/left, which also sets the right/bottom
        for x in 0..WORLD_WIDTH {
            world.set_wall(x, 0, Direction::Up, true);
        }

        for y in 0..WORLD_HEIGHT {
            world.set_wall(0, y, Direction::Left, true);
        }

        world
    }

    /// Gets the index into the wall data of a particular wall, and the mask required to
    /// get/set it.
    ///
    /// Arguments:
    /// * `x`: The x coordinate to set. Must be in range 0-11
    /// * `y`: The y coordinate to set. Must be in range 0-8
    /// * `direction`: The direction to set
    ///
    /// Return value:
    /// A tuple containing the wall index and the mask required to extract the given direction
    fn get_wrapped_wall_index_and_mask(
        x: usize,
        y: usize,
        direction: Direction,
    ) -> (usize, u8) {
        assert!(x < WORLD_WIDTH);
        assert!(y < WORLD_HEIGHT);

        // Find the affected walls array and position of the requested wall in terms of up/left
        let (e_x, e_y, mask) = match direction {
            Direction::Up => (x, y, TOP_WALL_MASK[x & 0x03]),
            Direction::Down => (x, (y + 1) % WORLD_HEIGHT, TOP_WALL_MASK[x & 0x03]),
            Direction::Left => (x, y, LEFT_WALL_MASK[x & 0x03]),
            Direction::Right => ((x + 1) % WORLD_WIDTH, y, LEFT_WALL_MASK[(x + 1) & 0x03]),
        };

        ((e_y * WORLD_WIDTH + e_x) / 4, mask)
    }

    /// Sets a wall present/non-present
    ///
    /// Arguments:
    /// * `x`: The x coordinate to set. Must be in range 0-11
    /// * `y`: The y coordinate to set. Must be in range 0-8
    /// * `direction`: The direction to set
    /// * `present`: If the wall should be present
    ///
    /// #examples
    /// ```
    /// use shoko_rocket_rust::{World, Direction};
    /// let mut world = World::new();
    /// world.set_wall(0, 0, Direction::Down, true);
    /// assert!(world.get_wall(0, 0, Direction::Down));
    /// ```
    pub fn set_wall(&mut self, x: usize, y: usize, direction: Direction, present: bool) {
        assert!(x < WORLD_WIDTH);
        assert!(y < WORLD_HEIGHT);

        let (wall_index, mask) = World::get_wrapped_wall_index_and_mask(x, y, direction);
        let byte = &mut self.data[HEADER_SIZE + wall_index];

        if present {
            *byte = *byte | mask;
        } else {
            *byte = *byte & !mask;
        }
    }

    /// Gets the presence of a wall in the specified position and direction
    ///
    /// Arguments:
    /// * `wall_data`: The internal representation of the walls
    /// * `x`: The x coordinate to check. Must be in range 0-11
    /// * `y`: The y coordinate to check. Must be in range 0-8
    /// * `direction`: The direction to check
    ///
    /// Return value:
    /// True if the value is present
    fn get_wall_static(wall_data: &[u8; 199], x: usize, y: usize, direction: Direction) -> bool {
        assert!(x < WORLD_WIDTH);
        assert!(y < WORLD_HEIGHT);

        let (wall_index, mask) = World::get_wrapped_wall_index_and_mask(x, y, direction);
        let byte = &wall_data[HEADER_SIZE + wall_index];
        return *byte & mask == mask;
    }

    /// Gets the presence of a wall in the specified position and direction
    ///
    /// Arguments:
    /// * `x`: The x coordinate to check. Must be in range 0-11
    /// * `y`: The y coordinate to check. Must be in range 0-8
    /// * `direction`: The direction to check
    ///
    /// Return value:
    /// True if the value is present
    ///
    /// #examples
    /// ```
    /// use shoko_rocket_rust::{World, Direction};
    /// let mut world = World::new();
    /// assert!(world.get_wall(0, 0, Direction::Up));
    /// ```
    pub fn get_wall(&self, x: usize, y: usize, direction: Direction) -> bool {
        World::get_wall_static(&self.data, x, y, direction)
    }

    /// Creates a walker. There are a limited number of walkers that can be created, and this
    /// function will panic if too many are created
    ///
    /// Arguments:
    /// * `x`: The x coordinate of the walker
    /// * `y`: The y coordinate of the walker
    /// * `direction`: The direction of the walker
    /// * `walker_type`: The type of walker
    ///
    /// #examples
    /// ```
    /// use shoko_rocket_rust::{World, Direction, WalkerType};
    /// let mut world = World::new();
    /// world.create_walker(0, 0, Direction::Right, WalkerType::Mouse);
    /// ```
    pub fn create_walker(&mut self, x: usize, y: usize, direction: Direction, walker_type: WalkerType) {
        assert!(x < WORLD_WIDTH);
        assert!(y < WORLD_HEIGHT);

        let walker = Walker::new(x as i8, y as i8, direction, walker_type);
        match walker.get_type() {
            WalkerType::Mouse => self.mice.push(walker),
            WalkerType::Cat => self.cats.push(walker)
        }
    }

    /// Advances the simulation state of the world
    /// * Mice move forward 3 units
    /// * Cats move forward 2 units
    /// Each frame check mouse/cat collisions
    /// * Mice are killed by cats, causing defeat
    /// On reaching a new grid, walkers check holes/rockets
    /// * Cats are killed by holes
    /// * Mice are killed by holes, causing defeat
    /// * Mice are rescued by rockets
    /// * Cats destroy rockets, causing defeat
    /// On reaching a new grid, walkers check arrow
    /// * Mice are directed by the arrow
    /// * Cats are directed by arrows, and if turned around, consume the arrow
    /// On reaching a new grid, walkers check walls
    /// On all mice rescued, victory
    pub fn tick(&mut self) {
        // 1. Advance mice and cats
        for walker in self.mice.iter_mut() {
            if walker.walk() == WalkResult::NewSquare {
                // 2. Check holes, rockets
                // 3. Check arrows
                // 4. Check walls
                World::check_walls(&self.data, walker);
            }
            // 5. Check cat/mouse collisions
        }
        for walker in self.cats.iter_mut() {
            walker.walk();
        }
    }

    /// Handles collisions with walls
    /// * If not blocked, keep going straight
    /// * If blocked and able to turn right, turn right
    /// * If blocked and unable to turn right, turn left
    /// * If blocked and unable to turn left or right, return around
    /// * If blocked all around, keep going straight. This will in practice not happen due to
    ///   level design
    fn check_walls(wall_data: &[u8; 199], walker: &mut Walker) {
        let x = walker.get_x().integer_part() as usize;
        let y = walker.get_y().integer_part() as usize;
        let direction = walker.get_direction();

        // Priority list of directions to travel. The first clear direction will be used
        let candidate_directions = [
            direction,
            direction.turn_right(),
            direction.turn_left(),
            direction.turn_around()
        ];

        for candidate_direction in candidate_directions {
            if !World::get_wall_static(wall_data, x, y, candidate_direction) {
                walker.set_direction(candidate_direction);
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// GIVEN A row of cells and a direction
    /// WHEN we calculate the wall index and bitmask
    /// THEN the correct values are returned
    #[test]
    fn index_and_masm() {
        assert_eq!((0, 0b00000001), World::get_wrapped_wall_index_and_mask(0, 0, Direction::Up));
        assert_eq!((0, 0b00000100), World::get_wrapped_wall_index_and_mask(1, 0, Direction::Up));
        assert_eq!((0, 0b00010000), World::get_wrapped_wall_index_and_mask(2, 0, Direction::Up));
        assert_eq!((0, 0b01000000), World::get_wrapped_wall_index_and_mask(3, 0, Direction::Up));
        assert_eq!((1, 0b00000001), World::get_wrapped_wall_index_and_mask(4, 0, Direction::Up));

        assert_eq!((0, 0b00000010), World::get_wrapped_wall_index_and_mask(0, 0, Direction::Left));
        assert_eq!((0, 0b00001000), World::get_wrapped_wall_index_and_mask(1, 0, Direction::Left));
        assert_eq!((0, 0b00100000), World::get_wrapped_wall_index_and_mask(2, 0, Direction::Left));
        assert_eq!((0, 0b10000000), World::get_wrapped_wall_index_and_mask(3, 0, Direction::Left));
        assert_eq!((1, 0b00000010), World::get_wrapped_wall_index_and_mask(4, 0, Direction::Left));

        // Down walls are the top wall of the cell below, increasing the index by 3
        assert_eq!((3, 0b00000001), World::get_wrapped_wall_index_and_mask(0, 0, Direction::Down));
        assert_eq!((3, 0b00000100), World::get_wrapped_wall_index_and_mask(1, 0, Direction::Down));
        assert_eq!((3, 0b00010000), World::get_wrapped_wall_index_and_mask(2, 0, Direction::Down));
        assert_eq!((3, 0b01000000), World::get_wrapped_wall_index_and_mask(3, 0, Direction::Down));
        assert_eq!((4, 0b00000001), World::get_wrapped_wall_index_and_mask(4, 0, Direction::Down));

        // Right walls are the left wall of the cell to the left, shifting the mask and increasing
        // the index by 1 for every 4th eleemnt
        assert_eq!((0, 0b00001000), World::get_wrapped_wall_index_and_mask(0, 0, Direction::Right));
        assert_eq!((0, 0b00100000), World::get_wrapped_wall_index_and_mask(1, 0, Direction::Right));
        assert_eq!((0, 0b10000000), World::get_wrapped_wall_index_and_mask(2, 0, Direction::Right));
        assert_eq!((1, 0b00000010), World::get_wrapped_wall_index_and_mask(3, 0, Direction::Right));
        assert_eq!((1, 0b00001000), World::get_wrapped_wall_index_and_mask(4, 0, Direction::Right));
    }

    /// GIVEN a newly created world
    /// WHEN we check the walls
    /// THEN the walls are around the edge only
    #[test]
    fn new_creates_outline() {
        let world = World::new();

        // Top and bottom set
        for x in 0..WORLD_WIDTH {
            assert_eq!(true, world.get_wall(x, 0, Direction::Up));
            assert_eq!(true, world.get_wall(x, WORLD_HEIGHT - 1, Direction::Down));
        }

        // Left and right set
        for y in 0..WORLD_HEIGHT {
            assert_eq!(true, world.get_wall(0, y, Direction::Left));
            assert_eq!(true, world.get_wall(WORLD_WIDTH - 1, y, Direction::Right));
        }

        // Everything else not set
        for y in 1..WORLD_HEIGHT - 1 {
            for x in 1..WORLD_WIDTH - 1 {
                assert_eq!(false, world.get_wall(x, y, Direction::Up));
                assert_eq!(false, world.get_wall(x, y, Direction::Down));
                assert_eq!(false, world.get_wall(x, y, Direction::Left));
                assert_eq!(false, world.get_wall(x, y, Direction::Right));
            }
        }
    }

    /// GIVEN a wall directly ahead
    /// WHEN a walker walks towards/along/away from the wall
    /// THEN the correct turns (right/none/none) are made
    #[test]
    fn walker_wall_straight() {
        let world = World::new();
        let mut walker_up = Walker::new(4, 0, Direction::Up, WalkerType::Mouse);
        let mut walker_down = Walker::new(4, 0, Direction::Down, WalkerType::Mouse);
        let mut walker_left = Walker::new(4, 0, Direction::Left, WalkerType::Mouse);
        let mut walker_right = Walker::new(4, 0, Direction::Right, WalkerType::Mouse);

        World::check_walls(&world.data, &mut walker_up);
        World::check_walls(&world.data, &mut walker_down);
        World::check_walls(&world.data, &mut walker_left);
        World::check_walls(&world.data, &mut walker_right);

        assert_eq!(Direction::Right, walker_up.get_direction());
        assert_eq!(Direction::Down, walker_down.get_direction());
        assert_eq!(Direction::Left, walker_left.get_direction());
        assert_eq!(Direction::Right, walker_right.get_direction());
    }

    /// GIVEN a wall directly ahead and to the right
    /// WHEN the walker walks towards/left/right/away from the wall
    /// THEN the correct turns (left/none/down/none) are made
    #[test]
    fn walker_wall_forced_left() {
        let world = World::new();
        let mut walker_up = Walker::new(11, 0, Direction::Up, WalkerType::Mouse);
        let mut walker_down = Walker::new(11, 0, Direction::Down, WalkerType::Mouse);
        let mut walker_left = Walker::new(11, 0, Direction::Left, WalkerType::Mouse);
        let mut walker_right = Walker::new(11, 0, Direction::Right, WalkerType::Mouse);

        World::check_walls(&world.data, &mut walker_up);
        World::check_walls(&world.data, &mut walker_down);
        World::check_walls(&world.data, &mut walker_left);
        World::check_walls(&world.data, &mut walker_right);

        assert_eq!(Direction::Left, walker_up.get_direction());
        assert_eq!(Direction::Down, walker_down.get_direction());
        assert_eq!(Direction::Left, walker_left.get_direction());
        assert_eq!(Direction::Down, walker_right.get_direction());
    }

    /// GIVEN a wall in a U shape (directly ahead and to the left and right)
    /// WHEN the walker walks towards/left/right/away from the wall
    /// THEN the correct turns (around/left/right/none) are made
    #[test]
    fn walker_wall_u_shape() {
        let mut world = World::new();
        world.set_wall(0, 0, Direction::Right, true);
        let mut walker_up = Walker::new(0, 0, Direction::Up, WalkerType::Mouse);
        let mut walker_down = Walker::new(0, 0, Direction::Down, WalkerType::Mouse);
        let mut walker_left = Walker::new(0, 0, Direction::Left, WalkerType::Mouse);
        let mut walker_right = Walker::new(0, 0, Direction::Right, WalkerType::Mouse);

        World::check_walls(&world.data, &mut walker_up);
        World::check_walls(&world.data, &mut walker_down);
        World::check_walls(&world.data, &mut walker_left);
        World::check_walls(&world.data, &mut walker_right);

        assert_eq!(Direction::Down, walker_up.get_direction());
        assert_eq!(Direction::Down, walker_down.get_direction());
        assert_eq!(Direction::Down, walker_left.get_direction());
        assert_eq!(Direction::Down, walker_right.get_direction());
    }
}
