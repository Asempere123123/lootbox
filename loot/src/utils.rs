use std::fs;
use std::io::Write;

pub fn create_file_with_content(name: &std::path::PathBuf, content: &[u8]) -> std::io::Result<()> {
    let mut file = fs::File::create(name)?;
    file.write_all(content)?;
    Ok(())
}
