use std::fs;
use std::io::Write;

pub fn create_file_with_content(name: &std::path::PathBuf, content: &[u8]) -> std::io::Result<()> {
    let mut file = fs::File::create(name)?;
    file.write_all(content)?;
    Ok(())
}

pub fn clone_dir(origin: &std::path::PathBuf, target: &std::path::PathBuf) -> std::io::Result<()> {
    // Create the target directory if it doesn't exist
    if !target.exists() {
        fs::create_dir_all(target)?;
    }

    // Read the contents of the origin directory
    for entry in fs::read_dir(origin)? {
        let entry = entry?;
        let path = entry.path();
        let mut target_path = target.clone();
        target_path.push(path.file_name().unwrap());

        if path.is_dir() {
            if path.file_name().unwrap() == "__pycache__" {
                continue;
            }

            // Recursively clone directories
            clone_dir(&path, &target_path)?;
        } else {
            // Copy files
            fs::copy(&path, &target_path)?;
        }
    }

    Ok(())
}
