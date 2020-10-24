use crate::{CONFIG_PATH, PRIVATE_KEY_PATH, REPOSITORY_DIR};
use git2::{Cred, RemoteCallbacks, Repository};
use log::info;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::{env, error::Error};

pub struct Config {
    pub repository: Repository,
    pub home: PathBuf,
    pub ignore_files: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize)]
struct UnserializedConfig {
    repository: String,
    home: String,
    ignore_files: Vec<String>,
}

impl Config {
    pub fn from_settings() -> Result<Config, Box<dyn Error>> {
        let file = match File::open(CONFIG_PATH) {
            Ok(file) => file,
            Err(_) => {
                let default_config = UnserializedConfig {
                    repository: String::new(),
                    home: env::var("HOME")?,
                    ignore_files: vec![
                        "README.md".to_string(),
                        ".gitignore".to_string(),
                        ".git".to_string(),
                    ],
                };
                let data = serde_json::to_string(&default_config)?;
                fs::write(CONFIG_PATH, data)?;
                File::open(CONFIG_PATH)?
            }
        };

        let reader = BufReader::new(file);

        let uconfig: UnserializedConfig = serde_json::from_reader(reader)?;

        let home = PathBuf::from(uconfig.home);
        home.metadata()?;

        info!("Fetched config");

        let repository = get_repository(uconfig.repository)?;
        let ignore_files: Vec<PathBuf> = uconfig
            .ignore_files
            .iter()
            .map(|file| repository.workdir().unwrap().join(PathBuf::from(file)))
            .collect();

        info!("Fetched repository");

        let config = Config {
            repository,
            home,
            ignore_files,
        };

        Ok(config)
    }
}

fn get_repository(url: String) -> Result<Repository, Box<dyn Error>> {
    let path = PathBuf::from(REPOSITORY_DIR);
    match Repository::open(&path) {
        Ok(r) => Ok(r),
        Err(_) => {
            info!("Repository path does not exist cloning...");

            let mut callbacks = RemoteCallbacks::new();
            callbacks.credentials(|url, username_from_url, _allowed_types| {
                info!("Asking ssh credentials for: {:?}", url);
                Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    Path::new(PRIVATE_KEY_PATH),
                    None,
                )
            });

            let mut fo = git2::FetchOptions::new();
            fo.remote_callbacks(callbacks);

            let mut builder = git2::build::RepoBuilder::new();
            builder.fetch_options(fo);

            let repo = builder.clone(&url, &path)?;
            return Ok(repo);
        }
    }
}
