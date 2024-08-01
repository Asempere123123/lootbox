use std::path::PathBuf;

use crate::app::AppExternal;
use crate::utils::create_file_with_content;
use crate::versions::{get_versions_of_package, version_exists};

pub async fn add_dependency(package: &String, version: &Option<String>, mut app: AppExternal<'_>) {
    let version_to_add = match version {
        Some(version) => {
            if version_exists(package, version).await {
                version.to_owned()
            } else {
                panic!("Version does not exist");
            }
        }
        None => {
            let versions = get_versions_of_package(package).await;
            let version = versions.iter().max().expect("Dependency has no versions");
            version.to_owned()
        }
    };

    app.make_internal(None).await;

    app.app_config
        .as_mut()
        .unwrap()
        .requirements
        .insert(package.to_owned(), version_to_add);

    create_file_with_content(
        &PathBuf::from(crate::DEPENDENCIES_FILE),
        toml::to_string_pretty(&app.app_config.unwrap())
            .expect("Could not convert to toml")
            .as_bytes(),
    )
    .expect("Error writing to config file");
}
