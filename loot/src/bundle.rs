use std::fs;
use std::path::Path;

use crate::dependencies::Package;
use crate::utils::run_venv_command_with_output;

pub fn bundle(data_path: &Path) {
    println!(
        "NOTE: You must run project at least once before bundling it to resolve the dependencies."
    );

    let _ = fs::remove_dir_all("./bundle");

    copy_dir_all(Path::new("./src"), Path::new("./bundle")).expect("Error cloning source code");

    let ouput = run_venv_command_with_output(data_path, "pipdeptree --json-tree")
        .expect("Error reading dependencies");

    if !ouput.status.success() {
        panic!("Error reading dependencies")
    }

    let dependencies_json: Vec<Package> =
        serde_json::from_slice(&ouput.stdout).expect("Error reading dependencies");

    let cleanup_needed = !crate::dependencies::project_uses_pipdeptree(&dependencies_json);
    if cleanup_needed {
        let uses_packaging = crate::dependencies::project_uses_packaging(&dependencies_json);

        let output = run_venv_command_with_output(data_path, "pip uninstall pipdeptree -y")
            .expect("Error doing cleanup");

        if !output.status.success() {
            panic!("Error doing cleanup");
        }

        if !uses_packaging {
            let output = run_venv_command_with_output(data_path, "pip uninstall packaging -y")
                .expect("Error doing cleanup");

            if !output.status.success() {
                panic!("Error doing cleanup");
            }
        }
    }

    let output = run_venv_command_with_output(data_path, "pip freeze > ./bundle/requirements.txt")
        .expect("Error creating requirements");

    if !output.status.success() {
        panic!("Error creating requirements");
    }

    if cleanup_needed {
        let output = run_venv_command_with_output(data_path, "pip install pipdeptree")
            .expect("Error reinstalling pipdeptree, recreate .lootbox manually");

        if !output.status.success() {
            panic!("Error reinstalling pipdeptree, recreate .lootbox manually");
        }
    }

    println!("Project bundled");
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
