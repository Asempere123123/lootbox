use inline_colorization::*;
use std::path::PathBuf;
use std::{
    collections::{HashMap, HashSet},
    fs,
};

use crate::app::{AppExternal, Config};
use crate::utils;

pub async fn run_app(mut app: AppExternal<'_>) {
    app.make_internal(None).await;

    let new_config = app.app_config.clone().unwrap();
    let old_config = AppExternal::get_old_config(None);

    if app.app_config.as_ref().unwrap() != &old_config {
        handle_incorrect_config(&mut app, new_config, old_config).await;
    }

    app.run_internal_command("python ./src/main.py".to_owned())
        .await;
}

async fn handle_incorrect_config(
    app: &mut AppExternal<'_>,
    new_config: Config,
    mut old_config: Config,
) {
    if new_config.python_version != old_config.python_version {
        println!("{color_yellow}Upgrading python version{color_reset}");
        fs::remove_dir_all("./.lootbox").expect("Error cleaning lootbox dir");
        crate::new::create_lootbox_dir(None, &new_config.python_version, app).await;

        app.make_internal(None).await;

        old_config.requirements = HashMap::new();
    }

    if old_config.requirements != new_config.requirements {
        println!("Resolving_dependencies");
        let dependencies =
            crate::python_dependency_resolver::resolve_dependencies(&new_config.requirements)
                .expect("Error resolving dependencies");

        let old_dependencies =
            crate::python_dependency_resolver::resolve_dependencies(&old_config.requirements)
                .expect("Error resolving dependencies");

        let old_set: HashSet<_> = old_dependencies.iter().collect();
        let new_set: HashSet<_> = dependencies.iter().collect();

        let old_not_in_new: Vec<_> = old_set.difference(&new_set).collect();
        let new_not_in_old: Vec<_> = new_set.difference(&old_set).collect();

        // Wait for all other venv tasks to finish before installing dependencies.
        // With output blocks the thread untill the output is recieved. This happens once all previous commands are done.
        // This is important since in some cases those commands might be recreating the venv and we cant run commands inside a venv that doesnt exist.
        let python_version_command_output = app
            .run_internal_command_with_output("python --version".to_owned())
            .await
            .unwrap();
        println!("{}", python_version_command_output.stdout);

        println!("{:?}", dependencies);

        let mut handles = Vec::new();
        for (name, _) in old_not_in_new {
            let command_to_run = format!("pip uninstall -y {}", name);
            let handle = app.run_paralel_internal_command(None, command_to_run);
            handles.push(handle.await);

            println!("uninstalls sent");
        }

        println!("all uninstalls sent");

        for handle in handles {
            while !handle.is_finished() {}
        }

        let mut handles = Vec::new();
        for (name, version) in new_not_in_old {
            let command_to_run = format!("pip install --upgrade --no-deps {}=={}", name, version);
            let handle = app.run_paralel_internal_command(None, command_to_run);
            handles.push(handle.await);

            println!("install sent");
        }

        println!("all installs sent");

        for handle in handles {
            while !handle.is_finished() {}
        }
    }

    utils::create_file_with_content(
        &PathBuf::from(format!("./.lootbox/{}", crate::DEPENDENCIES_FILE)),
        toml::to_string(&new_config)
            .expect("Error serializing new config")
            .as_bytes(),
    )
    .expect("Error writing new config");
}
