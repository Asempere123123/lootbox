use std::fs::File;
use std::path::Path;
use std::{fs, io::Write};
use std::process::Command;
use inline_colorization::*;

use crate::DEPENDENCIES_FILE;

pub fn new_project(cli: &crate::Cli, data_path: &Path, name: &std::path::PathBuf, python_version: &String) {
    fs::create_dir_all(name).expect("Invalid path for project");

    let mut file =
        File::create(name.join(DEPENDENCIES_FILE)).expect("Error creating dependencies file");

    let default_requirements = include_str!("default_requirements.toml");

    let default_requirements = default_requirements
        .replace(
            "{project_name}",
            &name.file_name().unwrap().to_string_lossy(),
        )
        .replace("{project_python_version}", python_version);

    file.write(default_requirements.as_bytes())
        .expect("Error writing dependencies file");

    // Create .lootbox dir
    let lootbox_dir_path = name.join(".lootbox");
    fs::create_dir(&lootbox_dir_path).expect("Error creating .lootbox dir");

    let python_bin_path = crate::install::get_bin_path(data_path, python_version);
    let create_venv_result = Command::new(python_bin_path)
        .arg("-m")
        .arg("venv")
        .arg(&lootbox_dir_path.join("venv"))
        .output()
        .expect("Error creating venv");

    if ! create_venv_result.status.success() {
        panic!("Error creating venv");
    }

    println!("{color_green}Project created{color_reset}")
}
