use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr},
    sync::{Arc, Mutex},
};
use thiserror::Error;

use serde::{Deserialize, Serialize};
use toml_edit::{value, Document, TomlError};

const CONFIG_FILENAME: &str = "evergreen.toml";
const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const DEFAULT_PORT: u16 = 8080;

#[derive(Serialize, Deserialize)]
pub struct PlantConfig {
    pub amount_ml: u32,
    pub name: String,
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
    #[error("Error while parsing: {0}")]
    ParseError2(#[from] TomlError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            mutex: Arc::new(Mutex::new(())),
        }
    }

    fn get_raw(&self) -> Result<String, ConfigError> {
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
        Ok(buffer)
    }

    fn get(&self) -> Result<Config, ConfigError> {
        let buffer = self.get_raw()?;
        Ok(toml_edit::de::from_str(buffer.as_str())?)
    }

    fn get_document(&self) -> Result<Document, ConfigError> {
        let buffer = self.get_raw()?;
        Ok(buffer.parse()?)
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

    pub fn put_plant_amount_ml(&self, index: usize, amount_ml: u32) -> Result<(), ConfigError> {
        let mut config = self.get_document()?;
        config["plants"][index]["amount_ml"] = value(amount_ml as i64);
        let mut file = OpenOptions::new().write(true).open(CONFIG_FILENAME)?;
        file.write_all(config.to_string().as_bytes())?;
        Ok(())
    }

    pub fn put_plant_name(&self, index: usize, name: String) -> Result<(), ConfigError> {
        let mut config = self.get_document()?;
        config["plants"][index]["name"] = value(name);
        let mut file = OpenOptions::new().write(true).open(CONFIG_FILENAME)?;
        file.write_all(config.to_string().as_bytes())?;
        Ok(())
    }
}
