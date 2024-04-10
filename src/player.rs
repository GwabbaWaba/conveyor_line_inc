use once_cell::sync::Lazy;

use crate::{display::{ColorDisplay, TextDisplay}, map_height, map_width};

pub static mut PLAYER: once_cell::sync::Lazy<Player> = Lazy::<Player>::new(||Player::new((0, 0)));
/// safe unsafe action lolz
pub fn player() -> &'static mut Player { unsafe { &mut PLAYER } }

/// Represents the player
pub struct Player {
    pub position: (usize, usize),
    pub text_display: TextDisplay,
    pub color_display: ColorDisplay,
}

impl Player {
    pub fn new(position: (usize, usize)) -> Self {
        Self {
            position: position,
            text_display: TextDisplay {
                character_left: Some('☺'),
                character_right: None
            },
            color_display: ColorDisplay {
                text_color_left: None,
                back_color_left: None,
                text_color_right: None,
                back_color_right: None
            }
        }
    }
}