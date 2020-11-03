use crate::util::repository_fetch;
use crate::CONFIG_PATH;
use crate::REPOSITORY_DIR;
use git2::Repository;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::{
    fs::{self, File},
    io,
};

pub struct Config {
    pub repository: Repository,
    pub home_path: PathBuf,
    pub private_key_path: PathBuf,
    pub ignore_files: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize)]
struct UnserializedConfig {
    repository: String,
    home_path: String,
    private_key_path: String,
    ignore_files: Vec<String>,
}

impl Config {
    pub fn from_settings() -> Result<Config, Box<dyn Error>> {
        if let Some(dir_path) = Path::new(CONFIG_PATH).parent() {
            fs::create_dir_all(dir_path)?;
        }
        let file = match File::open(CONFIG_PATH) {
            Ok(file) => file,
            Err(_) => {
                let default_config = UnserializedConfig {
                    repository: String::new(),
                    home_path: String::new(),
                    private_key_path: String::new(),
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

        let home_path = PathBuf::from(uconfig.home_path);
        home_path
            .metadata()
            .map_err(|e| return io::Error::new(e.kind(), format!("Invalid Home Path: {}", e)))?;

        let private_key_path = PathBuf::from(uconfig.private_key_path);
        private_key_path.metadata().map_err(|e| {
            return io::Error::new(e.kind(), format!("Invalid Private Key Path: {}", e));
        })?;

        fs::create_dir_all(REPOSITORY_DIR)?;

        let repository = repository_fetch(
            &uconfig.repository,
            Path::new(REPOSITORY_DIR),
            &private_key_path,
        )?;

        let ignore_files: Vec<PathBuf> = uconfig
            .ignore_files
            .iter()
            .map(|file| repository.workdir().unwrap().join(PathBuf::from(file)))
            .collect();

        debug!("Fetched repository");

        let config = Config {
            repository,
            home_path,
            private_key_path,
            ignore_files,
        };

        Ok(config)
    }

    /// Only when config is broken, there is no proper validation
    pub fn write(property: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let config_file = File::open(CONFIG_PATH)?;
        let reader = BufReader::new(&config_file);
        let mut old_data: Value = serde_json::from_reader(reader)?;
        old_data[property] = Value::String(value.to_string());
        let new_data = serde_json::to_vec_pretty(&old_data)?;
        fs::write(CONFIG_PATH, new_data)?;
        Ok(())
    }

    pub fn show_raw() -> Result<String, Box<dyn Error>> {
        let config_file = File::open(CONFIG_PATH)?;
        let reader = BufReader::new(&config_file);
        let old_data: Value = serde_json::from_reader(reader)?;
        let pretty_string = serde_json::to_string(&old_data)?;
        Ok(pretty_string)
    }

    pub fn set_home(&mut self, home: &str) -> Result<(), Box<dyn Error>> {
        let path = PathBuf::from(home);
        if !path.is_dir() {
            return Err("Path does not exist or is not a directory".into());
        }
        self.home_path = path;
        let config_file = File::open(CONFIG_PATH)?;
        let reader = BufReader::new(&config_file);
        let mut uconfig: UnserializedConfig = serde_json::from_reader(reader)?;
        uconfig.home_path = home.to_string();
        let data = serde_json::to_vec_pretty(&uconfig)?;
        fs::write(CONFIG_PATH, data)?;
        Ok(())
    }

    pub fn set_private_key(&mut self, private_key: &str) -> Result<(), Box<dyn Error>> {
        let path = PathBuf::from(private_key);
        if !path.is_dir() {
            return Err("Path does not exist or is not a directory".into());
        }
        self.private_key_path = path;
        let config_file = File::open(CONFIG_PATH)?;
        let reader = BufReader::new(&config_file);
        let mut uconfig: UnserializedConfig = serde_json::from_reader(reader)?;
        uconfig.private_key_path = private_key.to_string();
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
        self.repository = repository_fetch(repository_url, repo_path, &self.private_key_path)?;
        let config_file = File::open(CONFIG_PATH)?;
        let reader = BufReader::new(&config_file);
        let mut uconfig: UnserializedConfig = serde_json::from_reader(reader)?;
        uconfig.repository = repository_url.to_string();
        let data = serde_json::to_vec_pretty(&uconfig)?;
        fs::write(CONFIG_PATH, data)?;
        Ok(())
    }
}
