//! Gestion des préférences utilisateur (fichier JSON dans le répertoire de configuration).

use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

/// Structure des paramètres sauvegardés.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Temps de départ en secondes (doit être compris entre 4 et 12).
    pub start_seconds: u32,
    /// Durée de pause entre cycles (entre 4 et 12 secondes).
    pub break_seconds: u32,
    /// Mode boucle (redémarrage automatique après la pause).
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
    /// Charge les paramètres depuis le fichier JSON (ou renvoie les valeurs par défaut).
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

    /// Sauvegarde les paramètres actuels dans le fichier JSON.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self)?;
        fs::write(config_path, data)?;
        Ok(())
    }

    /// Chemin du fichier de configuration (dépend de l'OS).
    fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("basket_timer");
        path.push("settings.json");
        path
    }
}