use crate::display::{ColorDisplay, TextDisplay};

#[derive(Clone, Debug)]
pub struct Inventory {
    pub items: Vec<ItemStack>,
}

#[derive(Clone, Copy, Debug)]
pub struct ItemStack {
    pub item_type: u16,
    pub count: u32,
}

#[derive(Debug)]
pub struct ItemType {
    pub identifier: u16,
    pub text_display: TextDisplay,
    pub color_display: ColorDisplay
}