use inline_colorization::*;
use std::fs;
use std::path::Path;

use crate::config::detect_changes;
use crate::utils::run_venv_command;

pub fn run(data_path: &Path) {
    let mut changes = detect_changes(data_path);

    if changes.python_version {
        println!("{color_yellow}Python version change detected, updating python{color_reset}");
        fs::remove_dir_all("./.lootbox").expect("Error reinstalling python");
        crate::new::initialize_lootbox_dir(data_path);

        changes = detect_changes(data_path);
    }

    // Run python
    let runing_result = run_venv_command(data_path, "python src/main.py");

    if runing_result.is_err() {
        panic!("Error runing main");
    }
}
