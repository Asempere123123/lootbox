use dependency_graph::*;
use pep440_rs::{Version, VersionSpecifiers};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use tokio_stream::{self, StreamExt};

use reqwest::get;
use serde::Deserialize;
use serde_json;

#[derive(Deserialize, Debug)]
struct Package {
    releases: std::collections::HashMap<String, Vec<ReleaseData>>,
}

#[derive(Deserialize, Debug)]
struct ReleaseData {
    yanked: bool,
}

async fn get_global_package_info(package: &str) -> Package {
    let package = package.split_whitespace().next().unwrap();
    let url = format!("https://pypi.org/pypi/{package}/json");

    let response = get(url).await.expect("Error fetching package info");

    let mut package_info: Package =
        serde_json::from_str(&response.text().await.expect("Error fetching package info"))
            .expect("Error parsing package info");

    package_info
        .releases
        .retain(|_, datas| datas.iter().any(|data| !data.yanked));

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

#[derive(Debug, Deserialize)]
struct PackageWithVersion {
    info: PackageInfo,
}

#[derive(Debug, Deserialize)]
struct PackageInfo {
    requires_dist: Option<Vec<String>>,
}

async fn get_deps_of_dependency(
    dependency: &String,
    version: &Version,
) -> Vec<(String, VersionSpecifiers)> {
    let dependency = dependency.split_whitespace().next().unwrap();
    let url = format!("https://pypi.org/pypi/{dependency}/{version}/json");

    let response = get(url).await.expect("Error fetching package info");
    let package: PackageWithVersion =
        serde_json::from_str(&response.text().await.expect("Error fetching package info"))
            .expect("Error parsing package info");

    let mut package_dependencies = Vec::new();
    for dep_dep in package.info.requires_dist.unwrap_or_default() {
        let mut splited_dep_dep: Vec<String> = dep_dep.split("(").map(|s| s.to_string()).collect();

        println!("{:?}", splited_dep_dep);

        if splited_dep_dep.len() == 1 {
            let mut new_splited_dep_dep = None;
            if !splited_dep_dep[0].chars().all(char::is_alphanumeric) {
                let mut index = None;
                for (idx, char) in splited_dep_dep[0].chars().enumerate() {
                    if !char.is_alphanumeric() {
                        index = Some(idx);
                        break;
                    }
                }
                let a = splited_dep_dep[0][..index.unwrap()].to_owned();
                let b = splited_dep_dep[0][index.unwrap()..].to_owned();

                new_splited_dep_dep = Some(vec![a, b])
            }

            if new_splited_dep_dep.is_some() {
                splited_dep_dep = new_splited_dep_dep.unwrap();
            }
        }

        let package_name = splited_dep_dep[0].trim_end().to_string();

        let string_default = String::from("");

        let deps = splited_dep_dep
            .get(1)
            .unwrap_or(&string_default)
            .split(")")
            .next()
            .unwrap_or_default();
        let depencendies = VersionSpecifiers::from_str(&deps).unwrap_or(VersionSpecifiers::empty());

        package_dependencies.push((package_name, depencendies));
    }

    package_dependencies
}

async fn dependency_to_resolver_package(package: (String, String)) -> ResolverPackage {
    let dependency = package.0;
    let version = Version::from_str(&package.1).expect("Could not parse version");

    let deps = get_deps_of_dependency(&dependency, &version).await;
    let deps = deps
        .into_iter()
        .map(|(dep_name, requirements)| ResolverDependency {
            name: dep_name,
            version: requirements,
        })
        .collect();

    ResolverPackage {
        name: dependency.to_string(),
        version: version,
        dependencies: deps,
    }
}

async fn dependency_to_resolver_package_from_version_specifier(
    package: (String, String),
) -> ResolverPackage {
    let dependency = package.0;
    let version_requirements =
        VersionSpecifiers::from_str(&package.1).expect("Could not parse version");

    let mut versions = get_versions_of_package(&dependency).await;
    versions.sort();
    versions.reverse();

    for version in versions {
        let parsed_version = Version::from_str(&version).expect("Error parsing version");

        if version_requirements.contains(&parsed_version) {
            return dependency_to_resolver_package((dependency, version)).await;
        }
    }

    panic!("No version found that matches the requirements");
}

#[derive(Debug, Clone)]
struct ResolverPackage {
    name: String,
    version: Version,
    dependencies: Vec<ResolverDependency>,
}

#[derive(Debug, Clone)]
struct ResolverDependency {
    name: String,
    version: VersionSpecifiers,
}

impl Node for ResolverPackage {
    type DependencyType = ResolverDependency;

    fn dependencies(&self) -> &[Self::DependencyType] {
        &self.dependencies[..]
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        // Check that name is an exact match, and that the dependency
        // requirements are fulfilled by our own version
        self.name == dependency.name && dependency.version.contains(&self.version)
    }
}

pub async fn resolve_dependencies(dependencies: HashMap<String, String>) -> Vec<(String, String)> {
    let mut packages: Vec<ResolverPackage> = tokio_stream::iter(dependencies)
        .then(dependency_to_resolver_package)
        .collect()
        .await;

    let solved_graph;
    loop {
        let packages_clone = packages.clone();
        let graph = DependencyGraph::from(&packages_clone[..]);

        for unresolved_dep in graph.unresolved_dependencies() {
            packages.push(
                dependency_to_resolver_package_from_version_specifier((
                    unresolved_dep.name.clone(),
                    unresolved_dep.version.to_string(),
                ))
                .await,
            );
        }

        if graph.is_internally_resolvable() {
            solved_graph = DependencyGraph::from(&packages[..]);
            break;
        }
    }

    let mut dependencies = Vec::new();
    for package in solved_graph {
        match package {
            Step::Resolved(package) => {
                dependencies.push((package.name.to_owned(), package.version.to_string()))
            }

            Step::Unresolved(_) => unreachable!(),
        }
    }

    let mut seen = HashSet::new();
    dependencies.retain(|dep| seen.insert(dep.to_owned()));

    dependencies
}
