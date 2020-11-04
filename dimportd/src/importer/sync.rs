use crate::{
    importer::state::Difference,
    util::{find_equal_dir, repository_update},
    REPOSITORY_DIR,
};
use std::path::Path;
use std::{error::Error, os::unix::fs::symlink};
use std::{fs, io};

use log::{debug, info};

use crate::util::find_equal_files;
use crate::Importer;

impl Importer {
    /// Synchronize and notify if new changes and save to state
    pub fn sync_and_notify(&mut self) -> Result<(), Box<dyn Error>> {
        let has_changes = self.sync()?;

        if has_changes {
            let mut body = format!(
                "You have {} changed files.",
                self.state.differences.len() + self.state.picked_differences.len()
            );
            if self.state.suggested_files.len() > 0 {
                body.push_str(&format!(
                    "\nAnd {} suggested files.",
                    self.state.suggested_files.len()
                ));
            }
            self.notify(&body)?;
        }

        Ok(())
    }

    /// Remove files if link removed
    /// Update Suggested files
    /// Return true if there are new changed files
    pub fn sync(&mut self) -> Result<bool, Box<dyn Error>> {
        info!("Synchronizing..");
        self.link_removed()?;
        repository_update(&self.config.repository, &self.config.private_key_path)?;
        self.update_suggested()?;
        self.link_newly_added()?;

        let statuses = self.config.repository.statuses(None)?;

        // only add
        let mut differences = vec![];
        let mut picked_differences = vec![];
        let mut new_differences = vec![];
        'outer: for status in statuses.iter() {
            if let Some(path) = status.path() {
                for diff in &self.state.differences {
                    if diff.path == path {
                        differences.push(diff.clone());
                        continue 'outer;
                    }
                }
                for diff in &self.state.picked_differences {
                    if diff.path == path {
                        picked_differences.push(diff.clone());
                        continue 'outer;
                    }
                }
                new_differences.push(Difference::from_status_entry(status))
            }
        }
        let has_changes = new_differences.len() > 0;
        if has_changes {
            self.state.differences.append(&mut new_differences);
        }
        self.state.differences = differences;
        self.state.picked_differences = picked_differences;
        self.state.save()?;

        Ok(has_changes)
    }

    /// if symlink removed -> remove file from repository
    fn link_removed(&self) -> Result<(), io::Error> {
        let src = self.config.repository.workdir().unwrap();
        let dest = &self.config.home_path;

        let mut op = |from: &Path, to: &Path, _cur: &Path| {
            if let Ok(meta) = to.symlink_metadata() {
                if !meta.file_type().is_symlink() {
                    info!("Symlink removed, removing: {:?}", from);
                    fs::remove_file(from)?;
                }
            } else {
                info!("Symlink removed, removing: {:?}", from);
                fs::remove_file(from)?;
            }
            Ok(())
        };

        find_equal_files(src, dest, Path::new(""), &self.config.ignore_files, &mut op)
    }

    /// Link files that are newly added to the repository
    fn link_newly_added(&self) -> Result<(), io::Error> {
        let src = self.config.repository.workdir().unwrap();
        let dest = &self.config.home_path;

        let mut op = |from: &Path, to: &Path, _cur: &Path| {
            if !to.exists() {
                info!("New file found {:?}. Linking to {:?}", from, to);
                symlink(from, to)?;
            }
            Ok(())
        };

        find_equal_files(src, dest, Path::new(""), &self.config.ignore_files, &mut op)
    }

    /// If destination directory has new files add to suggested
    fn update_suggested(&mut self) -> Result<(), io::Error> {
        let home = self.config.home_path.clone();

        let mut op = |dir: &Path| {
            for entry in fs::read_dir(dir)? {
                let path = entry?.path();
                if path.is_file() {
                    let path = path.to_str().unwrap().to_string();
                    if !self.state.suggested_files.contains(&path)
                        && !self.state.mapped_files.contains(&path)
                    {
                        debug!("Adding {} to Suggested Files", path);
                        self.state.suggested_files.push(path);
                    }
                }
            }
            Ok(())
        };

        find_equal_dir(Path::new(REPOSITORY_DIR), &home, Path::new(""), &mut op)?;
        self.state.save()?;
        Ok(())
    }
}
