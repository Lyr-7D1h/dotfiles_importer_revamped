use std::{fs, path::Path};

use crate::{util::get_repository, Importer};

pub fn status(importer: &Importer) -> Result<String, String> {
    let changed_files = importer.state.changed_files.join("\n");
    let home_prefix = format!("{}/", importer.config.home.to_str().unwrap());
    let suggested_files: Vec<String> = importer
        .state
        .suggested_files
        .iter()
        .map(|file| file.strip_prefix(&home_prefix).unwrap().to_string())
        .collect();

    let suggested_files = suggested_files.join("\n");
    return Ok(format!(
        "Changed Files:\n{}\n\nSuggested Files:\n{}",
        changed_files, suggested_files
    ));
}

pub fn set_repository(repo: &str, importer: &mut Importer) -> Result<String, String> {
    let test_path = Path::new("/tmp/dimport/repo");
    if test_path.exists() {
        if let Err(e) = fs::remove_dir_all(test_path) {
            return Err(e.to_string());
        }
    }
    match get_repository(repo, test_path) {
        Ok(_) => {
            // Reset home to how it was before
            if let Err(e) = importer.restore() {
                return Err(format!("Could not restore files: {}", e));
            }
            // change current repository
            if let Err(e) = importer.config.set_repository(repo) {
                return Err(format!("Could not change repository: {}", e));
            }

            // setup with new repository
            importer.state.initialized = false;
            if let Err(e) = importer.state.save() {
                return Err(format!("Failed saving: {}", e));
            }
            if let Err(e) = importer.setup() {
                return Err(format!("Setting up with new repository failed: {}", e));
            }
            return Ok("Succesfully changed repository".into());
        }
        Err(e) => return Err(e.to_string()),
    };
}

pub fn set_home(home: &str, importer: &mut Importer) {}
