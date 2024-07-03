use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use std::path::Path;

use crate::utils::{run_venv_command, run_venv_command_with_output};

pub fn install_direct_dependency(
    data_path: &Path,
    name: &str,
    version: &str,
    sub_dependency_requirements: &mut HashMap<String, Vec<String>>,
) {
    // Install the depencency
    let command_to_run = format!("pip install --upgrade --no-deps {}=={}", name, version);
    run_venv_command(data_path, &command_to_run).expect("Error installing dependency");

    // Add it to requirement confitions
    let sub_dep_req = sub_dependency_requirements.get_mut(name);
    if let Some(req) = sub_dep_req {
        req.push(format!("=={version}"));
    } else {
        sub_dependency_requirements.insert(name.to_owned(), vec![format!("=={version}")]);
    }

    // Check for its dependencies
    let dependencies = get_dependencies_of_package(data_path, name);
    for (dependency, version) in dependencies {
        let sub_dep_req = sub_dependency_requirements.get_mut(&dependency);
        if let Some(req) = sub_dep_req {
            if version != "" {
                req.push(version);
            }
        } else {
            if version != "" {
                sub_dependency_requirements.insert(dependency.clone(), vec![version]);
            } else {
                sub_dependency_requirements.insert(dependency.clone(), Vec::new());
            }
        }

        install_sub_dependency_recursive(data_path, name, &dependency, sub_dependency_requirements);
    }
}

fn install_sub_dependency_recursive(
    data_path: &Path,
    original_package_name: &str,
    name: &str,
    sub_dependency_requirements: &mut HashMap<String, Vec<String>>,
) {
    let version_condition_string: String = sub_dependency_requirements[name].join(",");
    let command_to_run = format!(
        "pip install --upgrade --no-deps '{}{}'",
        name, version_condition_string
    );
    run_venv_command(data_path, &command_to_run).expect("Error installing subdependency");

    let dependencies = get_dependencies_of_subpackage(data_path, original_package_name, name);
    for (dependency, version) in dependencies {
        let sub_dep_req = sub_dependency_requirements.get_mut(&dependency);
        if let Some(req) = sub_dep_req {
            if version != "" {
                req.push(version);
            }
        } else {
            if version != "" {
                sub_dependency_requirements.insert(dependency.clone(), vec![version]);
            } else {
                sub_dependency_requirements.insert(dependency.clone(), Vec::new());
            }
        }

        install_sub_dependency_recursive(
            data_path,
            original_package_name,
            &dependency,
            sub_dependency_requirements,
        );
    }
}

// Will only work if package was already installed
fn get_dependencies_of_package(data_path: &Path, name: &str) -> HashMap<String, String> {
    let ouput = run_venv_command_with_output(data_path, "pipdeptree --json-tree")
        .expect("Error reading dependencies");

    if !ouput.status.success() {
        panic!("Error reading dependencies")
    }

    let dependencies_json: Vec<Package> =
        serde_json::from_slice(&ouput.stdout).expect("Error reading dependencies");

    let package = dependencies_json
        .iter()
        .find(|p| p.package_name == name || p.key == name)
        .expect("Package not found");

    let mut requirements = HashMap::new();
    for depencency in &package.dependencies {
        if depencency.required_version == "Any" {
            requirements.insert(depencency.package_name.clone(), "".to_owned());
            continue;
        }

        requirements.insert(
            depencency.package_name.clone(),
            depencency.required_version.clone(),
        );
    }

    requirements
}

// Will only work if package was already installed
fn get_dependencies_of_subpackage(
    data_path: &Path,
    original_package_name: &str,
    name: &str,
) -> HashMap<String, String> {
    let ouput = run_venv_command_with_output(data_path, "pipdeptree --json-tree")
        .expect("Error reading dependencies");

    if !ouput.status.success() {
        panic!("Error reading dependencies")
    }

    let dependencies_json: Vec<Package> =
        serde_json::from_slice(&ouput.stdout).expect("Error reading dependencies");

    let package = dependencies_json
        .iter()
        .find(|p| p.package_name == original_package_name || p.key == original_package_name)
        .expect("Root package not found for dependency");

    let dependency = search_dependency_recursive(package, name).expect("Dependenct not found");

    let mut requirements = HashMap::new();
    for depencency in &dependency.dependencies {
        if depencency.required_version == "Any" {
            requirements.insert(depencency.package_name.clone(), "".to_owned());
            continue;
        }

        requirements.insert(
            depencency.package_name.clone(),
            depencency.required_version.clone(),
        );
    }

    requirements
}

fn search_dependency_recursive(package: &Package, target_name: &str) -> Option<Package> {
    for dependency in &package.dependencies {
        if dependency.package_name == target_name || dependency.key == target_name {
            return Some(dependency.to_owned());
        }

        if let Some(target_dep) = search_dependency_recursive(dependency, target_name) {
            return Some(target_dep.to_owned());
        }
    }
    None
}

#[derive(Deserialize, Debug, Clone)]
struct Package {
    key: String,
    package_name: String,
    #[allow(dead_code)]
    installed_version: String,
    required_version: String,
    dependencies: Vec<Package>,
}

pub fn remove_dangling_dependencies(data_path: &Path) {
    let ouput = run_venv_command_with_output(data_path, "pipdeptree --json-tree")
        .expect("Error reading dependencies");

    if !ouput.status.success() {
        panic!("Error reading dependencies")
    }

    let dependencies_json: Vec<Package> =
        serde_json::from_slice(&ouput.stdout).expect("Error reading dependencies");

    let config_requirements = crate::config::get_config().requirements;

    let mut deps_to_remove = Vec::new();
    let mut dep_was_removed = false;
    for depencency in dependencies_json {
        let mut suposed_package_version = config_requirements.get(&depencency.package_name);

        if suposed_package_version.is_none() {
            suposed_package_version = config_requirements.get(&depencency.key);
        }

        if suposed_package_version.is_none() {
            if ["pipdeptree", "setuptools", "pip"].contains(&depencency.package_name.as_str()) {
                continue;
            }

            deps_to_remove.push(depencency.package_name);
            dep_was_removed = true;
        }
    }

    for dep in deps_to_remove {
        let command_to_run = format!("pip uninstall {} -y", dep);

        println!("Uninstall {dep}");
        let output = run_venv_command(data_path, &command_to_run)
            .expect("Error uninstalling dangling dependency");

        if !output.status.success() {
            panic!("Error uninstalling dangling dependency");
        }
    }

    if dep_was_removed {
        remove_dangling_dependencies(data_path)
    }
}
