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
}

impl State {
    /// Load from .state.json or create if does not exist
    pub fn get() -> Result<State, Box<dyn Error>> {
        match File::open(".state.json") {
            Ok(file) => {
                let reader = BufReader::new(file);
                let state: State = serde_json::from_reader(reader)?;
                return Ok(state);
            }
            Err(_) => {
                File::create(".state.json")?;
                let default_state = State {
                    initialized: false,
                    repository: String::new(),
                    changed_files: vec![],
                };
                default_state.save()?;
                return Ok(default_state);
            }
        }
    }
    pub fn save(&self) -> io::Result<()> {
        let data = serde_json::to_string(&self)?;
        fs::write(".state.json", data)
    }
}
