use crate::{importer::state::Difference, util::find_equal_dir, REPOSITORY_DIR};
use std::error::Error;
use std::path::Path;
use std::{fs, io};

use log::{debug, info};

use crate::util::find_equal_files;
use crate::Importer;

impl Importer {
    /// Synchronize and notify if new changes and save to state
    pub fn sync_and_notify(&mut self) -> Result<(), Box<dyn Error>> {
        let changes = self.sync()?;

        if changes.len() > self.state.differences.len() {
            let mut body = format!("You have {} changed files.", changes.len());
            if self.state.suggested_files.len() > 0 {
                body.push_str(&format!(
                    "\nAnd {} suggested files.",
                    self.state.suggested_files.len()
                ));
            }
            self.notify(&body)?;
        }

        // Always update changes
        self.state.differences = changes;
        self.state.save()?;

        Ok(())
    }

    /// Remove files if link removed
    /// Update Suggested files
    /// Return file status for changes
    pub fn sync(&mut self) -> Result<Vec<Difference>, Box<dyn Error>> {
        info!("Synchronizing..");
        self.link_removed()?;
        self.update_suggested()?;

        let statuses = self.config.repository.statuses(None)?;

        let differences = statuses
            .iter()
            .map(|status| Difference::from_status_entry(status))
            .collect();

        Ok(differences)
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

        // find_all_files_symlink(&home, &mut op)?;
        find_equal_dir(Path::new(REPOSITORY_DIR), &home, Path::new(""), &mut op)?;
        self.state.save()?;
        Ok(())
    }
}
