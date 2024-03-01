use std::{fmt::{Debug, Display}, fs};

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