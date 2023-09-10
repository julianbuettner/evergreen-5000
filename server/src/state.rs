use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{ErrorKind, Read, Write},
    sync::{Arc, Mutex},
};
use thiserror::Error;

const STATE_FILENAME: &str = "state.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonState {
    pub last_planned_watering: chrono::NaiveDate,
    pub last_seen: chrono::NaiveDateTime,
    pub last_accu_percentage: f32,
}

#[derive(Debug, Clone)]
pub struct JsonStateManager {
    mutex: Arc<Mutex<()>>,
}

#[derive(Error, Debug)]
pub enum StateError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parsing error: {0}")]
    Parse(#[from] serde_json::Error),
}

impl JsonStateManager {
    pub fn new() -> Self {
        Self {
            mutex: Arc::new(Mutex::new(())),
        }
    }

    pub fn get(&self) -> Result<JsonState, StateError> {
        let _guard = self.mutex.lock();
        let mut file = File::open(STATE_FILENAME)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        Ok(serde_json::from_str(buffer.as_str())?)
    }

    pub fn set(&self, state: JsonState) -> Result<(), StateError> {
        let _guard = self.mutex.lock();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(STATE_FILENAME)?;
        let buf = serde_json::to_string(&state)?;
        file.write(buf.as_bytes())?;
        Ok(())
    }

    pub fn ensure_state(&self) -> Result<(), StateError> {
        let state = match self.get() {
            Ok(s) => Ok(Some(s)),
            Err(StateError::Io(e)) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }?;

        if state.is_none() {
            let default_state = JsonState {
                last_seen: NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                last_accu_percentage: 0.0,
                last_planned_watering: NaiveDate::from_yo_opt(1970, 1).unwrap(),
            };
            self.set(default_state)?;
        }
        Ok(())
    }
}
