use inline_colorization::*;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use toml;

use crate::config::{detect_changes, get_config};
use crate::utils::{run_venv_command, run_venv_command_with_output};

pub fn run(data_path: &Path) {
    let mut changes = detect_changes(data_path);

    if changes.python_version {
        println!("{color_yellow}Python version change detected, updating python{color_reset}");
        fs::remove_dir_all("./.lootbox").expect("Error reinstalling python");
        crate::new::initialize_lootbox_dir(data_path);

        changes = detect_changes(data_path);
    }

    if changes.deps_changed {
        println!("{color_yellow}Installing new dependencies{color_reset}");
        let config = get_config();

        let mut sub_dependency_requiremens: HashMap<String, Vec<String>> = HashMap::new();
        for (dependency, version) in config.requirements {
            let _new_dependencies = crate::dependencies::install_direct_dependency(
                data_path,
                &dependency,
                &version,
                &mut sub_dependency_requiremens,
            );
        }

        crate::dependencies::remove_dangling_dependencies(data_path);

        // Update config file

        let mut config_to_write = fs::File::create("./.lootbox/lootbox.toml")
            .expect("Error opening config internal file");

        let mut config_to_read_buffer = Vec::new();
        fs::File::open("./lootbox.toml")
            .expect("Error opening config file")
            .read_to_end(&mut config_to_read_buffer)
            .expect("Error opening config file");
        config_to_write
            .write_all(&config_to_read_buffer)
            .expect("Error writing config file");

        println!("Dependencies installed");
    }

    // Run python
    let runing_result = run_venv_command(data_path, "python src/main.py");

    if runing_result.is_err() {
        panic!("Error runing main");
    }
}

pub fn add_package(data_path: &Path, package: &String, version: &Option<String>) {
    let versions = get_versions_list(data_path, package);

    if let Some(version) = version {
        if versions.contains(version) {
            add_version_to_project(package.to_owned(), version.to_owned());
        } else {
            panic!("Version {} not found", version);
        }
    } else {
        add_version_to_project(package.to_owned(), versions[0].clone());
    }

    println!("Package {} added", package);
}

fn add_version_to_project(package: String, version: String) {
    let mut config = get_config();

    config.requirements.insert(package, version);

    let new_config_string =
        toml::to_string_pretty(&config).expect("Couldn't write new version to config");
    let mut config_file =
        fs::File::create(crate::DEPENDENCIES_FILE).expect("Couldn't write new version to config");
    config_file
        .write(new_config_string.as_bytes())
        .expect("Couldn't write new version to config");
}

fn get_versions_list(data_path: &Path, package: &String) -> Vec<String> {
    let command_to_run = format!("pip index versions {}", package);
    let list_versions_output =
        run_venv_command_with_output(data_path, &command_to_run).expect("Error listing versions");

    let versions_string = String::from_utf8_lossy(&list_versions_output.stdout);
    let start_index = versions_string
        .find("Available versions: ")
        .expect("Versions not formatted successfully")
        + 20;
    let versions_string = &versions_string[start_index..versions_string.len()];

    versions_string
        .split(", ")
        .map(|v| v.trim().to_owned())
        .collect()
}
