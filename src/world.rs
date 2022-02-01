use crate::{
    walker::{self, WalkResult},
    Direction, TileType, Walker, WalkerState, WalkerType, WorldStateChange,
};
use arrayvec::ArrayVec;
use core::convert::TryInto;

/// The width of the world
pub const WORLD_WIDTH: usize = 12;
/// The height of the world
pub const WORLD_HEIGHT: usize = 9;
/// The maximum number of walkers, if all squares were filled with walkers
const MAX_WALKERS: usize = WORLD_WIDTH * WORLD_HEIGHT;
/// The maximum number of tiles, if all squares were filled with tiles
const MAX_TILES: usize = WORLD_WIDTH * WORLD_HEIGHT;
/// The size of the header (e.g. the name and author of the map)
const HEADER_SIZE: usize = 64;
/// The size of the wall block
const WALL_BLOCK_SIZE: usize = 27;
/// The masks used to pack the left walls. There are four walls packed into each byte
const LEFT_WALL_MASK: [u8; 4] = [0b00000010, 0b00001000, 0b00100000, 0b10000000];
/// The masks uses to pack the top walls.
const TOP_WALL_MASK: [u8; 4] = [0b00000001, 0b00000100, 0b00010000, 0b01000000];

const ENTITY_TYPE_MASK: u8 = 0b11100000u8;
const ENTITY_DIRECTION_MASK: u8 = 0b00011000u8;
const ARROW_PRESENT_MASK: u8 = 0b00000100u8;
const ARROW_DIRECTION_MASK: u8 = 0b00000011u8;

const ENTITY_TYPE_EMPTY: u8 = 0b00000000;
const ENTITY_TYPE_MOUSE: u8 = 0b00100000;
const ENTITY_TYPE_CAT: u8 = 0b01000000;
const ENTITY_TYPE_ROCKET: u8 = 0b01100000;
const ENTITY_TYPE_HOLE: u8 = 0b10000000;

const ENTITY_DIRECTION_UP: u8 = 0b00000000;
const ENTITY_DIRECTION_DOWN: u8 = 0b00001000;
const ENTITY_DIRECTION_LEFT: u8 = 0b00010000;
const ENTITY_DIRECTION_RIGHT: u8 = 0b00011000;

const ARROW_DIRECTION_UP: u8 = 0b00000000;
const ARROW_DIRECTION_DOWN: u8 = 0b00000001;
const ARROW_DIRECTION_LEFT: u8 = 0b00000010;
const ARROW_DIRECTION_RIGHT: u8 = 0b00000011;

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
    tiles: [TileType; MAX_TILES],
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
            tiles: [TileType::Empty; MAX_TILES],
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
    fn get_wrapped_wall_index_and_mask(x: usize, y: usize, direction: Direction) -> (usize, u8) {
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
    /// Return value:
    /// True if the walker was added. False if it could not be added for some reason
    /// (e.g. overlaps an existing walker)
    ///
    /// #examples
    /// ```
    /// use shoko_rocket_rust::{World, Direction, WalkerType};
    /// let mut world = World::new();
    /// world.create_walker(0, 0, Direction::Right, WalkerType::Mouse);
    /// ```
    pub fn create_walker(
        &mut self,
        x: usize,
        y: usize,
        direction: Direction,
        walker_type: WalkerType,
    ) -> bool {
        assert!(x < WORLD_WIDTH);
        assert!(y < WORLD_HEIGHT);

        // Check if this tile is already occupied
        let walker_data = &mut self.data[HEADER_SIZE + WALL_BLOCK_SIZE..];
        let walker_byte = &mut walker_data[y * WORLD_WIDTH + x];

        if (*walker_byte & ENTITY_TYPE_MASK) != ENTITY_TYPE_EMPTY {
            return false;
        }

        let walker = Walker::new(x as i8, y as i8, direction, walker_type);
        match walker.get_type() {
            WalkerType::Mouse => self.mice.push(walker),
            WalkerType::Cat => self.cats.push(walker),
        }

        // Create the corresponding walker in the data array
        *walker_byte = *walker_byte & (ARROW_PRESENT_MASK | ARROW_DIRECTION_MASK);
        *walker_byte = *walker_byte
            | match direction {
                Direction::Up => ENTITY_DIRECTION_UP,
                Direction::Down => ENTITY_DIRECTION_DOWN,
                Direction::Left => ENTITY_DIRECTION_LEFT,
                Direction::Right => ENTITY_DIRECTION_RIGHT,
            };
        *walker_byte = *walker_byte
            | match walker_type {
                WalkerType::Mouse => ENTITY_TYPE_MOUSE,
                WalkerType::Cat => ENTITY_TYPE_CAT,
            };

        true
    }

    /// Sets the arrow at the specified location
    ///
    /// Arguments:
    /// * `x`: The x coordinate to check. Must be in range 0-11
    /// * `y`: The y coordinate to check. Must be in range 0-8
    /// * `arrow_type`: The type of arrow to set
    pub fn set_arrow(&mut self, x: usize, y: usize, tile_type: TileType) {
        // TODO: Check tile is empty of rockets/holes
        // TODO: Handle stock of spare arrows
        World::set_tile_static(&mut self.tiles, x, y, tile_type)
    }

    /// Sets the tile at the specified location
    ///
    /// Arguments:
    /// * `x`: The x coordinate to check. Must be in range 0-11
    /// * `y`: The y coordinate to check. Must be in range 0-8
    /// * `arrow_type`: The type of arrow to set
    pub fn set_tile(&mut self, x: usize, y: usize, tile_type: TileType) {
        World::set_tile_static(&mut self.tiles, x, y, tile_type)
    }

    /// Sets the tile at the specified location. No checking is performed to
    /// ensure that the tile is empty - use the set_arrow method for this
    ///
    /// Arguments:
    /// * `tiles`: The internal representation of the tiles
    /// * `x`: The x coordinate to check. Must be in range 0-11
    /// * `y`: The y coordinate to check. Must be in range 0-8
    /// * `tile_type`: The type of tile to set
    fn set_tile_static(tiles: &mut [TileType; MAX_TILES], x: usize, y: usize, tile_type: TileType) {
        assert!(x < WORLD_WIDTH);
        assert!(y < WORLD_HEIGHT);

        tiles[y * WORLD_WIDTH + x] = tile_type;
    }

    /// Gets the arrow at the specified location
    ///
    /// Arguments:
    /// * `x`: The x coordinate to check. Must be in range 0-11
    /// * `y`: The y coordinate to check. Must be in range 0-8
    ///
    /// Return value:
    /// The type of arrow present at the specified coordinate
    ///
    /// #examples
    /// ```
    /// use shoko_rocket_rust::World;
    /// let mut world = World::new();
    /// world.get_arrow(0, 0);
    /// ```
    pub fn get_arrow(&self, x: usize, y: usize) -> TileType {
        return World::get_arrow_static(&self.tiles, x, y);
    }

    /// Gets the arrow at the specified location
    ///
    /// Arguments:
    /// * `arrows`: The internal representation of the arrows
    /// * `x`: The x coordinate to check. Must be in range 0-11
    /// * `y`: The y coordinate to check. Must be in range 0-8
    ///
    /// Return value:
    /// The type of arrow present at the specified coordinate
    fn get_arrow_static(arrows: &[TileType; MAX_TILES], x: usize, y: usize) -> TileType {
        assert!(x < WORLD_WIDTH);
        assert!(y < WORLD_HEIGHT);

        return arrows[y * WORLD_WIDTH + x];
    }

    /// Resets the state to that specified in the serialised form
    pub fn reset() {}

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
    pub fn tick(&mut self) -> WorldStateChange {
        let mut world_state_change = WorldStateChange::NoChange;

        // 1. Advance mice and cats
        let all_walkers = self.mice.iter_mut().chain(self.cats.iter_mut());
        for walker in all_walkers {
            if walker.walk() == WalkResult::NewSquare {
                // 2. Check holes, rockets
                World::check_rockets_and_holes(&mut self.tiles, walker);

                // 3. Check arrows
                World::check_arrows(&mut self.tiles, walker);

                // 4. Check walls
                World::check_walls(&self.data, walker);
            }
        }

        // 5. Check cat/mouse collisions

        // 6. Check if any mice died to holes, or any cats were rescued
        if self
            .mice
            .iter_mut()
            .any(|walker| walker.get_state() == WalkerState::Dead)
            || self
                .cats
                .iter_mut()
                .any(|walker| walker.get_state() == WalkerState::Rescued)
        {
            world_state_change = WorldStateChange::Lose;
        }

        // 7. Check if all mice have been rescued
        if !self.mice.is_empty()
            && self
                .mice
                .iter()
                .all(|walker| (*walker).get_state() == WalkerState::Rescued)
        {
            world_state_change = WorldStateChange::Win;
        }

        // 8. Remove dead/rescued walkers
        self.mice
            .retain(|walker| walker.get_state() == WalkerState::Alive);
        self.cats
            .retain(|walker| walker.get_state() == WalkerState::Alive);

        // 9. Return the new world state for the user to handle
        world_state_change
    }

    /// Handles collisions with walls
    /// * If not blocked, keep going straight
    /// * If blocked and able to turn right, turn right
    /// * If blocked and unable to turn right, turn left
    /// * If blocked and unable to turn left or right, return around
    /// * If blocked all around, keep going straight. This will in practice not happen due to
    ///   level design
    ///
    /// Arguments:
    /// * `wall_data`: The internal representation of the walls
    /// * `walker`: The Walker to check
    fn check_walls(wall_data: &[u8; 199], walker: &mut Walker) {
        let (x, y) =
            (walker.get_x().integer_part() as usize, walker.get_y().integer_part() as usize);
        let direction = walker.get_direction();

        // Priority list of directions to travel. The first clear direction will be used
        let candidate_directions = [
            direction,
            direction.turn_right(),
            direction.turn_left(),
            direction.turn_around(),
        ];

        for candidate_direction in candidate_directions {
            if !World::get_wall_static(wall_data, x, y, candidate_direction) {
                walker.set_direction(candidate_direction);
                break;
            }
        }
    }

    /// Handles collisions between arrows and walkers
    /// * Walkers are turned to face the arrow direction
    /// * If a cat is turned 180 degrees then the arrow is diminished
    ///
    /// Arguments:
    /// * `tile`: The tiles
    /// * `walker`: The Walker to check
    fn check_arrows(tiles: &mut [TileType; MAX_TILES], walker: &mut Walker) {
        let (x, y) =
            (walker.get_x().integer_part() as usize, walker.get_y().integer_part() as usize);
        let arrow = World::get_arrow_static(tiles, x, y);
        let arrow_direction = arrow.try_into();
        match arrow_direction {
            Ok(direction) => {
                // If turning around a cat, diminish the arrow
                if walker.get_type() == WalkerType::Cat
                    && walker.get_direction().turn_around() == direction
                {
                    World::set_tile_static(tiles, x, y, arrow.diminish());
                }
                walker.set_direction(direction);
            }
            Err(_) => {
                // Indicates an empty/hole/rocket tile, which is fine
            }
        };
    }

    /// Handles collisions between holes/rockets and walkers
    /// * Holes kill everything
    /// * Rockets rescue mice
    /// * Cats kill rockets
    /// * Mouse death changes world state to a loss
    /// * Rocket death changes world state to a loss
    /// * Rescue of all mice changes world state to a win
    ///
    /// Arguments:
    /// * `tiles`: The tiles
    /// * `walker`: The Walker to check
    fn check_rockets_and_holes(tiles: &mut [TileType; MAX_TILES], walker: &mut Walker) {
        let (x, y) =
            (walker.get_x().integer_part() as usize, walker.get_y().integer_part() as usize);
        let tile = tiles[y * WORLD_WIDTH + x];

        match tile {
            TileType::Hole => walker.kill(),
            TileType::Rocket => walker.rescue(),
            _ => {}
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
    fn index_and_mask() {
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

    /// GIVEN an empty world
    /// WHEN a walker is created
    /// THEN the walker is added to the correct walker array
    /// AND the walker is added to the backing data
    #[test]
    fn walker_creation() {
        let mut world = World::new();
        world.create_walker(1, 1, Direction::Down, WalkerType::Mouse);
        world.create_walker(4, 4, Direction::Up, WalkerType::Cat);

        // The walkers exist
        assert_eq!(1, world.mice.len());
        assert_eq!(1, world.cats.len());

        // The source data has been updated to include the walker
        let walker_data = &world.data[HEADER_SIZE + WALL_BLOCK_SIZE..];
        // Walkers are packed into one byte, so let's find them
        assert_eq!(ENTITY_TYPE_MOUSE, walker_data[WORLD_WIDTH * 1 + 1] & ENTITY_TYPE_MASK);
        assert_eq!(ENTITY_DIRECTION_DOWN, walker_data[WORLD_WIDTH * 1 + 1] & ENTITY_DIRECTION_MASK);
        assert_eq!(ENTITY_TYPE_CAT, walker_data[WORLD_WIDTH * 4 + 4] & ENTITY_TYPE_MASK);
        assert_eq!(ENTITY_DIRECTION_UP, walker_data[WORLD_WIDTH * 4 + 4] & ENTITY_DIRECTION_MASK);
    }

    /// GIVEN an existing walker
    /// WHEN a new walker is created at the same spot
    /// THEN the operation is rejected
    #[test]
    fn walkers_cannot_be_created_in_same_square() {
        let mut world = World::new();
        let created_1 = world.create_walker(0, 0, Direction::Down, WalkerType::Cat);
        let created_2 = world.create_walker(0, 0, Direction::Down, WalkerType::Cat);

        assert_eq!(true, created_1);
        assert_eq!(false, created_2);
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

    /// GIVEN an arrow with no nearby walls
    /// WHEN a walker encounters that arrow at right angles
    /// THEN the walker turns in that direction
    #[test]
    fn walker_arrow_right_angle_turns() {
        let mut world = World::new();
        world.set_arrow(2, 2, TileType::Up);
        world.set_arrow(4, 3, TileType::Down);
        world.set_arrow(6, 4, TileType::Left);
        world.set_arrow(8, 5, TileType::Right);

        // Walkers approaching up arrow from left/right
        world.create_walker(1, 2, Direction::Right, WalkerType::Mouse);
        world.create_walker(3, 2, Direction::Left, WalkerType::Mouse);

        // Walkers approaching down arrow from left/right
        world.create_walker(3, 3, Direction::Right, WalkerType::Mouse);
        world.create_walker(5, 3, Direction::Left, WalkerType::Mouse);

        // Walkers approaching left arrow from top/bottom
        world.create_walker(6, 3, Direction::Down, WalkerType::Mouse);
        world.create_walker(6, 5, Direction::Up, WalkerType::Mouse);

        // Walkers approaching right arrow from top/bottom
        world.create_walker(8, 4, Direction::Down, WalkerType::Mouse);
        world.create_walker(8, 6, Direction::Up, WalkerType::Mouse);

        // Run for the time is takes for a mouse to move just before one square
        for _ in 0..59 {
            world.tick();
        }

        // Walkers should not have changed their directions yet
        assert_eq!(Direction::Right, world.mice[0].get_direction());
        assert_eq!(Direction::Left, world.mice[1].get_direction());
        assert_eq!(Direction::Right, world.mice[2].get_direction());
        assert_eq!(Direction::Left, world.mice[3].get_direction());
        assert_eq!(Direction::Down, world.mice[4].get_direction());
        assert_eq!(Direction::Up, world.mice[5].get_direction());
        assert_eq!(Direction::Down, world.mice[6].get_direction());
        assert_eq!(Direction::Up, world.mice[7].get_direction());

        // Tick the final time required for mice to walk one square
        world.tick();

        // Walkers should now have been directed by the arrows
        assert_eq!(Direction::Up, world.mice[0].get_direction());
        assert_eq!(Direction::Up, world.mice[1].get_direction());
        assert_eq!(Direction::Down, world.mice[2].get_direction());
        assert_eq!(Direction::Down, world.mice[3].get_direction());
        assert_eq!(Direction::Left, world.mice[4].get_direction());
        assert_eq!(Direction::Left, world.mice[5].get_direction());
        assert_eq!(Direction::Right, world.mice[6].get_direction());
        assert_eq!(Direction::Right, world.mice[7].get_direction());
    }

    /// GIVEN an arrow with no nearby walls
    /// WHEN a mouse encounters an arrow in opposite direction
    /// THEN the mouse is turned around
    /// AND the arrow is unchanged
    #[test]
    fn mice_do_not_diminish_arrows_if_opposed() {
        let mut world = World::new();
        world.set_arrow(4, 3, TileType::Up);
        world.set_arrow(4, 5, TileType::Down);
        world.set_arrow(3, 4, TileType::Left);
        world.set_arrow(5, 4, TileType::Right);

        world.create_walker(4, 2, Direction::Down, WalkerType::Mouse);
        world.create_walker(4, 6, Direction::Up, WalkerType::Mouse);
        world.create_walker(2, 4, Direction::Right, WalkerType::Mouse);
        world.create_walker(6, 4, Direction::Left, WalkerType::Mouse);

        // Run for the time is takes for a mouse to move just before one square
        for _ in 0..59 {
            world.tick();
        }

        // The walkers have not yet encountered the arrows and have their original direction
        assert_eq!(Direction::Down, world.mice[0].get_direction());
        assert_eq!(Direction::Up, world.mice[1].get_direction());
        assert_eq!(Direction::Right, world.mice[2].get_direction());
        assert_eq!(Direction::Left, world.mice[3].get_direction());

        // Tick the final time required for mice to walk one square
        world.tick();

        // Walkers should now have been directed by the arrows
        assert_eq!(Direction::Up, world.mice[0].get_direction());
        assert_eq!(Direction::Down, world.mice[1].get_direction());
        assert_eq!(Direction::Left, world.mice[2].get_direction());
        assert_eq!(Direction::Right, world.mice[3].get_direction());

        // The arrows are unchanged.
        assert_eq!(TileType::Up, world.get_arrow(4, 3));
        assert_eq!(TileType::Down, world.get_arrow(4, 5));
        assert_eq!(TileType::Left, world.get_arrow(3, 4));
        assert_eq!(TileType::Right, world.get_arrow(5, 4));
    }

    /// GIVEN an arrow with no nearby walls
    /// WHEN a cat encounters an arrow in opposite direction
    /// THEN the cat is turned around
    /// AND the arrow is diminished
    #[test]
    fn cats_diminish_arrows_if_opposed() {
        let mut world = World::new();
        world.set_arrow(4, 4, TileType::Down);
        world.create_walker(4, 5, Direction::Up, WalkerType::Cat);

        // Walk cat to edge of arrow. Cats move at 2/3 speed of a mouse, so 90 ticks required
        for _ in 0..89 {
            world.tick();
        }

        // The arrow and cat are unchanged
        assert_eq!(TileType::Down, world.get_arrow(4, 4));
        assert_eq!(Direction::Up, world.cats[0].get_direction());

        // Tick the final time required for cats to walk one square
        world.tick();

        // The arrow is diminished and the cat turned around
        assert_eq!(TileType::DownHalf, world.get_arrow(4, 4));
        assert_eq!(Direction::Down, world.cats[0].get_direction());
    }

    /// GIVEN an arrow with no nearby walls
    /// WHEN three cats encounter an arrow in opposite direction
    /// THEN the first two cats are turned around
    /// AND the last two cats continues
    /// AND the arrow is removed
    #[test]
    fn double_diminish_removes_arrow() {
        let mut world = World::new();
        world.set_arrow(4, 4, TileType::Down);
        world.create_walker(4, 5, Direction::Up, WalkerType::Cat);
        world.create_walker(4, 6, Direction::Up, WalkerType::Cat);
        world.create_walker(4, 7, Direction::Up, WalkerType::Cat);

        // Walk first cat into the arrow
        for _ in 0..90 {
            world.tick();
        }

        // The arrow is dimished and first cat turned around
        assert_eq!(TileType::DownHalf, world.get_arrow(4, 4));
        assert_eq!(Direction::Down, world.cats[0].get_direction());
        assert_eq!(Direction::Up, world.cats[1].get_direction());
        assert_eq!(Direction::Up, world.cats[2].get_direction());

        // Walk second cat into the arrow
        for _ in 0..90 {
            world.tick();
        }

        // The arrow is removed and the second cat turned around
        assert_eq!(TileType::Empty, world.get_arrow(4, 4));
        assert_eq!(Direction::Down, world.cats[0].get_direction());
        assert_eq!(Direction::Down, world.cats[1].get_direction());
        assert_eq!(Direction::Up, world.cats[2].get_direction());

        // Walk third cat into the 'arrow'
        for _ in 0..90 {
            world.tick();
        }

        // The arrow is still gone and the cat continues on his way
        assert_eq!(TileType::Empty, world.get_arrow(4, 4));
        assert_eq!(Direction::Down, world.cats[0].get_direction());
        assert_eq!(Direction::Down, world.cats[1].get_direction());
        assert_eq!(Direction::Up, world.cats[2].get_direction());
    }

    /// GIVEN an arrow against a wall
    /// WHEN a walker is turned into the wall by the arrow
    /// THEN the normal wall rule are applied, causing the walker to turn
    #[test]
    fn walker_turn_into_wall() {
        let mut world = World::new();
        world.set_arrow(4, 0, TileType::Up);
        world.create_walker(5, 0, Direction::Left, WalkerType::Mouse);

        // Walk the walker into the arrow
        for _ in 0..60 {
            world.tick();
        }

        // The walker is turned up by the arrow, then right by the wall
        assert_eq!(Direction::Right, world.mice[0].get_direction());
    }

    /// GIVEN a newly created world
    /// WHEN an arrow is set
    /// THEN reading that arrow back returns the correct value
    #[test]
    fn arrow_get_set() {
        let mut world = World::new();

        assert_eq!(TileType::Empty, world.get_arrow(0, 0));
        world.set_arrow(0, 0, TileType::Right);
        assert_eq!(TileType::Right, world.get_arrow(0, 0));

        assert_eq!(TileType::Empty, world.get_arrow(7, 3));
        world.set_arrow(7, 3, TileType::Right);
        assert_eq!(TileType::Right, world.get_arrow(7, 3));
    }

    /// GIVEN a world with a hole one unit to the right of a cat
    /// WHEN the cat walks into the hole
    /// THEN the cat is killed
    /// AND the world state does not change
    #[test]
    fn holes_kill_cats() {
        let mut world = World::new();

        world.set_tile(1, 0, TileType::Hole);
        world.create_walker(0, 0, Direction::Right, WalkerType::Cat);

        for _ in 0..89 {
            world.tick();
        }

        // After 89 ticks the cat is still alive
        assert_eq!(1, world.cats.len());

        // The 90th tick kills the cat as it falls into the hole
        let world_state_change = world.tick();

        assert_eq!(WorldStateChange::NoChange, world_state_change);
        assert_eq!(0, world.cats.len());
    }

    /// GIVEN a world with a hole one unit to the right of a mouse
    /// WHEN the mouse walks into the hole
    /// THEN the mouse is killed
    /// AND the world state changes to lose
    #[test]
    fn holes_kill_mice_and_cause_loss() {
        let mut world = World::new();

        world.set_tile(1, 0, TileType::Hole);
        world.create_walker(0, 0, Direction::Right, WalkerType::Mouse);

        for _ in 0..59 {
            world.tick();
        }

        // After 59 ticks the mouse is still alive
        assert_eq!(1, world.mice.len());

        // The 90th tick kills the mouse as it falls into the hole
        let world_state_change = world.tick();

        assert_eq!(WorldStateChange::Lose, world_state_change);
        assert_eq!(0, world.mice.len());
    }

    /// GIVEN a world with a rocket one unit to the right of a cat
    /// WHEN the cat walks into the rocket
    /// THEN the cat is rescued
    /// AND the world state changes to lose
    #[test]
    fn rockets_rescue_cats_and_cause_loss() {
        let mut world = World::new();

        world.set_tile(1, 0, TileType::Rocket);
        world.create_walker(0, 0, Direction::Right, WalkerType::Cat);

        for _ in 0..89 {
            world.tick();
        }

        // After 89 ticks the cat is still alive
        assert_eq!(1, world.cats.len());

        // The 90th tick rescues the cat as it hits the rocket
        let world_state_change = world.tick();

        assert_eq!(0, world.cats.len());
        assert_eq!(WorldStateChange::Lose, world_state_change);
    }

    /// GIVEN a world with a rocket one unit to the right of a mouse
    /// WHEN the mouse walks into the rocket
    /// THEN the mouse is rescued
    /// AND the world state changes to win
    #[test]
    fn rockets_rescue_mice_and_cause_win() {
        let mut world = World::new();

        world.set_tile(1, 0, TileType::Rocket);
        world.create_walker(0, 0, Direction::Right, WalkerType::Mouse);

        for _ in 0..59 {
            world.tick();
        }

        // After 59 ticks the mouse is still alive
        assert_eq!(1, world.mice.len());

        // The 90th tick rescues the mouse as it falls into the hole
        let world_state_change = world.tick();

        assert_eq!(WorldStateChange::Win, world_state_change);
        assert_eq!(0, world.mice.len());
    }
}
