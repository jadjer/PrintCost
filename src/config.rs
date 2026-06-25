use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub const CONFIG_FILE: &str = "print_config.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub price_per_hour: f64,
    pub materials: HashMap<String, f64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            price_per_hour: 5.0,
            materials: [("pla".to_string(), 50.0), ("abs".to_string(), 50.0), ("petg".to_string(), 50.0)]
                .into_iter()
                .collect(),
        }
    }
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    if !Path::new(CONFIG_FILE).exists() {
        return Err("File does not exist".into());
    }
    
    let mut file = File::open(CONFIG_FILE)?;
    let mut contents = String::new();
    
    file.read_to_string(&mut contents)?;
    
    let config: Config = serde_json::from_str(&contents)?;
    
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(CONFIG_FILE)?;
    let json = serde_json::to_string_pretty(config)?;
    
    file.write_all(json.as_bytes())?;
    
    Ok(())
}
