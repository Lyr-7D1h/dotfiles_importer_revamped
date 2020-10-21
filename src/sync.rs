use std::error::Error;
use std::path::Path;
use std::{fs, io};

use log::info;

use crate::util::recurse;
use crate::Importer;

impl Importer {
    // Synchronize every 5 minutes
    pub fn sync(&self) -> Result<Vec<String>, Box<dyn Error>> {
        info!("Synchronizing..");

        if self.link_removed()? {
            // let state = self.config.repository.statuses(None)?;
            // println!("{:?}", state.len());
        }
        let statuses = self.config.repository.statuses(None)?;

        let filenames = statuses
            .iter()
            .map(|status| status.path().unwrap().to_string())
            .collect();

        Ok(filenames)
    }

    // if symlink removed -> remove file from repository
    fn link_removed(&self) -> Result<bool, io::Error> {
        let src = self.config.repository.workdir().unwrap();
        let dest = &self.config.home;

        let mut removed = false;
        let mut op = |from: &Path, to: &Path, _cur: &Path| {
            // Remove repo file if file does not exist or not a symlink
            if let Ok(meta) = to.symlink_metadata() {
                if !meta.file_type().is_symlink() {
                    fs::remove_file(from)?;
                    removed = true
                }
            } else {
                fs::remove_file(from)?;
                removed = true
            }
            Ok(())
        };

        recurse(src, dest, Path::new(""), &self.config.ignore_files, &mut op)?;
        Ok(removed)
    }
}
