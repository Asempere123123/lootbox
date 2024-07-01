use inline_colorization::*;
use reqwest;
use std::fs;
use std::path::{Path, PathBuf};

const PYTHON_INSTALLS_DIRECTORY: &str = "python_installs";

#[cfg(target_os = "windows")]
const PYTHON_INSTALLER_NAME: &str = "installer.exe";
#[cfg(target_os = "linux")]
const PYTHON_INSTALLER_NAME: &str = "installer.tgz";

pub fn install_version(cli: &crate::Cli, data_path: &Path, version_to_install: &String, force: &bool) {
    let install_path = data_path
        .join(PYTHON_INSTALLS_DIRECTORY)
        .join(version_to_install);

    // create install path if doesn't exist
    fs::create_dir_all(&install_path).expect("Couldn't create new install directory");

    if *force {
        fs::remove_dir_all(&install_path).expect("Couldn't delete previous instalation");

        fs::create_dir_all(&install_path).expect("Couldn't create new install directory");
    } else {
        if fs::read_dir(&install_path)
            .expect("Couldn't read installation dir")
            .next()
            .is_some()
        {
            panic!("Install directory is not empty, use --force to override previous install");
        }
    }

    // Install python installer
    install_python_installer(cli, &install_path, version_to_install);

    println!(
        r#"{color_green}Installed python at "{}"{color_reset}"#,
        install_path.to_string_lossy()
    )
}

#[cfg(target_os = "windows")]
pub fn install_python_installer(cli: &crate::Cli, install_directory: &PathBuf, version_to_install: &String) {

    let download_url = format!("https://www.python.org/ftp/python/{version_to_install}/python-{version_to_install}-amd64.exe");

    let mut installer_content_response =
        reqwest::blocking::get(download_url)
            .expect("Error requesting python version");

    if !installer_content_response.status().is_success() {
        panic!("Python version does not exist. Write the pythons version complete name (3.10.0)");
    }

    crate::print_debug!(cli, "{}", install_directory.join(PYTHON_INSTALLER_NAME).to_string_lossy());
    let mut installer_file =
        fs::File::create(install_directory.join(PYTHON_INSTALLER_NAME))
            .expect("Couldn't create installer file");

    std::io::copy(&mut installer_content_response, &mut installer_file).expect("Couldn't copy installer contents");
}

#[cfg(target_os = "linux")]
pub fn install_python_installer(cli: &crate::Cli, install_directory: &PathBuf, version_to_install: &String) {

    let download_url = format!("https://www.python.org/ftp/python/{version_to_install}/python-{version_to_install}-amd64.exe");

    let mut installer_content_response =
        reqwest::blocking::get(download_url)
            .expect("Error requesting python version");

    if !installer_content_response.status().is_success() {
        panic!("Python version does not exist. Write the pythons version complete name (3.10.0)");
    }

    crate::print_debug!(cli, "{}", install_directory.join(PYTHON_INSTALLER_NAME).to_string_lossy());
    let mut installer_file =
        fs::File::create(install_directory.join(PYTHON_INSTALLER_NAME))
            .expect("Couldn't create installer file");

    std::io::copy(&mut installer_content_response, &mut installer_file).expect("Couldn't copy installer contents");
}
