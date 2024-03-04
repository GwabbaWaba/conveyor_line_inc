use std::{ffi::OsString, fs::DirEntry};

pub const MODULES_PATH: &str = r#"resources/modules"#;

pub fn dir_entry_is_dir(dir_entry: Result<&DirEntry, &std::io::Error>) -> bool {
    dir_entry.is_ok() && {
        let file_type = dir_entry.unwrap().file_type();
        file_type.is_ok() && file_type.unwrap().is_dir()
    }
}

pub fn os_string_to_string(os_string: OsString) -> String {
    String::from(os_string.to_str().unwrap())
}