use inline_colorization::*;
use reqwest;
use std::fs;
use std::path::{Path, PathBuf};

use crate::PYTHON_INSTALLS_DIRECTORY;

#[cfg(target_os = "windows")]
const PYTHON_INSTALLER_NAME: &str = "installer.zip";
#[cfg(target_os = "linux")]
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
    install_python(cli, &install_path);

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

#[cfg(target_os = "linux")]
fn install_python_installer(
    cli: &crate::Cli,
    install_directory: &PathBuf,
    version_to_install: &String,
) {
    let download_url = format!(
        "https://www.python.org/ftp/python/{version_to_install}/python-{version_to_install}.tgz"
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
fn install_python(cli: &crate::Cli, install_directory: &PathBuf) {
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
