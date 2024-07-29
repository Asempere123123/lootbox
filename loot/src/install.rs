use std::fs;
use std::io::{BufWriter, Cursor};
use std::path::PathBuf;

use crate::app::AppExternal;
use crate::PYTHON_INSTALLS_DIRECTORY;

#[cfg(target_os = "windows")]
const PYTHON_INSTALLER_NAME: &str = "nuget.exe";
#[cfg(not(target_os = "windows"))]
const PYTHON_INSTALLER_NAME: &str = "installer.tgz";

pub async fn install_python_version(
    version_to_install: &String,
    force: &bool,
    app: AppExternal<'_>,
) {
    let install_path = app
        .data_path
        .join(PYTHON_INSTALLS_DIRECTORY)
        .join(version_to_install);

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

    install_python(install_path, version_to_install, &app).await;
}

#[cfg(target_os = "windows")]
async fn install_python(install_path: PathBuf, version_to_install: &String, app: &AppExternal<'_>) {
    let download_url = "https://dist.nuget.org/win-x86-commandline/latest/nuget.exe";

    let installer_content_response = reqwest::get(download_url)
        .await
        .expect("Error requesting python version");

    if !installer_content_response.status().is_success() {
        panic!("NuGet does not exist!?, The url might have moved");
    }
    let response_bytes = installer_content_response.bytes();

    let mut installer_file = BufWriter::new(
        fs::File::create(install_path.join(PYTHON_INSTALLER_NAME))
            .expect("Couldn't create installer file"),
    );

    std::io::copy(
        &mut Cursor::new(response_bytes.await.unwrap()),
        &mut installer_file,
    )
    .expect("Couldn't copy installer contents");
    drop(installer_file);

    let install_version_command = format!(
        "{} install python -Version {} -OutputDirectory {}",
        install_path.join(PYTHON_INSTALLER_NAME).to_string_lossy(),
        version_to_install,
        install_path.to_string_lossy()
    );
    app.run_external_command(install_version_command).await;
}
