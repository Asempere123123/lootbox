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

    let default_requirements = include_str!("default_files/default_requirements.toml");

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
        .expect("Error creating venv. Is the python version installed?");

    crate::print_debug!(cli, "{:?}", create_venv_result);
    if !create_venv_result.status.success() {
        panic!("Error creating venv. Is the python version installed?");
    }

    let mut previous_config = File::create(lootbox_dir_path.join(DEPENDENCIES_FILE))
        .expect("Error creating dependencies file");

    previous_config
        .write(default_requirements.as_bytes())
        .expect("Error writing dependencies file");

    crate::utils::run_venv_command_from_out(
        data_path,
        "python -m pip install --upgrade pip",
        &name.to_string_lossy(),
    )
    .expect("Could not upgrade pip");

    crate::utils::run_venv_command_from_out(
        data_path,
        "pip install pipdeptree",
        &name.to_string_lossy(),
    )
    .expect("Could not install pipdeptree");

    // Create src
    fs::create_dir_all(name.join("src")).expect("Error creating src directory");

    let mut main_file =
        File::create(name.join("src").join("main.py")).expect("Error creating main.py file");

    main_file
        .write(include_str!("default_files/default_main.py").as_bytes())
        .expect("Error writing main.py file");

    println!("{color_green}Project created{color_reset}")
}

// This is a simple modification over the code above for any big diferences, the one above should be the canonical one
pub fn initialize_lootbox_dir(data_path: &Path) {
    let lootbox_dir_path = Path::new("./.lootbox");

    // Venv
    let python_bin_path =
        crate::install::get_bin_path(data_path, &crate::config::get_python_version());
    let create_venv_result = Command::new(&python_bin_path)
        .arg("-m")
        .arg("venv")
        .arg(&lootbox_dir_path.join("venv"))
        .output()
        .expect("Error creating venv. Is the python version installed?");

    if !create_venv_result.status.success() {
        panic!("Error creating venv. Is the python version installed?");
    }

    // Previous config
    let mut previous_config = File::create(lootbox_dir_path.join(DEPENDENCIES_FILE))
        .expect("Error creating dependencies file");

    let current_config = crate::config::get_config();
    let default_requirements = include_str!("default_files/default_requirements.toml");
    let default_requirements = default_requirements
        .replace("{project_name}", &current_config.name)
        .replace("{project_python_version}", &current_config.python_version);

    previous_config
        .write(default_requirements.as_bytes())
        .expect("Could not write previous config");

    crate::utils::run_venv_command_with_output(data_path, "python -m pip install --upgrade pip")
        .expect("Could not upgrade pip");

    crate::utils::run_venv_command_with_output(data_path, "pip install pipdeptree")
        .expect("Could not install pipdeptree");
}
