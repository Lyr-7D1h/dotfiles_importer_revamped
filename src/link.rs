use std::fs;
use std::io;
use std::io::Error;
use std::path::{Path, PathBuf};

use crate::Config;

pub fn link(config: &Config) -> Result<(), Error> {
    let src = config.repository.workdir().unwrap();
    let dest = &config.home;

    println!("Linking from {:?} to {:?}", src, dest);
    link_recurse(src, dest, Path::new(""), &config.ignore_files)
}

fn link_recurse(
    src: &Path,
    dest: &Path,
    cur: &Path,
    ignore_files: &Vec<PathBuf>,
) -> io::Result<()> {
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
                link_recurse(src, dest, &cur, ignore_files)?;
            } else if path.is_file() {
                println!("Linking file: {:?} to {:?}", path, dest.join(cur));
                // link
            }
        }
    }

    Ok(())
}
