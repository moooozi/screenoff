use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub secondary_monitors: Vec<String>,
    pub saved_modes: HashMap<String, (u32, u32, i32, i32)>, // width, height, x, y
}

fn get_config_path() -> PathBuf {
    let app_id = env!("APP_ID");
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let config_dir = PathBuf::from(appdata).join(app_id);

    // Create directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).ok();
    }

    config_dir.join("config.json")
}

pub fn load_config() -> Config {
    let path = get_config_path();
    if path.exists() {
        let data = fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save_config(config: &Config) {
    let path = get_config_path();
    let data = serde_json::to_string(config).unwrap();
    fs::write(path, data).unwrap();
}
