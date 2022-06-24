extern crate proc_macro;
use debugless_unwrap::*;
use itertools::Itertools;
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use std::vec::Vec;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Error, LitStr, Token,
};

/// The width of the world
const WORLD_WIDTH: usize = 12;
/// The height of the world
const WORLD_HEIGHT: usize = 9;
const MAP_NAME_SIZE: usize = 32;
const MAP_NAME_OFFSET: usize = 0;
/// The map name field
const MAP_AUTHOR_SIZE: usize = 32;
const MAP_AUTHOR_OFFSET: usize = MAP_NAME_OFFSET + MAP_NAME_SIZE;
/// The size of the wall block
const WALL_BLOCK_SIZE: usize = 27;
const WALL_BLOCK_OFFSET: usize = MAP_AUTHOR_OFFSET + MAP_AUTHOR_SIZE;
/// The size and offset of the tile block
const TILE_BLOCK_SIZE: usize = 108;
const TILE_BLOCK_OFFSET: usize = WALL_BLOCK_OFFSET + WALL_BLOCK_SIZE;
/// The masks used to pack the left walls. There are four walls packed into each byte
const LEFT_WALL_MASK: [u8; 4] = [0b00000010, 0b00001000, 0b00100000, 0b10000000];
/// The masks uses to pack the top walls.
const TOP_WALL_MASK: [u8; 4] = [0b00000001, 0b00000100, 0b00010000, 0b01000000];

// The types of entity in the tile block. Leave the top three bits zero to indicate empty
const ENTITY_TYPE_MOUSE: u8 = 0b00100000;
const ENTITY_TYPE_CAT: u8 = 0b01000000;
const ENTITY_TYPE_ROCKET: u8 = 0b01100000;
const ENTITY_TYPE_HOLE: u8 = 0b10000000;

// The directions of entities in the tile block
const ENTITY_DIRECTION_UP: u8 = 0b00000000;
const ENTITY_DIRECTION_DOWN: u8 = 0b00001000;
const ENTITY_DIRECTION_LEFT: u8 = 0b00010000;
const ENTITY_DIRECTION_RIGHT: u8 = 0b00011000;

/// The values to use for packed arrow directions
const ARROW_DIRECTION_UP: u8 = 0b00000000;
const ARROW_DIRECTION_DOWN: u8 = 0b00000001;
const ARROW_DIRECTION_LEFT: u8 = 0b00000010;
const ARROW_DIRECTION_RIGHT: u8 = 0b00000011;

/// Gets the index into the wall data of a particular wall, and the mask required to
/// get/set it.
///
/// Arguments:
/// * `x`: The x coordinate to set. Must be in range 0-11
/// * `y`: The y coordinate to set. Must be in range 0-8
///
/// Return value:
/// A tuple containing the wall index and the mask required for left walls
const fn get_l_wall_index_and_mask(x: usize, y: usize) -> (usize, u8) {
    assert!(x < WORLD_WIDTH);
    assert!(y < WORLD_HEIGHT);
    ((y * WORLD_WIDTH + x) / 4, LEFT_WALL_MASK[x & 0x03])
}

/// Gets the index into the wall data of a particular wall, and the mask required to
/// get/set it.
///
/// Arguments:
/// * `x`: The x coordinate to set. Must be in range 0-11
/// * `y`: The y coordinate to set. Must be in range 0-8
///
/// Return value:
/// A tuple containing the wall index and the mask required for top walls
const fn get_t_wall_index_and_mask(x: usize, y: usize) -> (usize, u8) {
    assert!(x < WORLD_WIDTH);
    assert!(y < WORLD_HEIGHT);
    ((y * WORLD_WIDTH + x) / 4, TOP_WALL_MASK[x & 0x03])
}

struct PuzzleMacroInput {
    pub name: LitStr,
    pub author: LitStr,
    pub body: [LitStr; 19],
}

impl Parse for PuzzleMacroInput {
    // Parses the input to the seq macro
    // This will consist of three strings with two commas
    fn parse(input: ParseStream) -> Result<Self, Error> {
        let name: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;
        let author: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        // TODO: Improve error reporting here if there are an incorrect number of lines
        let mut body: Vec<LitStr> = Vec::new();
        for _row_index in 0..19 {
            let row: LitStr = input.parse()?;
            body.push(row);
        }

        Ok(PuzzleMacroInput {
            name,
            author,
            body: body.try_into().debugless_unwrap(),
        })
    }
}

/// Checks a string has a valid length, and copies it to the output if valid
/// Arguments:
/// * `string_token`: The string token to check
/// * `max_size`: The maximum length in bytes of the string
/// * `output`: The slice to write to. Should be same size as max_size
fn check_and_add_string(
    string_token: &LitStr,
    max_size: usize,
    output: &mut [u8],
) -> Option<TokenStream> {
    let value = string_token.value();

    // Check the input value is valid
    if value.len() == 0 {
        return Some(
            quote_spanned! {
                string_token.span() => compile_error!("String cannot be empty")
            }
            .into(),
        );
    }
    // TODO: Figure out how to format an error message here
    if value.len() > max_size {
        return Some(
            quote_spanned! {
                string_token.span() => compile_error!("String is too long")
            }
            .into(),
        );
    }

    // Copy to output
    output[0..value.len()].copy_from_slice(value.as_bytes());
    None
}

/// Checks that the top/bottom lines are consistent
/// OK:
/// ┌────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┐
/// └────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┘
/// Not OK:
/// ┌────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┐
/// └    ┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┴────┘
/// Arguments:
/// * `top`: The top line
/// * `max_size`: The maximum length in bytes of the string
/// * `output`: The slice to write to. Should be same size as max_size
fn check_top_bottom_consistency(top: &LitStr, bottom: &LitStr) -> Option<TokenStream> {
    // Check the top-bottom markers are consistent
    let top_value = top.value();
    let bottom_value = bottom.value();

    for col_index in 0..WORLD_WIDTH {
        // Check that the top and bottom are the same
        if top_value.chars().nth(col_index * 5 + 1) != bottom_value.chars().nth(col_index * 5 + 1) {
            return Some(
                quote_spanned! {
                    top.span() => compile_error!("Top and bottom walls must be consistent")
                }
                .into(),
            );
        }
    }

    None
}

/// Checks that all rows are 61 characters long
/// Arguments:
/// * `rows`: All rows in the input
fn check_line_lengths(rows: &[LitStr]) -> Option<TokenStream> {
    for row in rows {
        if row.value().chars().count() != 61 {
            return Some(
                quote_spanned! {
                    row.span() => compile_error!("Line must be 61 characters long")
                }
                .into(),
            );
        }
    }
    None
}

/// Checks that within a row, the four top characters in a cell are consistent
/// Arguments:
/// * `rows`: The even rows
fn check_cell_consistency(rows: &Vec<&LitStr>) -> Option<TokenStream> {
    for row in rows {
        let row_value = row.value();
        for col_index in 0..WORLD_WIDTH {
            for intracell_index in 1..5 {
                if row_value.chars().nth(col_index * 5 + 1)
                    != row_value.chars().nth(col_index * 5 + intracell_index)
                {
                    return Some(
                        quote_spanned! {
                            row.span() => compile_error!("All top walls within a cell must be the same")
                        }
                        .into(),
                    );
                }
            }
        }
    }

    None
}

/// Checks that rows start and end with the same character. Should be called with odd rows only
/// Arguments:
/// * `rows`: Rows to check
fn check_left_right_consistency(rows: &Vec<&LitStr>) -> Option<TokenStream> {
    for row in rows {
        let row_value = row.value();
        // Check the wraparound markers are consistent L-R
        if row_value.chars().last().unwrap() != row_value.chars().next().unwrap() {
            return Some(
                quote_spanned! {
                    row.span() => compile_error!("Left and right walls must be consistent")
                }
                .into(),
            );
        }
    }

    None
}

/// Extracts the top walls from the even rows
/// Arguments:
/// * `rows`: The even rows, excluding the last one
/// * `output`: The wall block in the output
fn extract_top_walls(rows: &Vec<&LitStr>, output: &mut [u8]) -> Option<TokenStream> {
    for (row_index, line_literal) in rows.iter().enumerate() {
        let line = line_literal.value();

        for col_index in 0..WORLD_WIDTH {
            match line.chars().nth(col_index * 5 + 1).unwrap() {
                '─' => {
                    // Set the appropriate bit in the output
                    let (wall_index, mask) = get_t_wall_index_and_mask(col_index, row_index);
                    let byte = &mut output[wall_index];
                    *byte = *byte | mask;
                }
                ' ' => { /* Do nothing */ }
                '-' => {
                    return Some(quote_spanned! {
                            line_literal.span() => compile_error!("Unexpected top wall - must be ' ' or '─'. Found '-' - look closely!")
                        }.into());
                }
                _ => {
                    return Some(quote_spanned! {
                            line_literal.span() => compile_error!("Unexpected top wall - must be ' ' or '─'")
                        }.into());
                }
            };
        }
    }

    None
}

/// Extracts the left walls from the odd rows
/// Arguments:
/// * `rows`: The odd rows
/// * `output`: The wall block in the output
fn extract_left_walls(rows: &Vec<&LitStr>, output: &mut [u8]) -> Option<TokenStream> {
    for (row_index, line_literal) in rows.iter().enumerate() {
        let line = line_literal.value();

        // Set appropriate bits for left walls
        for col_index in 0..WORLD_WIDTH {
            match line.chars().nth(col_index * 5).unwrap() {
                '│' => {
                    let (wall_index, mask) = get_l_wall_index_and_mask(col_index, row_index);
                    let byte = &mut output[wall_index];
                    *byte = *byte | mask;
                }
                ' ' => { /* No wall, do nothing */ }
                '|' => {
                    return Some(quote_spanned! {
                        line_literal.span() => compile_error!("Unexpected left wall - must be ' ' or '│'. Found '|' - look closely!")
                    }.into());
                }
                _ => {
                    return Some(quote_spanned! {
                        line_literal.span() => compile_error!("Unexpected left wall - must be ' ' or '│'")
                    }.into());
                }
            }
        }
    }

    None
}

/// Extracts the solution arrows from the odd rows
/// Arguments:
/// * `rows`: The odd rows
/// * `output`: The tile block in the output
fn extract_arrows(rows: &Vec<&LitStr>, output: &mut [u8]) -> Option<TokenStream> {
    for (row_index, line_literal) in rows.iter().enumerate() {
        let line = line_literal.value();

        // Set appropriate bits for left walls
        for col_index in 0..WORLD_WIDTH {
            let arrow_index = col_index + row_index * WORLD_WIDTH;
            let byte = &mut output[arrow_index];

            match (
                line.chars().nth(col_index * 5 + 3).unwrap(),
                line.chars().nth(col_index * 5 + 4).unwrap(),
            ) {
                ('A', '<') => {
                    *byte = *byte | ARROW_DIRECTION_LEFT;
                }
                ('A', '>') => {
                    *byte = *byte | ARROW_DIRECTION_RIGHT;
                }
                ('A', '^') => {
                    *byte = *byte | ARROW_DIRECTION_UP;
                }
                ('A', 'v') => {
                    *byte = *byte | ARROW_DIRECTION_DOWN;
                }
                (' ', ' ') => { /* No arrows and no direction, do nothing */ }
                ('A', _) => {
                    return Some(quote_spanned! {
                        line_literal.span() => compile_error!("If an arrow is specified with 'A' then it must be followed by one of <>^v")
                    }.into());
                }
                (_, _) => {
                    return Some(quote_spanned! {
                        line_literal.span() => compile_error!("Unexpected characters in arrow cell")
                    }.into());
                }
            }
        }
    }

    None
}

/// Extracts the tiles from the odd rows
/// Arguments:
/// * `rows`: The odd rows
/// * `output`: The tile block in the output
fn extract_tiles(rows: &Vec<&LitStr>, output: &mut [u8]) -> Option<TokenStream> {
    for (row_index, line_literal) in rows.iter().enumerate() {
        let line = line_literal.value();

        // Set appropriate bits for left walls
        for col_index in 0..WORLD_WIDTH {
            let tile_index = col_index + row_index * WORLD_WIDTH;
            let byte = &mut output[tile_index];

            match (
                line.chars().nth(col_index * 5 + 1).unwrap(),
                line.chars().nth(col_index * 5 + 2).unwrap(),
            ) {
                // TBD: There's got to be a way to DRY this up
                ('M', '<') => {
                    *byte = *byte | ENTITY_TYPE_MOUSE | ENTITY_DIRECTION_LEFT;
                }
                ('M', '>') => {
                    *byte = *byte | ENTITY_TYPE_MOUSE | ENTITY_DIRECTION_RIGHT;
                }
                ('M', '^') => {
                    *byte = *byte | ENTITY_TYPE_MOUSE | ENTITY_DIRECTION_UP;
                }
                ('M', 'v') => {
                    *byte = *byte | ENTITY_TYPE_MOUSE | ENTITY_DIRECTION_DOWN;
                }
                ('C', '<') => {
                    *byte = *byte | ENTITY_TYPE_CAT | ENTITY_DIRECTION_LEFT;
                }
                ('C', '>') => {
                    *byte = *byte | ENTITY_TYPE_CAT | ENTITY_DIRECTION_RIGHT;
                }
                ('C', '^') => {
                    *byte = *byte | ENTITY_TYPE_CAT | ENTITY_DIRECTION_UP;
                }
                ('C', 'v') => {
                    *byte = *byte | ENTITY_TYPE_CAT | ENTITY_DIRECTION_DOWN;
                }
                ('R', ' ') => {
                    *byte = *byte | ENTITY_TYPE_ROCKET;
                }
                ('H', ' ') => {
                    *byte = *byte | ENTITY_TYPE_HOLE;
                }
                ('M' | 'C', _) => {
                    return Some(quote_spanned! {
                        line_literal.span() => compile_error!("If a mouse or cat is specified then it must be followed by one of <>^v")
                    }.into());
                }
                ('R' | 'H', _) => {
                    return Some(quote_spanned! {
                        line_literal.span() => compile_error!("If a rocket or hole is specified then it must be followed by a blank space")
                    }.into());
                }
                (' ', ' ') => { /* No tile and no direction, do nothing */ }
                (_, _) => {
                    return Some(quote_spanned! {
                        line_literal.span() => compile_error!("Unexpected characters in tile cell")
                    }.into());
                }
            }
        }
    }

    None
}

/// Generates a serialised map based on a graphical representation
/// of the solution.
/// Walls are drawn using box drawing characters. Only the left/top walls
/// are actually parsed - other walls are ignored. Where there are multiple walls for top,
/// only the left-most is used.
///
/// The solution requires the overlay of arrows and walkers, each with directions.
/// This is done by having four-wide cells, with the left half for the walker,
/// and the right half for the arrow.
///
/// Wall symbols:
/// ┘ ┐ ┌ └ ┼ ─ ├ ┤ ┴ ┬ │
/// Arrow symbols:
/// < > ^ v
/// Walker/rocket/hole symbols:
/// M C R H
/// ┌────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┐
/// │<M< │    │    │    │    │    │    │    │    │    │    │    │
/// ├────┴────┼────┴────┴────┼────┼────┼────┼────┼────┼────┼────┤
/// │<M<  vC> │
/// │         │
/// │         │H    R
/// └────┬────┼──── ──── ──── ──── ──── ──── ──── ──── ──── ────
///
///
/// Usage:
/// let map = puzzle!("Name", "Author", "....")
#[proc_macro]
pub fn puzzle(tokens: TokenStream) -> TokenStream {
    // Uncomment to see what the macro is invoked with
    // dbg!(&tokens);

    let input = parse_macro_input!(tokens as PuzzleMacroInput);
    let mut output = [0u8; 199];

    // Omit the last row from even_rows, as this is purely for looks
    let even_rows: Vec<&LitStr> = input.body.iter().step_by(2).dropping_back(1).collect();
    let odd_rows: Vec<&LitStr> = input.body.iter().skip(1).step_by(2).collect();

    // Validate the name and author, then copy into output
    if let Some(x) = check_and_add_string(
        &input.name,
        MAP_NAME_SIZE,
        &mut output[MAP_NAME_OFFSET..MAP_NAME_OFFSET + MAP_NAME_SIZE],
    ) {
        return x;
    }
    if let Some(x) = check_and_add_string(
        &input.author,
        MAP_AUTHOR_SIZE,
        &mut output[MAP_AUTHOR_OFFSET..MAP_AUTHOR_OFFSET + MAP_AUTHOR_SIZE],
    ) {
        return x;
    }

    // Check top-bottom wraparound edges for consistency
    if let Some(x) = check_top_bottom_consistency(&input.body[0], &input.body[18]) {
        return x;
    }

    // Check that within each cell, all top walls are the same
    if let Some(x) = check_cell_consistency(&even_rows) {
        return x;
    }

    // Check line lengths
    if let Some(x) = check_line_lengths(&input.body) {
        return x;
    }

    // Check left-right wraparound edges for consistency
    if let Some(x) = check_left_right_consistency(&odd_rows) {
        return x;
    }

    // Read the top walls
    if let Some(x) = extract_top_walls(
        &even_rows,
        &mut output[WALL_BLOCK_OFFSET..WALL_BLOCK_OFFSET + WALL_BLOCK_SIZE],
    ) {
        return x;
    }

    // Read the left walls
    if let Some(x) = extract_left_walls(
        &odd_rows,
        &mut output[WALL_BLOCK_OFFSET..WALL_BLOCK_OFFSET + WALL_BLOCK_SIZE],
    ) {
        return x;
    }

    // Read the arrows
    if let Some(x) = extract_arrows(
        &odd_rows,
        &mut output[TILE_BLOCK_OFFSET..TILE_BLOCK_OFFSET + TILE_BLOCK_SIZE],
    ) {
        return x;
    }

    // Read the tiles
    if let Some(x) = extract_tiles(
        &odd_rows,
        &mut output[TILE_BLOCK_OFFSET..TILE_BLOCK_OFFSET + TILE_BLOCK_SIZE],
    ) {
        return x;
    }

    // Generate the list of bytes to output
    let bytes = output.iter().map(|b| {
        quote! { #b }
    });

    let tokens = quote! {
        [
            #(#bytes),*
        ]
    };
    // Uncomment this line to see what macro invocation outputs
    // eprintln!("TOKENS: {}", tokens);

    tokens.into()
}
