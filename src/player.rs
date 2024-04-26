use once_cell::sync::Lazy;

use crate::{display::{ColorDisplay, TextDisplay}, map_height, map_width, Point};

pub static mut PLAYER: once_cell::sync::Lazy<Player> = Lazy::<Player>::new(||Player::new(Point{x: 0, y: 0}));
/// safe unsafe action lolz
pub fn player() -> &'static mut Player { unsafe { &mut PLAYER } }

/// Represents the player
pub struct Player {
    pub position: Point,
    pub text_display: TextDisplay,
    pub color_display: ColorDisplay,
}

impl Player {
    pub fn new(position: Point) -> Self {
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