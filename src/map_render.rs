
use crossterm::cursor;

use crate::{display::{color_format_char, ColorDisplay, HasBackColor, HasTextColor, HasTextDisplay, ANSI_DEFAULT_TEXT_COLOR}, ground_map, player::player, std_out, tile_map};


pub const MAP_LENGTH: usize = 26;
pub const MAP_HEIGHT: usize = 26;

/// Prints out the given map
pub fn display_map() {
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_LENGTH {
            // left and right sides of the tile rendering
            let left: String;
            let right: String;
            
            // the color data for the ground at current point
            let ground_colors = ground_map()[y][x].ansi_back_colors();

            // tile at current point
            let tile = &tile_map()[y][x];
            left = get_left_tile_display((x, y), tile, ground_colors);

            right = get_right_tile_display(tile, ground_colors);


            // print the left and right sides of the tile
            print!("{}{}", left, right);
        }
        // new line every row
        ignorant_queue!(std_out, cursor::MoveDown(1));
        ignorant_execute!(std_out, cursor::MoveLeft(MAP_LENGTH as u16 * 2));
    }
}


/// Used in display_map() to determine the left end of rendering
fn get_left_tile_display<T: HasTextColor + HasBackColor + HasTextDisplay>(point: (usize, usize), tile: &T, ground_colors: ColorDisplay) -> String {
    let left: String;
    // display player layer if present
    if point == player().position {
        // text color is default terminal color
        let text_color = ANSI_DEFAULT_TEXT_COLOR.to_owned();
        /* back color is ground color
        * if there is no back color, the program should not have compiled in the first place so it panics
        */
        let back_color = match ground_colors.back_color_left {
            Some((r, g, b)) => format!("48;2;{};{};{}", r, g, b),
            None => unreachable!("ground color is unnassigned"),
        };
        
        left = color_format_char(
            text_color,
            back_color,
            player().text_display.character_left.unwrap_or(' ')
        );
    // display tile layer when player is not present
    } else {
        // text color is either defined tile text color or default terminal color
        let text_color = match tile.ansi_text_colors().text_color_left {
            Some((r, g, b)) => format!("38;2;{};{};{};", r, g, b),
            None => ANSI_DEFAULT_TEXT_COLOR.to_owned(),
        };
        /* back color is either tile back color or ground color
        * if there is no back color, the program should not have compiled in the first place so it panics
        */
        let back_color = match (tile.ansi_back_colors().back_color_left, ground_colors.back_color_left) {
            (Some((r, g, b)), _) | (None, Some((r, g, b))) => format!("48;2;{};{};{}", r, g, b),
            (None, None) => unreachable!("ground color is unnassigned"),
        };

        left = color_format_char(
            text_color,
            back_color,
            tile.text_display().character_left.unwrap_or(' '),
        );
    }
    left
}

/// Used in map_display() to determine the right end of rendering
fn get_right_tile_display<T: HasTextColor + HasBackColor + HasTextDisplay>(tile: &T, ground_colors: ColorDisplay) -> String {
    // text color is either tile text color or the tile text color on the left side of the tile or default terminal color
    let text_color = match (tile.ansi_text_colors().text_color_right, tile.ansi_text_colors().text_color_left) {
        (Some((r, g, b)), _) | (None, Some((r, g, b))) => format!("38;2;{};{};{};", r, g, b),
        (None, None) => ANSI_DEFAULT_TEXT_COLOR.to_owned(),
    };
    /* back color is either tile back color or ground color
    * if there is no back color, the program should not have compiled in the first place so it panics
    */
    let back_color = match (tile.ansi_back_colors().back_color_right, ground_colors.back_color_right, ground_colors.back_color_left)  {
        (Some((r, g, b)), _, _) | (None, Some((r, g, b)), _) | (None, None, Some((r, g, b)))  => format!("48;2;{};{};{}", r, g, b),
        (None, None, None) => unreachable!("ground color is unnassigned"),
    };
    
    // chatacter is either the tile right side character or a ' '
    let character = match tile.text_display().character_right {
        Some(character) => character,
        None => ' ',
    };

    color_format_char(
        text_color,
        back_color,
        character,
    )
}