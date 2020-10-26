use crate::util::get_repository;
use crate::CONFIG_PATH;
use crate::REPOSITORY_DIR;
use git2::Repository;
use log::info;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
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

        let repository = get_repository(&uconfig.repository, Path::new(REPOSITORY_DIR))?;
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

    pub fn set_home(&mut self, home: &str) -> Result<(), Box<dyn Error>> {
        let path = PathBuf::from(home);
        if !path.is_dir() {
            return Err("Path does not exist or is not a directory".into());
        }
        self.home = path;
        let config_file = File::open(CONFIG_PATH)?;
        let reader = BufReader::new(&config_file);
        let mut uconfig: UnserializedConfig = serde_json::from_reader(reader)?;
        uconfig.home = home.to_string();
        let data = serde_json::to_vec_pretty(&uconfig)?;
        fs::write(CONFIG_PATH, data)?;
        Ok(())
    }

    /// Removes current repository and sets a new one in its place and saves to CONFIG_PATH
    pub fn set_repository(&mut self, repository_url: &str) -> Result<(), Box<dyn Error>> {
        let repo_path = Path::new(REPOSITORY_DIR);
        if repo_path.exists() {
            fs::remove_dir_all(repo_path)?;
        }
        self.repository = get_repository(repository_url, repo_path)?;
        let config_file = File::open(CONFIG_PATH)?;
        let reader = BufReader::new(&config_file);
        let mut uconfig: UnserializedConfig = serde_json::from_reader(reader)?;
        uconfig.repository = repository_url.to_string();
        let data = serde_json::to_vec_pretty(&uconfig)?;
        fs::write(CONFIG_PATH, data)?;
        Ok(())
    }
}
