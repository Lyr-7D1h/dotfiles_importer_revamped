use crate::importer::state::Difference;
use git2::Cred;
use git2::RemoteCallbacks;
use git2::Repository;
use std::error::Error;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use log::{debug, info};

pub fn find_equal_files<F>(
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
                find_equal_files(src, dest, &cur, ignore_files, op)?;
            } else if path.is_file() {
                op(&path, &dest.join(cur).join(path.file_name().unwrap()), cur)?;
            }
        }
    }
    Ok(())
}

/// Find all files without following symlinks
pub fn find_all_files_symlink<F>(path: &Path, op: &mut F) -> io::Result<()>
where
    F: FnMut(&Path) -> io::Result<()>,
{
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let path = entry?.path();

            if path.is_dir() {
                find_all_files_symlink(&path, op)?;
            } else if path.symlink_metadata()?.is_file() {
                op(&path)?;
            }
        }
    }
    Ok(())
}

pub fn repository_push(
    repository: &git2::Repository,
    private_key_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut remote = repository.find_remote("origin")?;
    let mut po = git2::PushOptions::new();
    po.remote_callbacks(get_callbacks(private_key_path));
    remote.push(&["refs/heads/master:refs/heads/master"], Some(&mut po))?;
    Ok(())
}
pub fn repository_commit(
    repository: &git2::Repository,
    description: &str,
) -> Result<(), Box<dyn Error>> {
    let signature = get_signature()?;
    let mut index = repository.index()?;
    index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let oid = index.write_tree()?;
    let parent_commit = repository.head()?.peel_to_commit()?;
    let tree = repository.find_tree(oid)?;

    repository.commit(
        Some("HEAD"),
        &signature,
        &signature,
        description,
        &tree,
        &[&parent_commit],
    )?;

    Ok(())
}
pub fn repository_fetch(
    url: &str,
    path: &Path,
    private_key_path: &Path,
) -> Result<Repository, Box<dyn Error>> {
    if url.len() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Repository url can not be empty",
        )
        .into());
    }

    let repo = match Repository::open(&path) {
        Ok(r) => r,
        Err(_) => {
            info!("Repository path does not exist cloning...");

            let mut fo = git2::FetchOptions::new();
            fo.remote_callbacks(get_callbacks(private_key_path));

            let mut builder = git2::build::RepoBuilder::new();
            builder.fetch_options(fo);

            let repo = builder.clone(&url, &path)?;
            return Ok(repo);
        }
    };
    // if current repo differs remove and fetch again
    if repo.find_remote("origin")?.url().unwrap() != url {
        fs::remove_dir_all(&path)?;
        return repository_fetch(url, path, private_key_path);
    }
    Ok(repo)
}

pub fn differences_to_string(differences: &Vec<Difference>) -> String {
    differences
        .iter()
        .map(|diff| format!("[{}] {}", diff.kind, diff.path))
        .collect::<Vec<String>>()
        .join("\n")
}

fn get_callbacks<'a>(private_key_path: &'a Path) -> RemoteCallbacks<'a> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |url, username_from_url, _allowed_types| {
        debug!("Asking ssh credentials for: {:?}", url);
        Cred::ssh_key(
            username_from_url.unwrap_or("git"),
            None,
            private_key_path,
            None,
        )
    });
    callbacks
}

fn get_signature<'a>() -> Result<git2::Signature<'a>, git2::Error> {
    let config = git2::Config::open_default()?;

    let name = config.get_entry("user.name")?;
    let name = name.value().unwrap();
    let email = config.get_entry("user.email")?;
    let email = email.value().unwrap();

    info!("Using name: {} email: {} for signature", name, email);

    git2::Signature::now(&name, &email)
}
