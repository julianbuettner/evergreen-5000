use std::{
    fs::File,
    io::Read,
    net::{IpAddr, Ipv4Addr},
    sync::{Arc, Mutex},
};
use thiserror::Error;

use serde::{Deserialize, Serialize};

const CONFIG_FILENAME: &str = "evergreen.toml";
const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const DEFAULT_PORT: u16 = 8080;

#[derive(Serialize, Deserialize)]
pub struct PlantConfig {
    amount_ml: u32,
    name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    host: Option<IpAddr>,
    port: Option<u16>,
    plants: Vec<PlantConfig>,
}

#[derive(Clone)]
pub struct ConfigManager {
    mutex: Arc<Mutex<()>>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file {} not found in working directory", CONFIG_FILENAME)]
    NotFound,
    #[error("Error while parsing: {0}")]
    ParseError(#[from] toml_edit::de::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            mutex: Arc::new(Mutex::new(())),
        }
    }

    fn get(&self) -> Result<Config, ConfigError> {
        let _guard = self.mutex.lock();
        let mut file = File::open(CONFIG_FILENAME)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ConfigError::NotFound
            } else {
                e.into()
            }
        })?;
        Ok(toml_edit::de::from_str(buffer.as_str())?)
    }

    pub fn get_host(&self) -> Result<IpAddr, ConfigError> {
        Ok(self.get()?.host.unwrap_or(DEFAULT_HOST))
    }

    pub fn get_port(&self) -> Result<u16, ConfigError> {
        Ok(self.get()?.port.unwrap_or(DEFAULT_PORT))
    }

    pub fn get_plant_config(&self) -> Result<Vec<PlantConfig>, ConfigError> {
        Ok(self.get()?.plants)
    }
}
