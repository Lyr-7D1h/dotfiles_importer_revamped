use crate::STATE_PATH;
use git2;
use git2::{Delta, StatusEntry};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;
use std::{fs::File, io::BufReader};

#[derive(Serialize, Deserialize, Debug)]
pub struct Difference {
    pub kind: String,
    pub path: String,
}
impl Difference {
    pub fn from_status_entry(entry: StatusEntry) -> Difference {
        let path = entry.path().unwrap().into();
        let status = match entry.index_to_workdir() {
            Some(diff) => diff.status(),
            None => Delta::Unreadable,
        };
        let kind = match status {
            Delta::Modified => "Modified",
            Delta::Added | Delta::Untracked => "Added",
            Delta::Deleted => "Deleted",
            _ => "Undefined",
        }
        .into();

        Difference { kind, path }
    }
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub initialized: bool,
    pub differences: Vec<Difference>,
    pub mapped_files: Vec<String>,
    pub suggested_files: Vec<String>,
}

impl State {
    /// Load from .state.json or create if does not exist
    pub fn get() -> Result<State, Box<dyn Error>> {
        if let Some(dir_path) = Path::new(STATE_PATH).parent() {
            fs::create_dir_all(dir_path)?;
        }
        match File::open(STATE_PATH) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let state: State = serde_json::from_reader(reader)?;
                return Ok(state);
            }
            Err(_) => {
                File::create(STATE_PATH)?;
                let default_state = State {
                    initialized: false,
                    differences: vec![],
                    mapped_files: vec![],
                    suggested_files: vec![],
                };
                default_state.save()?;
                return Ok(default_state);
            }
        }
    }
    pub fn save(&self) -> io::Result<()> {
        let data = serde_json::to_string(&self)?;
        fs::write(STATE_PATH, data)
    }
}
