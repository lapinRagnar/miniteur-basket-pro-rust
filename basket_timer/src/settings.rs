use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub start_seconds: u32,
    pub break_seconds: u32,
    pub loop_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            start_seconds: 12,
            break_seconds: 5,
            loop_enabled: false,
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        if config_path.exists() {
            if let Ok(data) = fs::read_to_string(&config_path) {
                if let Ok(settings) = serde_json::from_str(&data) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(config_path, data)?;
        Ok(())
    }

    fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("basket_timer");
        path.push("settings.json");
        path
    }
}