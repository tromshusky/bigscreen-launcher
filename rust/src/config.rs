use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_steam_mode")]
    pub steam_launch_mode: String,
    #[serde(default = "default_grid_rows")]
    pub grid_rows: u32,
    #[serde(default = "default_grid_columns")]
    pub grid_columns: u32,
    #[serde(default = "default_scrolling_mode")]
    pub scrolling_mode: String,
}

fn default_steam_mode() -> String { "bigpicture".to_string() }
fn default_grid_rows() -> u32 { 1 }
fn default_grid_columns() -> u32 { 4 }
fn default_scrolling_mode() -> String { "continuous".to_string() }

impl Default for Config {
    fn default() -> Self {
        Config {
            steam_launch_mode: default_steam_mode(),
            grid_rows: default_grid_rows(),
            grid_columns: default_grid_columns(),
            scrolling_mode: default_scrolling_mode(),
        }
    }
}

impl Config {
    fn config_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config").join("bigscreen-launcher")
    }

    fn config_file() -> PathBuf {
        Self::config_dir().join("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::config_file();
        if let Ok(contents) = fs::read_to_string(&path) {
            match serde_json::from_str::<Config>(&contents) {
                Ok(cfg) => return cfg,
                Err(e) => eprintln!("Warning: could not parse config file: {e}"),
            }
        }
        let cfg = Config::default();
        cfg.save();
        cfg
    }

    pub fn save(&self) {
        let dir = Self::config_dir();
        if let Err(e) = fs::create_dir_all(&dir) {
            eprintln!("Warning: could not create config dir: {e}");
            return;
        }
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = fs::write(Self::config_file(), json) {
                    eprintln!("Warning: could not save config file: {e}");
                }
            }
            Err(e) => eprintln!("Warning: could not serialize config: {e}"),
        }
    }
}