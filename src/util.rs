use std::{
    fs, io,
    path::{Path, PathBuf},
};

use log::debug;

pub fn recurse<F>(
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
                debug!("Ignoring {:?}", path);
                continue;
            }

            if path.is_dir() {
                let cur = path.strip_prefix(src).unwrap();
                recurse(src, dest, &cur, ignore_files, op)?;
            } else if path.is_file() {
                op(&path, &dest.join(cur).join(path.file_name().unwrap()), cur)?;
            }
        }
    }
    Ok(())
}
