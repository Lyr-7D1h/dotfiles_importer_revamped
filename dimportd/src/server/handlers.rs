use std::{fs, path::Path};

use regex::Regex;

use crate::{util::get_repository, Importer};

pub fn status(importer: &Importer) -> Result<String, String> {
    let mut result = String::new();

    if importer.state.changed_files.len() > 0 {
        result.push_str("Changed Files\n");
        result.push_str(
            &importer
                .state
                .changed_files
                .iter()
                .map(|diff| format!("[{}] {}", diff.kind, diff.path))
                .collect::<Vec<String>>()
                .join("\n"),
        );
    }
    let home_prefix = format!("{}/", importer.config.home.to_str().unwrap());

    if importer.state.suggested_files.len() > 0 {
        let suggested_files: Vec<String> = importer
            .state
            .suggested_files
            .iter()
            .map(|file| file.strip_prefix(&home_prefix).unwrap().to_string())
            .collect();
        result.push_str("Suggested Files");
        result.push_str(&suggested_files.join("\n"));
    }
    if result.len() == 0 {
        return Ok("Everything is up to date and no suggestions".into());
    }
    Ok(result)
}

pub fn backup(importer: &Importer) -> Result<String, String> {
    if let Err(e) = importer.backup() {
        return Err(format!("Could not backup: {}", e));
    }
    Ok("Backup succeeded".into())
}

pub fn config(importer: &Importer) -> Result<String, String> {
    if let Ok(remote) = importer.config.repository.find_remote("origin") {
        if let Some(url) = remote.url() {
            let mut ignore_files = String::new();
            for file in importer.config.ignore_files.iter() {
                ignore_files.push_str("\n");
                ignore_files.push_str(&file.file_name().unwrap().to_string_lossy())
            }

            let res = format!(
                r#"
Repository: {:?}
Home Path: {:?}
Ignored Files: {}
        "#,
                url, importer.config.home, ignore_files
            );
            return Ok(res);
        }
    }
    return Err("Problem resolving repository".into());
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

pub fn set_home(home: &str, importer: &mut Importer) -> Result<String, String> {
    if let Err(e) = importer.config.set_home(home) {
        return Err(format!("Could not set home: {}", e));
    }

    // Reset home to how it was before
    if let Err(e) = importer.restore() {
        return Err(format!("Could not restore files: {}", e));
    }

    if let Err(e) = importer.setup() {
        return Err(format!("Setting up with new home failed: {}", e));
    }

    return Ok("Succesfully changed and setup home folder".into());
}

pub fn ignore_all(importer: &mut Importer) -> Result<String, String> {
    importer
        .state
        .mapped_files
        .append(&mut importer.state.suggested_files);
    importer.state.suggested_files = vec![];
    if let Err(e) = importer.state.save() {
        return Err(format!("Could not save settings: {}", e));
    }
    Ok("Ignored all suggested files".into())
}

pub fn ignore_regex(regex: &str, importer: &mut Importer) -> Result<String, String> {
    let regex = Regex::new(regex).unwrap();
    let mut removed_suggested = vec![];
    importer.state.suggested_files.retain(|file| {
        if regex.is_match(file) {
            removed_suggested.push(file.clone());
            false
        } else {
            true
        }
    });

    importer.state.mapped_files.append(&mut removed_suggested);

    Ok(format!(
        "Ignored {} suggested files",
        removed_suggested.len()
    ))
}

pub fn restore(regex: &str, importer: &Importer) -> Result<String, String> {
    let regex = Regex::new(regex);
    Ok(format!("Restored {} files", 2))
}
