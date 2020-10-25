use crate::util::find_all_files_symlink;
use log::{debug, info};

use crate::{util::find_equal_files, Importer, BACKUP_DIR};
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
                let mut backup_path = Path::new(BACKUP_DIR).join(cur);
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
        info!("Restoring from backup");
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
                    Path::new(BACKUP_DIR),
                    &self.config.home,
                    Path::new(""),
                    &self.config.ignore_files,
                    &mut restore_from_backup,
                );
            }
            let backup_path = Path::new(BACKUP_DIR)
                .join(_cur)
                .join(to.file_name().unwrap());

            if backup_path.exists() {
                debug!("Copying {:?} from backup to {:?}", backup_path, to);
                fs::copy(backup_path, to)?;
            }

            Ok(())
        };

        self.recurse_with_config(&op)
    }
    pub fn intitialize_mapped(&mut self) -> Result<(), Error> {
        let home = self.config.home.clone();

        self.state.mapped_files = vec![];
        self.state.save()?;

        let mut op = |file: &Path| {
            let file = file.to_str().unwrap().to_string();
            debug!("Adding {} to Mapped Files", file);
            self.state.mapped_files.push(file);
            self.state.save()?;
            Ok(())
        };

        find_all_files_symlink(&home, &mut op)
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
