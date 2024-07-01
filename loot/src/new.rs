use inline_colorization::*;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use std::{fs, io::Write};

use crate::DEPENDENCIES_FILE;

pub fn new_project(
    cli: &crate::Cli,
    data_path: &Path,
    name: &std::path::PathBuf,
    python_version: &String,
) {
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
    println!("{}", python_bin_path.to_string_lossy());
    let create_venv_result = Command::new(&python_bin_path)
        .arg("-m")
        .arg("venv")
        .arg(&lootbox_dir_path.join("venv"))
        .output()
        .expect("Error creating venv. Is the python version installed? An external python install is also required on windows");

    println!("{:?}", create_venv_result);
    if !create_venv_result.status.success() {
        panic!("Error creating venv. Is the python version installed? An external python install is also required on windows");
    }

    println!("{color_green}Project created{color_reset}")
}
