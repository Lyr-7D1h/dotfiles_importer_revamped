use crate::STATE_PATH;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io;
use std::{fs::File, io::BufReader};

#[derive(Serialize, Deserialize)]
pub struct State {
    pub initialized: bool,
    pub repository: String,
    pub changed_files: Vec<String>,
    pub mapped_files: Vec<String>,
    pub suggested_files: Vec<String>,
}

impl State {
    /// Load from .state.json or create if does not exist
    pub fn get() -> Result<State, Box<dyn Error>> {
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
                    repository: String::new(),
                    changed_files: vec![],
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