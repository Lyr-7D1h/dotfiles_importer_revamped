use std::error::Error;
use std::path::Path;
use std::{fs, io};

use log::info;

use crate::util::{find_all_files_symlink, find_equal_files};
use crate::Importer;

impl Importer {
    // Synchronize every 5 minutes
    pub fn sync(&mut self) -> Result<Vec<String>, Box<dyn Error>> {
        info!("Synchronizing..");
        self.link_removed()?;
        self.update_suggested()?;

        let statuses = self.config.repository.statuses(None)?;

        let filenames = statuses
            .iter()
            .map(|status| status.path().unwrap().to_string())
            .collect();

        Ok(filenames)
    }

    /// if symlink removed -> remove file from repository
    fn link_removed(&self) -> Result<(), io::Error> {
        let src = self.config.repository.workdir().unwrap();
        let dest = &self.config.home;

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
        let home = self.config.home.clone();

        let mut op = |file: &Path| {
            let file = file.to_str().unwrap().to_string();
            if !self.state.suggested_files.contains(&file)
                && !self.state.mapped_files.contains(&file)
            {
                info!("Adding {} to Suggested Files", file);
                self.state.suggested_files.push(file);
                self.state.save()?;
            }
            Ok(())
        };

        find_all_files_symlink(&home, &mut op)
    }
}
