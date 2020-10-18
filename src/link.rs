use std::fs;
use std::io;
use std::io::Error;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use crate::Config;

pub fn backup(config: &Config) -> Result<(), Error> {
    let mut c = 0;
    let backup = |_from: &Path, to: &Path, cur: &Path| {
        if to.exists() {
            let mut backup_path = Path::new("backup").join(cur);
            if !backup_path.exists() {
                fs::create_dir_all(&backup_path)?;
            }
            backup_path = backup_path.join(to.file_name().unwrap());
            println!("Backing up {:?} {:?}", to, backup_path);
            fs::copy(to, backup_path)?;
            c = c + 1;
        }

        Ok(())
    };
    let res = recurse_with_config(config, backup)?;
    println!("Backed up {} files", c);
    Ok(())
}

pub fn link(config: &Config) -> Result<(), Error> {
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
        println!("Linking file: {:?} to {:?}", from, to);
        if !to.parent().unwrap().exists() {
            fs::create_dir_all(to.parent().unwrap())?;
        }
        let res = symlink(from, to);
        res
    };

    recurse_with_config(config, &link)
}

pub fn restore(config: &Config) -> Result<(), Error> {
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

            return recurse(
                Path::new("backup"),
                &config.home,
                Path::new(""),
                &config.ignore_files,
                &mut restore_from_backup,
            );
        }
        let backup_path = Path::new("backup").join(_cur).join(to.file_name().unwrap());

        if backup_path.exists() {
            println!("Copying {:?} from backup to {:?}", backup_path, to);
            fs::copy(backup_path, to)?;
        }

        Ok(())
    };

    recurse_with_config(config, &op)
}

fn recurse_with_config<F>(config: &Config, mut op: F) -> Result<(), Error>
where
    F: FnMut(&Path, &Path, &Path) -> io::Result<()>,
{
    let src = config.repository.workdir().unwrap();
    let dest = &config.home;

    recurse(src, dest, Path::new(""), &config.ignore_files, &mut op)
}

fn recurse<F>(
    src: &Path,
    dest: &Path,
    cur: &Path,
    ignore_files: &Vec<PathBuf>,
    op: &mut F,
) -> io::Result<()>
where
    F: FnMut(&Path, &Path, &Path) -> io::Result<()>,
{
    let cur_dir = src.join(cur);

    if cur_dir.is_dir() {
        for entry in fs::read_dir(cur_dir)? {
            let entry = entry?;
            let path = entry.path();

            if ignore_files.contains(&path.to_path_buf()) {
                println!("Ignoring {:?}", path);
                continue;
            }

            if path.is_dir() {
                let cur = path.strip_prefix(src).unwrap();
                recurse(src, dest, &cur, ignore_files, op)?;
            } else if path.is_file() {
                // println!("{:?} {:?}", dest, cur);
                op(&path, &dest.join(cur).join(path.file_name().unwrap()), cur)?;
            }
        }
    }

    Ok(())
}
