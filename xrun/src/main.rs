use std::{error::Error, process::Command};

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        if Command::new("cargo").args(["run"]).spawn()?.wait()?.code().unwrap() != 1 {
            break;
        }
    }
    Ok(())
}
