use std::{fmt::{Debug, Display}, fs};

use crossterm::{cursor, execute};

use crate::terminal;

pub fn write_to_debug<T: Display + Debug>(text: T) {
    let mut data = fs::read_to_string("src/tmp/debug.txt").expect("Unable to read file");
    data = format!("{}{}\n", data, text);
    fs::write("src/tmp/debug.txt", data).expect("Unable to write file");
}

pub fn write_to_debug_pretty(text: String) {
    write_to_debug(text.replace("{", "{\n")
        .replace("}", "}\n")
        .replace(r"\n", "\n")
        .replace(r"\t", "\t")
    );
}

pub fn clear_debug() {
    fs::write("src/tmp/debug.txt", "").expect("Unable to write file");
}

pub fn positional_print<T: Display>(x: u16, y: u16, text: T) {
    let _ = execute!(terminal().backend_mut(), cursor::MoveTo(x, y));
    print!("{}", text);
}

pub fn positional_debug_print<T: Debug>(x: u16, y: u16, text: T) {
    let _ = execute!(terminal().backend_mut(), cursor::MoveTo(x, y));
    print!("{:?}", text);
}