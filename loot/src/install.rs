use inline_colorization::*;
use reqwest;
use std::fs;
use std::path::{Path, PathBuf};

use crate::PYTHON_INSTALLS_DIRECTORY;

#[cfg(target_os = "windows")]
const PYTHON_INSTALLER_NAME: &str = "installer.zip";
#[cfg(not(target_os = "windows"))]
const PYTHON_INSTALLER_NAME: &str = "installer.tgz";

pub fn install_version(
    cli: &crate::Cli,
    data_path: &Path,
    version_to_install: &String,
    force: &bool,
) {
    let install_path = data_path
        .join(PYTHON_INSTALLS_DIRECTORY)
        .join(version_to_install);

    crate::print_debug!(cli, "{}", install_path.to_string_lossy());

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

    // Install python
    install_python(cli, &install_path, version_to_install);

    println!(
        r#"{color_green}Installed python at "{}"{color_reset}"#,
        install_path.to_string_lossy()
    );
}

#[cfg(target_os = "windows")]
fn install_python_installer(
    cli: &crate::Cli,
    install_directory: &PathBuf,
    version_to_install: &String,
) {
    let download_url = format!("https://www.python.org/ftp/python/{version_to_install}/python-{version_to_install}-embed-amd64.zip");

    let mut installer_content_response =
        reqwest::blocking::get(download_url).expect("Error requesting python version");

    if !installer_content_response.status().is_success() {
        panic!("Python version does not exist. Write the pythons version complete name (3.10.0)");
    }

    crate::print_debug!(
        cli,
        "{}",
        install_directory
            .join(PYTHON_INSTALLER_NAME)
            .to_string_lossy()
    );
    let mut installer_file = fs::File::create(install_directory.join(PYTHON_INSTALLER_NAME))
        .expect("Couldn't create installer file");

    std::io::copy(&mut installer_content_response, &mut installer_file)
        .expect("Couldn't copy installer contents");
}

#[cfg(not(target_os = "windows"))]
fn install_python_installer(
    cli: &crate::Cli,
    install_directory: &PathBuf,
    version_to_install: &String,
) {
    let download_url = format!(
        "https://www.python.org/ftp/python/{version_to_install}/Python-{version_to_install}.tgz"
    );

    let mut installer_content_response =
        reqwest::blocking::get(download_url).expect("Error requesting python version");

    if !installer_content_response.status().is_success() {
        panic!("Python version does not exist. Write the pythons version complete name (3.10.0)");
    }

    crate::print_debug!(
        cli,
        "{}",
        install_directory
            .join(PYTHON_INSTALLER_NAME)
            .to_string_lossy()
    );
    let mut installer_file = fs::File::create(install_directory.join(PYTHON_INSTALLER_NAME))
        .expect("Couldn't create installer file");

    std::io::copy(&mut installer_content_response, &mut installer_file)
        .expect("Couldn't copy installer contents");
}

#[cfg(target_os = "windows")]
fn install_python(cli: &crate::Cli, install_directory: &PathBuf, _version_to_install: &String) {
    let installer_file = fs::File::open(install_directory.join(PYTHON_INSTALLER_NAME))
        .expect("Error opening installer file");

    let mut installer_archive =
        zip::ZipArchive::new(installer_file).expect("Error creating archive for installer");

    for i in 0..installer_archive.len() {
        let mut file = installer_archive
            .by_index(i)
            .expect("Error reading file from zip");
        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };

        if file.is_dir() {
            crate::print_debug!(cli, "File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(install_directory.join(outpath)).unwrap();
        } else {
            crate::print_debug!(
                cli,
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(install_directory.join(p)).unwrap();
                }
            }
            let mut outfile = fs::File::create(install_directory.join(outpath)).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn install_python(_cli: &crate::Cli, install_directory: &PathBuf, version_to_install: &String) {
    use std::process::Command;

    // Decompress
    let tar_output = Command::new("tar")
        .arg("-xf")
        .arg(install_directory.join(PYTHON_INSTALLER_NAME))
        .arg("-C")
        .arg(install_directory)
        .output()
        .expect("Failed to decompress python installer");

    if !tar_output.status.success() {
        panic!("Failed to decompress python installer");
    }

    // Install
    let source_directory_name = format!("Python-{version_to_install}");
    let python_source_directory = install_directory.join(source_directory_name);

    let configure_output = Command::new(&python_source_directory.join("configure"))
        .current_dir(&python_source_directory)
        .arg("--enable-optimizations")
        .arg(format!("--prefix={}", install_directory.to_string_lossy()))
        .output()
        .expect("Error configuring python install");

    if !configure_output.status.success() {
        panic!("Error configuring python install");
    }

    println!("Python install configured");

    let make_output = Command::new("make")
        .current_dir(&python_source_directory)
        .output()
        .expect("Error running make");

    if !make_output.status.success() {
        panic!("Error running make");
    }

    println!("Installing python");

    let make_install_output = Command::new("make")
        .current_dir(&python_source_directory)
        .arg("install")
        .output()
        .expect("Error installing python. Try runing with admin persissions (sudo)");

    if !make_install_output.status.success() {
        panic!("Error installing python. Try runing with admin persissions (sudo)");
    }
}

#[cfg(target_os = "windows")]
pub fn get_bin_path(data_path: &Path, version: &String) -> PathBuf {
    data_path
        .join(PYTHON_INSTALLS_DIRECTORY)
        .join(version)
        .join("python.exe")
}

#[cfg(not(target_os = "windows"))]
pub fn get_bin_path(data_path: &Path, version: &String) -> PathBuf {
    data_path
        .join(PYTHON_INSTALLS_DIRECTORY)
        .join(version)
        .join("bin")
        .join("python3")
}
