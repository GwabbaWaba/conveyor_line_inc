
use std::io::{self, Write};

use crossterm::{cursor, execute};

use crate::{display::{color_format_char, ColorDisplay, HasBackColor, HasTextColor, HasTextDisplay, ANSI_DEFAULT_TEXT_COLOR}, ground_map, map_height, map_width, player::player, terminal, tile_map, MAP_HEIGHT, MAP_WIDTH, STDOUT_REF};


pub static mut MAP_DISPLAY_WIDTH: Option<usize> = None;
pub static mut MAP_DISPLAY_HEIGHT: Option<usize> = None;
pub fn map_display_width() -> usize {
    unsafe { MAP_DISPLAY_WIDTH.unwrap() }
}
pub fn map_display_height() -> usize {
    unsafe { MAP_DISPLAY_HEIGHT.unwrap() }
}

/// Prints out the given map
pub fn display_map() -> Result<(), io::Error> {
    let mut terminal_row = 3;
    let terminal_col = 81;
    execute!(
        terminal().backend_mut(),
        cursor::MoveTo(terminal_col, terminal_row)
    )?;

    let horizontal_middle = player().position.0;
    let vertical_middle = player().position.1;
    let horizontal_dist = map_display_width() / 2;
    let vertical_dist = map_display_height() / 2;

    let left = isize::clamp(horizontal_middle as isize - horizontal_dist as isize, 0, map_width() as isize) as usize;
    let right= usize::clamp(horizontal_middle + horizontal_dist, 0, map_width());
    let top = isize::clamp(vertical_middle as isize - vertical_dist as isize, 0, map_width() as isize) as usize;
    let bottom= usize::clamp(vertical_middle + vertical_dist, 0, map_width());

    //let mut locked_stdout = unsafe { (*STDOUT_REF).lock() };
    for y in top..bottom {
        for x in left..right {
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
            //let _ = locked_stdout.write(format!("{}{}", left, right).as_bytes());
            print!("{}{}", left, right);
        }

        terminal_row += 1;
        execute!(
            //locked_stdout,
            terminal().backend_mut(),
            cursor::MoveTo(terminal_col, terminal_row)
        )?;
    }

    Ok(())
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