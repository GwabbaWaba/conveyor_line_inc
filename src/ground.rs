use crate::{display::{ColorDisplay, HasBackColor, HasTextColor, HasTextDisplay, TextDisplay}, game_data_dump};

/// Represents the ground
#[derive(Clone, Copy)]
pub struct Ground {
    pub ground_type: u16,
    pub text_display: TextDisplay,
    pub color_display: ColorDisplay
}

impl Ground {
    pub fn new(ground_type: u16) -> Self {
        let tile_from_map = game_data_dump().ground_types.get(&ground_type).unwrap();

        Self {
            ground_type,
            text_display: tile_from_map.text_display,
            color_display: tile_from_map.color_display
        }
    }
}

/// The types of ground that Ground can represent
#[derive(Debug)]
pub struct GroundType {
    pub identifier: u16,
    pub text_display: TextDisplay,
    pub color_display: ColorDisplay,
    pub solid: bool,
    pub world_gen_weight: f64
}

// HasColor implements
impl HasTextColor for Ground {
    fn ansi_text_colors (&self) -> ColorDisplay {
        self.color_display
    }
}

impl HasBackColor for Ground {
    fn ansi_back_colors (&self) -> ColorDisplay {
        self.color_display
    }
}

impl HasTextDisplay for Ground {
    fn text_display (&self) -> TextDisplay {
        self.text_display
    }
}