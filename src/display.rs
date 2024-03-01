pub const ANSI_DEFAULT_TEXT_COLOR: &str = "38;0;";

/// text and back color must be formatted as:
/// "(38/48);(0/2)(;###;###;###/)"
pub fn color_format_char(text_color: String, back_color: String, char: char) -> String {
    format!("\u{001B}[{}{}m{}\u{001B}[0m", text_color, back_color, char)
}

auto_builder! {
    /// 2 characters which represents how to display something
    #[derive(Clone, Copy, Debug)]
    pub TextDisplay
    pub TextDisplayBuilder
    pub character_left: Option<char>,
    pub character_right: Option<char>
}

auto_builder! {
    /// 2 colors which represent how to display something
    #[derive(Clone, Copy, Debug)]
    pub ColorDisplay
    pub ColorDisplayBuilder
    pub text_color_left: Option<(u8, u8, u8)>,
    pub back_color_left: Option<(u8, u8, u8)>,
    pub text_color_right: Option<(u8, u8, u8)>,
    pub back_color_right: Option<(u8, u8, u8)>
}

impl TextDisplayBuilder {
    pub fn new() -> Self {
        Self {
            character_left: None,
            character_right: None,
        }
    }

    pub fn character_left(&mut self, character_left: char) -> &mut Self {
        self.character_left = Some(character_left);
        self
    }

    pub fn character_right(&mut self, character_right: char) -> &mut Self {
        self.character_right = Some(character_right);
        self
    }

    pub fn finalize(&self) -> TextDisplay {
        TextDisplay { character_left: self.character_left, character_right: self.character_right }
    }
}

impl ColorDisplayBuilder {
    pub fn new() -> Self {
        Self {
            text_color_left: None,
            back_color_left: None,
            text_color_right: None,
            back_color_right: None,
        }
    }

    pub fn text_color_left(&mut self, text_color_left: (u8, u8, u8)) -> &mut Self {
        self.text_color_left = Some(text_color_left);
        self
    }

    pub fn back_color_left(&mut self, back_color_left: (u8, u8, u8)) -> &mut Self {
        self.back_color_left = Some(back_color_left);
        self
    }

    pub fn text_color_right(&mut self, text_color_right: (u8, u8, u8)) -> &mut Self {
        self.text_color_right = Some(text_color_right);
        self
    }

    pub fn back_color_right(&mut self, back_color_right: (u8, u8, u8)) -> &mut Self {
        self.back_color_right = Some(back_color_right);
        self
    }

    pub fn finalize(&self) -> ColorDisplay {
        ColorDisplay { 
            text_color_left: self.text_color_left,
            back_color_left: self.back_color_left,
            text_color_right: self.text_color_right,
            back_color_right: self.back_color_right 
        }
    }
}

/// Returns the ansi sequence for a struct with a DisplayInfo field
pub trait HasTextColor {
    fn ansi_text_colors (&self) -> ColorDisplay;
}

pub trait HasBackColor {
    fn ansi_back_colors (&self) -> ColorDisplay;
}

pub trait HasTextDisplay {
    fn text_display (&self) -> TextDisplay;
}