use std::fs;

use crate::app::AppExternal;
use crate::utils::create_file_with_content;
use crate::DEPENDENCIES_FILE;

pub async fn new_project(
    name: &std::path::PathBuf,
    python_version: &String,
    force: &bool,
    mut app: AppExternal<'_>,
) {
    // Check if dir is empty
    if fs::exists(name).unwrap_or(true) {
        if *force {
            fs::remove_dir_all(name).expect("Error cleaning target dir");
        } else {
            panic!("Target directory is not empty. Use --force to override it");
        }
    }

    // Create all directories
    fs::create_dir_all(name.join("src")).expect("Error creating src dir");

    // Create all files needed
    create_file_with_content(
        &name.join("src").join("main.py"),
        include_bytes!("default_files/default_main.py"),
    )
    .expect("Error creating default main.py");
    create_file_with_content(
        &name.join(DEPENDENCIES_FILE),
        include_bytes!("default_files/default_requirements.toml"),
    )
    .expect("Error creating default lootbox project file");

    create_lootbox_dir(Some(name), python_version, &mut app).await;
}

pub async fn create_lootbox_dir(
    path: Option<&std::path::PathBuf>,
    python_version: &String,
    app: &mut AppExternal<'_>,
) {
    let source_location = match path {
        Some(path) => path,
        None => &std::path::PathBuf::new(),
    };

    if !source_location.join(DEPENDENCIES_FILE).exists() {
        panic!("Not inside a lootbox directory");
    }
    let location = source_location.join(".lootbox");

    // Create all files
    let _ = fs::remove_dir_all(&location);
    fs::create_dir_all(&location).expect("Error creating .lootbox dir");

    create_file_with_content(
        &location.join(DEPENDENCIES_FILE),
        include_bytes!("default_files/default_requirements.toml"),
    )
    .expect("Error creating default lootbox project file");

    // Setup venv
    let python_binary = app
        .get_python_binary(python_version)
        .expect("Python version does not exist");

    let create_venv_command = format!(
        "{} -m venv {}",
        python_binary.to_string_lossy(),
        location.join("venv").to_string_lossy()
    );
    app.run_external_command(create_venv_command).await;

    app.make_internal(Some(source_location.to_owned())).await;

    app.run_internal_command("python -m pip install --upgrade pip".to_owned())
        .await;
    app.run_internal_command("pip install pipdeptree".to_owned())
        .await;
}
