use crate::{
    display::*, game_data_dump
};

/// Represents a tile, which is a solid or non-solid element of the map
#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub tile_type: u16,
    pub text_display: TextDisplay,
    pub color_display: ColorDisplay,
}

impl Tile {
    pub fn new_unchecked(tile_type: u16) -> Self {
        let tile_from_map = game_data_dump().tile_types.get(&tile_type).unwrap();

        Self {
            tile_type,
            text_display: tile_from_map.text_display,
            color_display: tile_from_map.color_display
        }
    }
    pub fn new(tile_type: u16) -> Option<Self> {
        match game_data_dump().tile_types.get(&tile_type) {
            Some(tile_from_map) => {
                Some(
                    Self {
                        tile_type,
                        text_display: tile_from_map.text_display,
                        color_display: tile_from_map.color_display
                    }
                )
            },
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PositionTracker {
    pub id: u32,
    pub position: Option<(usize, usize)>,
}

/// The types of tiles a Tile can represent
#[derive(Debug)]
pub struct TileType {
    pub identifier: u16,
    pub name: String,
    pub text_display: TextDisplay,
    pub color_display: ColorDisplay,
    pub solid: bool,
    pub world_gen_weight: f64,
}

// HasColor implements
impl HasTextColor for Tile {
    fn ansi_text_colors (&self) -> ColorDisplay {
        self.color_display
    }
}

impl HasBackColor for Tile {
    fn ansi_back_colors (&self) -> ColorDisplay {
        self.color_display
    }
}

impl HasTextDisplay for Tile {
    fn text_display (&self) -> TextDisplay {
        self.text_display
    }
}