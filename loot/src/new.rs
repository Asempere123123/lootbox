use std::fs::File;
use std::{fs, io::Write};

use crate::DEPENDENCIES_FILE;

pub fn new_project(name: &std::path::PathBuf) {
    fs::create_dir(name).expect("Invalid path for project");

    let mut file: File =
        File::create(name.join(DEPENDENCIES_FILE)).expect("Error creating dependencies file");

    let default_requirements = include_str!("default_requirements.toml");

    let default_requirements = default_requirements.replace(
        "{project_name}",
        &name.file_name().unwrap().to_string_lossy(),
    );
    file.write(default_requirements.as_bytes())
        .expect("Error writing dependencies file");
}
