use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

const STATE_FILENAME: &str = "state.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub last_planned_watering: Vec<chrono::NaiveDate>,
    pub last_seen: chrono::NaiveDateTime,
    pub last_accu_percentage: f32,
}

#[derive(Debug, Clone)]
pub struct Stateholder {
    mutex: Arc<Mutex<()>>,
}

#[derive(Debug)]
pub enum StateError {
    Io(std::io::Error),
    Parse(serde_json::Error),
}

impl From<std::io::Error> for StateError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for StateError {
    fn from(value: serde_json::Error) -> Self {
        Self::Parse(value)
    }
}

impl Stateholder {
    pub fn new() -> Self {
        Self {
            mutex: Arc::new(Mutex::new(())),
        }
    }

    pub fn get(&self) -> Result<State, StateError> {
        let _guard = self.mutex.lock();
        let mut file = File::open(STATE_FILENAME)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        Ok(serde_json::from_str(buffer.as_str())?)
    }

    pub fn set(&self, state: State) -> Result<(), StateError> {
        let _guard = self.mutex.lock();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(STATE_FILENAME)?;
        let mut buf = serde_json::to_string(&state)?;
        file.write(buf.as_bytes())?;
        Ok(())
    }
}
