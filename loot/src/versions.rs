use pep440_rs::{Version, VersionSpecifiers};
use std::str::FromStr;

use reqwest::get;
use serde::Deserialize;
use serde_json;

use crate::app::Config;

#[derive(Deserialize, Debug, Clone)]
struct Package {
    releases: std::collections::HashMap<String, Vec<ReleaseData>>,
}

#[derive(Deserialize, Debug, Clone)]
struct ReleaseData {
    yanked: bool,
    requires_python: Option<String>,
}

async fn get_global_package_info(package: &str) -> Package {
    let url = format!("https://pypi.org/pypi/{package}/json");
    let response = get(url).await.expect("Error fetching package info");

    let mut package_info: Package =
        serde_json::from_str(&response.text().await.expect("Error fetching package info"))
            .expect("Error parsing package info");

    package_info.releases.retain(|_, datas| {
        if datas.len() == 0 {
            return true;
        }

        datas.iter().any(|data| {
            let version_match;
            if data.requires_python.is_none() {
                version_match = true;
            } else {
                let config = std::fs::read_to_string(crate::DEPENDENCIES_FILE)
                    .expect("Could not read config");
                let config: Config = toml::from_str(&config).expect("Could not parse config");
                let python_version = Version::from_str(&config.python_version)
                    .expect("Error parsing python version");

                let version_requirements =
                    VersionSpecifiers::from_str(&data.requires_python.as_ref().unwrap())
                        .expect("Error parsing python version requirements");

                version_match = version_requirements.contains(&python_version);
            }
            !data.yanked && version_match
        })
    });

    package_info
}

pub async fn get_versions_of_package(package: &str) -> Vec<String> {
    let package_info = get_global_package_info(package).await;

    let version_numbers: Vec<String> = package_info
        .releases
        .into_iter()
        .map(|(version, _)| version)
        .collect();

    version_numbers
}

pub async fn version_exists(package: &str, version: &String) -> bool {
    let versions = get_versions_of_package(package).await;

    versions.contains(version)
}
