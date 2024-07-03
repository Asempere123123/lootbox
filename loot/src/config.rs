use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
use toml;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub name: String,
    pub python_version: String,
    pub requirements: HashMap<String, String>,
}

pub fn get_python_version() -> String {
    let config_string = read_to_string(crate::DEPENDENCIES_FILE).expect("Error reading config");

    let config: Config = toml::from_str(&config_string).expect("Error parsing config");

    config.python_version
}

pub fn get_config() -> Config {
    let config_string = read_to_string(crate::DEPENDENCIES_FILE).expect("Error reading config");

    toml::from_str(&config_string).expect("Error parsing config")
}

pub struct ChangesDetected {
    pub python_version: bool,
    pub deps_changed: bool,
}

pub fn detect_changes(data_path: &Path) -> ChangesDetected {
    let config_string = read_to_string(crate::DEPENDENCIES_FILE).expect("Error reading config");
    let config: Config = toml::from_str(&config_string).expect("Error parsing config");

    let mut previous_config_string =
        read_to_string(Path::new("./.lootbox").join(crate::DEPENDENCIES_FILE));

    if previous_config_string.is_err() {
        crate::new::initialize_lootbox_dir(data_path);
        previous_config_string =
            read_to_string(Path::new("./.lootbox").join(crate::DEPENDENCIES_FILE));
    }

    let previous_config: Config =
        toml::from_str(&previous_config_string.unwrap()).expect("Error parsing config");

    ChangesDetected {
        python_version: previous_config.python_version != config.python_version,
        deps_changed: previous_config.requirements != config.requirements,
    }
}
