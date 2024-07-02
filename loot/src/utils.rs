use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::process::{Command, Output};

use crate::DEPENDENCIES_FILE;

#[macro_export]
macro_rules! print_debug {
    ($cli:expr, $($arg:tt)*) => {
        if $cli.debug {
            let to_print = format!($($arg)*);
            println!("{color_yellow}{}{color_reset}", to_print);
        }
    };
}

fn get_activate_path() -> String {
    if fs::metadata("./.lootbox/venv/Scripts").is_ok()
        && fs::metadata("./.lootbox/venv/Scripts").unwrap().is_dir()
    {
        if cfg!(target_os = "windows") {
            r#".\.lootbox\venv\Scripts\activate.bat"#.to_owned()
        } else {
            r#"./.lootbox/venv/Scripts/activate"#.to_owned()
        }
    } else {
        if cfg!(target_os = "windows") {
            r#".\.lootbox\venv\bin\activate.bat"#.to_owned()
        } else {
            r#"./.lootbox/venv/bin/activate"#.to_owned()
        }
    }
}

#[cfg(target_os = "windows")]
pub fn run_venv_command(data_path: &Path, command: &str) -> Result<Output, std::io::Error> {
    lootbox_dir_validations(data_path);

    let command_to_run = format!("{} && {}", get_activate_path(), command);
    Command::new("cmd")
        .args(&["/C", &command_to_run])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
}

#[cfg(not(target_os = "windows"))]
pub fn run_venv_command(data_path: &Path, command: &str) -> Result<Output, std::io::Error> {
    lootbox_dir_validations(data_path);

    let command_to_run = format!(". {} && {}", get_activate_path(), command);
    Command::new("sh")
        .args(&["-c", &command_to_run])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
}

#[cfg(target_os = "windows")]
pub fn run_venv_command_with_output(
    data_path: &Path,
    command: &str,
) -> Result<Output, std::io::Error> {
    lootbox_dir_validations(data_path);

    let command_to_run = format!("{} && {}", get_activate_path(), command);
    Command::new("cmd").args(&["/C", &command_to_run]).output()
}

#[cfg(not(target_os = "windows"))]
pub fn run_venv_command_with_output(
    data_path: &Path,
    command: &str,
) -> Result<Output, std::io::Error> {
    lootbox_dir_validations(data_path);

    let command_to_run = format!("{} && {}", get_activate_path(), command);
    Command::new("sh").args(&["-c", &command_to_run]).output()
}

fn lootbox_dir_validations(data_path: &Path) {
    if fs::metadata("./.lootbox").is_err() && fs::metadata(DEPENDENCIES_FILE).is_ok() {
        println!(".lootbox dir not detected, recreating it.");
        crate::new::initialize_lootbox_dir(data_path);
    } else if fs::metadata(DEPENDENCIES_FILE).is_err() {
        panic!("Dependencies file not detected");
    }
}
