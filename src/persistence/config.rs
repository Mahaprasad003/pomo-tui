use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Timer durations (in minutes)
    pub work_duration_mins: u64,
    pub short_break_mins: u64,
    pub long_break_mins: u64,
    pub sessions_before_long_break: u8,

    // Mode settings
    pub default_mode: String,
    pub auto_start_breaks: bool,

    // Goals & Streaks
    pub daily_goal_pomodoros: u8,
    pub show_streak: bool,

    // Appearance
    pub breathing_enabled: bool,
    pub hide_hints_after_secs: u8,
    pub theme: String,

    // Focus behavior
    pub focus_mode_on_start: bool,

    // Notifications
    pub notifications_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_duration_mins: 25,
            short_break_mins: 5,
            long_break_mins: 15,
            sessions_before_long_break: 4,
            default_mode: "pomodoro".to_string(),
            auto_start_breaks: false,
            daily_goal_pomodoros: 8,
            show_streak: true,
            breathing_enabled: false,
            hide_hints_after_secs: 3,
            theme: "dark".to_string(),
            focus_mode_on_start: false,
            notifications_enabled: true,
        }
    }
}

impl Config {
    fn file_path() -> Result<PathBuf> {
        Ok(super::config_dir()?.join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::file_path()?;

        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            // Use serde's default for missing fields
            let config: Config = serde_json::from_str(&contents).unwrap_or_default();
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::file_path()?;
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}
