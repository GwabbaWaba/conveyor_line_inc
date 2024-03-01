
use crossterm::{cursor, event::{KeyCode, KeyEvent, KeyModifiers}, QueueableCommand};

use crate::{map_render::MAP_HEIGHT, print_input_text, std_out, wipe_input};

pub const MAX_INPUT_CHARS: usize = 35;
pub static mut BEHIND_PRINT_AMOUNT: usize = MAX_INPUT_CHARS;

/// safe unsafe action lolz
pub fn behind_print_amount() ->  usize { unsafe { BEHIND_PRINT_AMOUNT } }

/// not that much to type, but enough for a function
/// safe unsafe action lolz
pub fn increment_behind_print_amount(incrementation: usize, is_decrement: bool) {
    unsafe { 
        BEHIND_PRINT_AMOUNT = if is_decrement {
            if incrementation <= BEHIND_PRINT_AMOUNT {
                BEHIND_PRINT_AMOUNT - incrementation
            } else {
                0
            }
        } else {
            (BEHIND_PRINT_AMOUNT + incrementation).min(MAX_INPUT_CHARS)
        };
    };
}

/// handles what happens in accordance to input key event
/// returns an input when enter is pressed
pub fn key_output(event: KeyEvent, chars_behind_cursor: &mut Vec<char>, chars_ahead_cursor: &mut Vec<char>) -> Option<String> {
    let _ = std_out().queue(cursor::RestorePosition);
    let mut current_input = None;

    match event.code {
        // normal typing
        KeyCode::Char(c) => {
            chars_behind_cursor.push(c);

            increment_behind_print_amount(1, false);

            print_input_text(chars_behind_cursor, chars_ahead_cursor);
        },
        // normal backspace funtionality
        KeyCode::Backspace if event.modifiers == KeyModifiers::CONTROL => {
            let looking_for_space = chars_behind_cursor.last() == Some(&' ');
            if looking_for_space {
                loop {
                    if chars_behind_cursor.last() != Some(&' ') || chars_behind_cursor.is_empty() { break; }
                    chars_behind_cursor.pop();
                }
            } else {
                loop {
                    if chars_behind_cursor.last() == Some(&' ') || chars_behind_cursor.is_empty() { break; }
                    chars_behind_cursor.pop();
                }
            }
            print_input_text(chars_behind_cursor, chars_ahead_cursor);
        },
        KeyCode::Backspace => {
            chars_behind_cursor.pop();

            print_input_text(chars_behind_cursor, chars_ahead_cursor);
        },
        // normal delete functionality
        KeyCode::Delete if event.modifiers == KeyModifiers::CONTROL => {
            let looking_for_space = chars_ahead_cursor.last() == Some(&' ');
            if looking_for_space {
                loop {
                    if chars_ahead_cursor.last() != Some(&' ') || chars_ahead_cursor.is_empty() { break; }
                    chars_ahead_cursor.pop();
                }
            } else {
                loop {
                    if chars_ahead_cursor.last() == Some(&' ') || chars_ahead_cursor.is_empty() { break; }
                    chars_ahead_cursor.pop();
                }
            }
            print_input_text(chars_behind_cursor, chars_ahead_cursor);
        },
        KeyCode::Delete => {
            chars_ahead_cursor.pop();

            print_input_text(chars_behind_cursor, chars_ahead_cursor);
        },
        // navigation keys
        KeyCode::Left => {
            match chars_behind_cursor.pop() {
                Some(c) => {
                    chars_ahead_cursor.push(c);

                    increment_behind_print_amount(1, true);

                    print_input_text(chars_behind_cursor, chars_ahead_cursor);
                },
                _ => {},
            }
        },
        KeyCode::Right => {
            match chars_ahead_cursor.pop() {
                Some(c) => {
                    chars_behind_cursor.push(c);

                    increment_behind_print_amount(1, false);

                    print_input_text(chars_behind_cursor, chars_ahead_cursor);
                },
                _ => {},
            }
        },
        // submit command
        KeyCode::Enter => {
            ignorant_queue!(std_out, cursor::MoveTo(27, MAP_HEIGHT as u16 + 6));
            ignorant_execute!(std_out, cursor::SavePosition);

            wipe_input();

            // putting all the input chars into chars_behind_cursor
            chars_ahead_cursor.reverse();
            chars_behind_cursor.append(chars_ahead_cursor);

            current_input = Some(chars_behind_cursor.iter().collect());

            chars_behind_cursor.clear();
            chars_ahead_cursor.clear();
            unsafe { BEHIND_PRINT_AMOUNT = MAX_INPUT_CHARS };
        },
        _ => {} 
    };

    current_input
}