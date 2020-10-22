use log::{debug, info};

use crate::{util::find_equal_files, Importer};
use std::fs;
use std::io;
use std::io::Error;
use std::os::unix::fs::symlink;
use std::path::Path;

impl Importer {
    pub fn backup(&self) -> Result<(), Error> {
        let mut c = 0;
        let backup = |_from: &Path, to: &Path, cur: &Path| {
            if to.exists() {
                let mut backup_path = Path::new("backup").join(cur);
                if !backup_path.exists() {
                    fs::create_dir_all(&backup_path)?;
                }
                backup_path = backup_path.join(to.file_name().unwrap());
                debug!("Backing up {:?} {:?}", to, backup_path);
                fs::copy(to, backup_path)?;
                c = c + 1;
            }

            Ok(())
        };
        self.recurse_with_config(backup)?;
        info!("Backed up {} files", c);
        Ok(())
    }
    pub fn link(&self) -> Result<(), Error> {
        let link = |from: &Path, to: &Path, _cur: &Path| {
            // Remove if exists
            if let Ok(meta) = to.symlink_metadata() {
                let meta = meta.file_type();
                if meta.is_file() || meta.is_symlink() {
                    fs::remove_file(to)?;
                } else if meta.is_dir() {
                    fs::remove_dir_all(to)?;
                }
            }
            debug!("Linking file: {:?} to {:?}", from, to);
            if !to.parent().unwrap().exists() {
                fs::create_dir_all(to.parent().unwrap())?;
            }
            symlink(from, to)
        };

        self.recurse_with_config(&link)
    }
    pub fn restore(&self) -> Result<(), Error> {
        let op = |_from: &Path, to: &Path, _cur: &Path| {
            // Remove all symbolic links made
            if let Ok(meta) = to.symlink_metadata() {
                if meta.file_type().is_symlink() {
                    fs::remove_file(to)?;
                }

                let mut restore_from_backup = |from: &Path, to: &Path, _cur: &Path| {
                    fs::copy(from, to)?;
                    Ok(())
                };

                return find_equal_files(
                    Path::new("backup"),
                    &self.config.home,
                    Path::new(""),
                    &self.config.ignore_files,
                    &mut restore_from_backup,
                );
            }
            let backup_path = Path::new("backup").join(_cur).join(to.file_name().unwrap());

            if backup_path.exists() {
                debug!("Copying {:?} from backup to {:?}", backup_path, to);
                fs::copy(backup_path, to)?;
            }

            Ok(())
        };

        self.recurse_with_config(&op)
    }

    fn recurse_with_config<F>(&self, mut op: F) -> Result<(), Error>
    where
        F: FnMut(&Path, &Path, &Path) -> io::Result<()>,
    {
        let src = self.config.repository.workdir().unwrap();
        let dest = &self.config.home;

        find_equal_files(src, dest, Path::new(""), &self.config.ignore_files, &mut op)
    }
}
