use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub secondary_monitors: Vec<String>,
    pub saved_modes: HashMap<String, (u32, u32, i32, i32)>, // width, height, x, y
}

pub fn load_config() -> Config {
    let path = Path::new("screenoff_config.json");
    if path.exists() {
        let data = fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save_config(config: &Config) {
    let data = serde_json::to_string(config).unwrap();
    fs::write("screenoff_config.json", data).unwrap();
}
