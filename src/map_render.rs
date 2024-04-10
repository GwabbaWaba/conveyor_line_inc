
use std::{alloc::{dealloc, Layout}, io::{self, Write}, iter::{Chain, Zip}, ops::Range, ptr::addr_of, sync::mpsc};

use crossterm::{cursor, execute, queue, terminal};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use tui::{backend::CrosstermBackend, Terminal};

use crate::{display::{color_format_char, ColorDisplay, HasBackColor, HasTextColor, HasTextDisplay, ANSI_DEFAULT_TEXT_COLOR}, ground_map, map_height, map_width, player::player, terminal, tile_map, write_to_debug, Writer, MAP_HEIGHT, MAP_WIDTH, STDOUT_REF};


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

    let map_width = map_width() as isize;
    let map_height = map_height() as isize;

    let horizontal_middle = player().position.0 as isize;
    let vertical_middle = player().position.1 as isize;
    let horizontal_dist = (map_display_width() / 2) as isize;
    let vertical_dist = (map_display_height() / 2) as isize;

    let left = horizontal_middle - horizontal_dist;
    let right = horizontal_middle + horizontal_dist;
    let top = vertical_middle - vertical_dist;
    let bottom = vertical_middle + vertical_dist;

    let left_deadzone = left;
    let top_deadzone = top;

    let right_deadzone = right;
    let bottom_deadzone = bottom;

    let left = left.max(0);
    let top = top.max(0);
    let right = right.min(map_width);
    let bottom = bottom.min(map_height);

    let mut locked_stdout = unsafe { (*STDOUT_REF).lock() };
    for y in (top_deadzone..0).chain(top..bottom).chain(map_height..bottom_deadzone) {
        for x in (left_deadzone..0).chain(left..right).chain(map_width..right_deadzone) {
            // left and right sides of the tile rendering
            let left: String;
            let right: String;
            
            if y >= map_height || x >= map_width || y < 0 || x < 0  {
                left = get_farlands_tile_display();
                right = get_farlands_tile_display();

                let _ = locked_stdout.write(format!("{}{}", left, right).as_bytes());
            } else {
                let (x, y) = (x as usize, y as usize);
                // the color data for the ground at current point
                let ground_colors = ground_map()[y][x].ansi_back_colors();
    
                // tile at current point
                let tile = &tile_map()[y][x];

                left = get_left_tile_display((x, y), tile, ground_colors);
                right = get_right_tile_display(tile, ground_colors);
                
                let _ = locked_stdout.write(format!("{}{}", left, right).as_bytes());
            }
            
        }

        terminal_row += 1;
        execute!(
            locked_stdout,
            cursor::MoveTo(terminal_col, terminal_row)
        )?;
    }

    Ok(())
}

static mut FARLANDIAN_BYTE_CALLS: u8 = 0;

fn farlandian_byte<T>(source: Box<*mut T>) -> u8 {
    unsafe { FARLANDIAN_BYTE_CALLS = FARLANDIAN_BYTE_CALLS.overflowing_add(1).0; };
    (*source as *const u8 as u8).overflowing_add(unsafe{ FARLANDIAN_BYTE_CALLS }).0
}
fn farlandian_byte_alt<T>(source: Box<*mut T>) -> u8 {
    unsafe { FARLANDIAN_BYTE_CALLS = FARLANDIAN_BYTE_CALLS.overflowing_mul(3).0; };
    (*source as *const u8 as u8).overflowing_add(unsafe{ FARLANDIAN_BYTE_CALLS }).0
}

fn get_farlands_tile_display() -> String {
    let s: String;

    let c = farlandian_byte_alt(Box::new(unsafe {STDOUT_REF})) as char;
    let c = match c {
        ' '..='~' | '\u{A0}'..='\u{AC}' | '\u{AE}'..='\u{FF}' => { c },
        _ => { char::REPLACEMENT_CHARACTER }
    };

    let text_color = format!("38;2;{};{};{};", 
        farlandian_byte(Box::new(unsafe {STDOUT_REF})), 
        unsafe { std::ptr::read_volatile(STDOUT_REF as *const u8) },
        std::ptr::addr_of!(c) as u8,
    );
    let back_color = format!("48;2;{};{};{}", 
        farlandian_byte_alt(Box::new(std::ptr::addr_of!(text_color) as *mut u8)), 
        // this read is kinda iffy
        unsafe { std::ptr::read_volatile(addr_of!(c)) as u8 },
        farlandian_byte(Box::new(std::ptr::null_mut::<u8>())), 
    );

    s = color_format_char(
        text_color,
        back_color,
        c,
    );

    s
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