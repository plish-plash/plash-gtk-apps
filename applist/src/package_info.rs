use std::{path::Path, process::Command};

pub fn get_package_info(executable: &Path) -> Result<Vec<(String, String)>, String> {
    // Run `pacman -Qqo <file>` to get the name of the package that owns file.
    let output = Command::new("pacman")
        .arg("-Qqo")
        .arg(executable)
        .output()
        .expect("failed to run pacman");
    if !output.status.success() {
        return Err(String::from_utf8(output.stderr).expect("pacman output not UTF-8"));
    }
    let package_name = String::from_utf8(output.stdout).expect("pacman output not UTF-8");

    // Run `pacman -Qi <package>` to get detailed information about the package.
    let output = Command::new("pacman")
        .arg("-Qi")
        .arg(package_name.trim())
        .output()
        .expect("failed to run pacman");
    if !output.status.success() {
        return Err(String::from_utf8(output.stderr).expect("pacman output not UTF-8"));
    }
    let output = String::from_utf8(output.stdout).expect("pacman output not UTF-8");
    let mut result = Vec::new();
    for line in output.lines() {
        if let Some((key, value)) = line.split_once(':') {
            result.push((key.trim().to_owned(), value.trim().to_owned()));
        }
    }
    Ok(result)
}
