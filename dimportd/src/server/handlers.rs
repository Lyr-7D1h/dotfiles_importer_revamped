use crate::util::repository_commit;
use crate::util::repository_fetch;
use crate::util::{differences_to_string, repository_push};
use std::{fs, path::Path};

use git2::build::CheckoutBuilder;
use log::info;
use regex::Regex;

use crate::Importer;

/// Sync and return status
pub fn status(importer: &mut Importer) -> Result<String, String> {
    let changes = match importer.sync() {
        Ok(changes) => changes,
        Err(e) => return Err(format!("Could not sync: {}", e)),
    };

    importer.state.differences = changes;
    if let Err(e) = importer.state.save() {
        return Err(format!("Could not save changes"));
    }

    let mut result = String::new();

    if importer.state.differences.len() > 0 {
        result.push_str("Changed Files\n");
        result.push_str(&differences_to_string(&importer.state.differences));
        if importer.state.suggested_files.len() > 0 {
            result.push_str("\n");
            result.push_str("\n");
        }
    }
    let home_prefix = format!("{}/", importer.config.home_path.to_str().unwrap());

    if importer.state.suggested_files.len() > 0 {
        let suggested_files = importer
            .state
            .suggested_files
            .iter()
            .map(|file| file.strip_prefix(&home_prefix).unwrap().to_string())
            .collect::<Vec<String>>()
            .join("\n");
        result.push_str("Suggested Files\n");
        result.push_str(&suggested_files);
    }
    if result.len() == 0 {
        return Ok("Everything is up to date and no suggestions".into());
    }
    Ok(result)
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
Private Key Path: {:?}
Ignored Files: {}
        "#,
                url, importer.config.home_path, importer.config.private_key_path, ignore_files
            );
            return Ok(res);
        }
    }
    return Err("Problem resolving repository".into());
}

pub fn sync(importer: &mut Importer) -> Result<String, String> {
    if let Err(e) = importer.sync_and_notify() {
        return Err(format!("Could not sync: {}", e));
    }
    Ok("Synchronization succeeded".into())
}

pub fn set_repository(repo: &str, importer: &mut Importer) -> Result<String, String> {
    let test_path = Path::new("/tmp/dimport/repo");
    if test_path.exists() {
        if let Err(e) = fs::remove_dir_all(test_path) {
            return Err(e.to_string());
        }
    }
    match repository_fetch(repo, test_path, &importer.config.private_key_path) {
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

pub fn set_private_key(private_key_path: &str, importer: &mut Importer) -> Result<String, String> {
    if let Err(e) = importer.config.set_private_key(private_key_path) {
        return Err(format!("Could not set home: {}", e));
    }

    return Ok("Succesfully changed private key path".into());
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

    let removed_amount = removed_suggested.len();
    importer.state.mapped_files.append(&mut removed_suggested);
    if let Err(e) = importer.state.save() {
        return Err(format!("Could not save state: {}", e));
    }

    Ok(format!("Ignored {} suggested files", removed_amount))
}

pub fn restore(regex: &str, importer: &Importer) -> Result<String, String> {
    let regex = Regex::new(regex).unwrap();
    let mut restore_file_paths = vec![];
    let mut builder = CheckoutBuilder::new();
    builder.force();

    for diff in &importer.state.differences {
        if regex.is_match(&diff.path) {
            builder.path(&diff.path);
            restore_file_paths.push(diff.path.clone());
        }
    }

    if let Err(e) = importer
        .config
        .repository
        .checkout_index(None, Some(&mut builder))
    {
        return Err(format!("Could not checkout files from repository: {}", e));
    }

    for file in restore_file_paths.iter() {
        if let Err(e) = importer.link(file) {
            return Err(format!("Could not link restored files: {}", e));
        }
    }

    Ok(format!(
        "{}\nRestored {} Files.",
        restore_file_paths.join("\n"),
        restore_file_paths.len(),
    ))
}

pub fn add(absolute_src_path_string: &str, importer: &mut Importer) -> Result<String, String> {
    let home_path_string = importer.config.home_path.to_str().unwrap();
    if !absolute_src_path_string.starts_with(home_path_string) {
        return Err("Path is not in home folder".into());
    }
    let relative_path = absolute_src_path_string
        .strip_prefix(&format!("{}{}", home_path_string, "/"))
        .unwrap();
    let repository_path = importer
        .config
        .repository
        .workdir()
        .unwrap()
        .join(relative_path);

    let absolute_src_path = Path::new(absolute_src_path_string);

    if !absolute_src_path.exists() {
        return Err(format!("Could not find {:?}", absolute_src_path));
    }
    if let Err(e) = fs::copy(&absolute_src_path, &repository_path) {
        return Err(format!("Could not copy file: {}", e));
    }
    if let Err(e) = fs::remove_file(&absolute_src_path) {
        return Err(format!("Could not remove source file: {}", e));
    }
    if let Err(e) = importer.link(relative_path) {
        return Err(format!("Could not link file: {}", e));
    }

    // Remove from suggested if it exists
    let mut removed_suggested = vec![];
    importer.state.suggested_files.retain(|file_path| {
        if file_path == absolute_src_path_string {
            removed_suggested.push(file_path.clone());
            false
        } else {
            true
        }
    });

    importer.state.mapped_files.append(&mut removed_suggested);

    if let Err(e) = importer.state.save() {
        return Err(format!("Could not save state: {}", e));
    }

    Ok("Succesfully added path.".into())
}

pub fn save(description: Option<&str>, importer: &Importer) -> Result<String, String> {
    let description = match description {
        Some(description) => description.to_string(),
        None => differences_to_string(&importer.state.differences),
    };

    if let Err(e) = repository_commit(&importer.config.repository, &description) {
        return Err(format!("Could not commit files: {}", e));
    }

    info!("Commited");

    if let Err(e) = repository_push(
        &importer.config.repository,
        &importer.config.private_key_path,
    ) {
        return Err(format!("Could not push repository: {}", e));
    }
    info!("Pushed commit");

    Ok("Succesfully saved.".into())
}
