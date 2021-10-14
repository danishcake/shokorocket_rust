use super::direction::Direction;

pub const WORLD_WIDTH: usize = 12;
pub const WORLD_HEIGHT: usize = 9;
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
///   uint8_t: 2 entity; // 0 -> empty, 1 -> mouse, 2 -> cat, 3 -> rocket
///   uint8_t: 2 entity_direction; // 0 -> up, 1 -> down, 2 -> left, 3 -> right
///   uint8_t: 1 arrow; // 0 -> empty, 1 -> arrow
///   uint8_t: 2 arrow_direction; // 0 -> up, 1 -> down, 2 -> left, 3 -> right
///   uint8_t: 1 unused;
/// }
/// For a total of 64 + 27 + 108 bytes = 199 bytes
/// TODO: More constants!
pub struct World {
    data: [u8; 199],
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
        let mut world = World { data: [0; 199] };

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
    ///
    /// #examples
    /// ```
    /// use shoko_rocket_rust::{World, Direction};
    /// let mut world = World::new();
    /// world.set_wall(0, 0, Direction::Down, true);
    /// assert!(world.get_wall(0, 0, Direction::Down));
    /// ```
    fn get_wrapped_wall_index_and_mask(
        &self,
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

        let (wall_index, mask) = self.get_wrapped_wall_index_and_mask(x, y, direction);
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
        assert!(x < WORLD_WIDTH);
        assert!(y < WORLD_HEIGHT);

        let (wall_index, mask) = self.get_wrapped_wall_index_and_mask(x, y, direction);
        let byte = &self.data[HEADER_SIZE + wall_index];
        return *byte & mask == mask;
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
        let world = World::new();
        assert_eq!((0, 0b00000001), world.get_wrapped_wall_index_and_mask(0, 0, Direction::Up));
        assert_eq!((0, 0b00000100), world.get_wrapped_wall_index_and_mask(1, 0, Direction::Up));
        assert_eq!((0, 0b00010000), world.get_wrapped_wall_index_and_mask(2, 0, Direction::Up));
        assert_eq!((0, 0b01000000), world.get_wrapped_wall_index_and_mask(3, 0, Direction::Up));
        assert_eq!((1, 0b00000001), world.get_wrapped_wall_index_and_mask(4, 0, Direction::Up));

        assert_eq!((0, 0b00000010), world.get_wrapped_wall_index_and_mask(0, 0, Direction::Left));
        assert_eq!((0, 0b00001000), world.get_wrapped_wall_index_and_mask(1, 0, Direction::Left));
        assert_eq!((0, 0b00100000), world.get_wrapped_wall_index_and_mask(2, 0, Direction::Left));
        assert_eq!((0, 0b10000000), world.get_wrapped_wall_index_and_mask(3, 0, Direction::Left));
        assert_eq!((1, 0b00000010), world.get_wrapped_wall_index_and_mask(4, 0, Direction::Left));

        // Down walls are the top wall of the cell below, increasing the index by 3
        assert_eq!((3, 0b00000001), world.get_wrapped_wall_index_and_mask(0, 0, Direction::Down));
        assert_eq!((3, 0b00000100), world.get_wrapped_wall_index_and_mask(1, 0, Direction::Down));
        assert_eq!((3, 0b00010000), world.get_wrapped_wall_index_and_mask(2, 0, Direction::Down));
        assert_eq!((3, 0b01000000), world.get_wrapped_wall_index_and_mask(3, 0, Direction::Down));
        assert_eq!((4, 0b00000001), world.get_wrapped_wall_index_and_mask(4, 0, Direction::Down));

        // Right walls are the left wall of the cell to the left, shifting the mask and increasing
        // the index by 1 for every 4th eleemnt
        assert_eq!((0, 0b00001000), world.get_wrapped_wall_index_and_mask(0, 0, Direction::Right));
        assert_eq!((0, 0b00100000), world.get_wrapped_wall_index_and_mask(1, 0, Direction::Right));
        assert_eq!((0, 0b10000000), world.get_wrapped_wall_index_and_mask(2, 0, Direction::Right));
        assert_eq!((1, 0b00000010), world.get_wrapped_wall_index_and_mask(3, 0, Direction::Right));
        assert_eq!((1, 0b00001000), world.get_wrapped_wall_index_and_mask(4, 0, Direction::Right));
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
}
