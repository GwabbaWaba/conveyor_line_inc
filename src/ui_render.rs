
use crossterm::cursor;

use crate::std_out;

/*~ TODO - make system for scrolling text both vertical and horizontal ~*/

pub struct TextBox {
    top_left: (u16, u16),
    pub width: u16,
    pub height: u16,
    last_cursor_pos: (u16, u16),
    text: Option<String>,
    flair: SideFlair
}

pub struct TextBoxBuilder {
    top_left: (u16, u16),
    pub width: u16,
    pub height: u16,
    last_cursor_pos: (u16, u16),
    text: Option<String>,
    flair: SideFlair
}

impl TextBoxBuilder {
    pub fn new(top_left: (u16, u16), mut width: u16, height: u16) -> Self {
        // print_box requires width >= 2
        if width < 2 {
            width = 2;
        }

        Self { 
            top_left: top_left,
            width: width,
            height: height,
            last_cursor_pos: (top_left.0, top_left.1),
            text: None,
            flair: SideFlair {
                top_horizontal: format!("╔{:═<w$}╗", "", w = width as usize - 2), 
                bottom_horizontal: format!("╚{:═<w$}╝", "", w = width as usize - 2), 
                verticals: format!("{0}{1: <w$}{0}", "║", "",  w = width as usize - 2) 
            }
        }
    }

    pub fn text(&mut self, text: String) -> &mut Self {
        self.text = Some(text);
        self
    }

    pub fn flair_top(&mut self, flair_top: String) -> &mut Self {
        self.flair.top_horizontal = flair_top;
        self
    }

    pub fn flair_bottom(&mut self, flair_bottom: String) -> &mut Self {
        self.flair.bottom_horizontal = flair_bottom;
        self
    }

    pub fn flair_vertical(&mut self, flair_vertical: String) -> &mut Self {
        self.flair.verticals = flair_vertical;
        self
    }

    pub fn finalize(&self) -> TextBox {
        let mut text_box = TextBox { 
            top_left: self.top_left,
            width: self.width,
            height: self.height,
            last_cursor_pos: self.last_cursor_pos,
            text: self.text.clone(),
            flair: self.flair.clone()
        };
        text_box.print_box();
        text_box
    }
}

#[derive(Clone)]
struct SideFlair {
    pub top_horizontal: String,
    pub bottom_horizontal: String,
    pub verticals: String
}

impl TextBox {
    pub fn update_text(&mut self, text_update_type: TextUpdateType, text: &'static str) {
        match text_update_type {
            TextUpdateType::Append => {
                let print_end = pos_print(
                    self.last_cursor_pos,
                    self.width - 1,
                    self.remaining_bounds(),
                    text
                );
                self.last_cursor_pos = (self.top_left.0, self.last_cursor_pos.1 + print_end.1);
            },
            TextUpdateType::Replace => {
                self.clear();

                let print_end = pos_print(
                    self.top_left,
                    self.width - 1,
                    self.height,
                    text
                );
                self.last_cursor_pos = (self.top_left.0, self.top_left.1 + print_end.1);
            }
        };
    }

    pub fn remaining_bounds(&self) -> u16 {
        (self.last_cursor_pos.1 - self.top_left.1) + self.height
    }

    pub fn clear(&mut self) {
        pos_print(
            self.top_left,
            self.width - 1,
            self.height,
            &format!("{: <w$}", "", w = ((self.width - 2) * self.height) as usize)
        );
        self.last_cursor_pos = (self.top_left.0, self.top_left.1);
    }

    fn print_box(&self) {
        ignorant_queue!(std_out, cursor::MoveTo(self.top_left.0, self.top_left.1));
        print!("{}", self.flair.top_horizontal);
    
        for _ in 0..self.height {
            ignorant_queue!(std_out, cursor::MoveDown(1));
            ignorant_queue!(std_out, cursor::MoveToColumn(self.top_left.0));
    
            print!("{}", self.flair.verticals);
        }
    
        ignorant_queue!(std_out, cursor::MoveDown(1), cursor::MoveToColumn(self.top_left.0));
    
        print!("{}", self.flair.bottom_horizontal);
    }
}

pub enum TextUpdateType {
    Append,
    Replace
}

pub fn pos_print(top_left: (u16, u16), chars_per_line: u16, max_lines: u16, txt: &str) -> (u16, u16) {
    ignorant_queue!(std_out, cursor::MoveTo(top_left.0, top_left.1));
    let mut i: u16 = 1;
    let mut j: u16 = 1;

    for c in txt.as_bytes() {
        print!("{}", *c as char);
        i += 1;

        if i % chars_per_line == 0 {
            // making sure it doesn't escape the bounds
            j += 1;
            if j > max_lines {
                break;
            }

            ignorant_queue!(std_out, cursor::MoveDown(1));
            ignorant_queue!(std_out, cursor::MoveToColumn(top_left.0));
            i += 1;
        }
    }
    (i, j)
}