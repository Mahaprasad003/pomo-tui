pub mod config;
pub mod sessions;
pub mod tags;
pub mod tasks;

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Get the config directory path (~/.config/pomo-tui/)
pub fn config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    let path = base.join("pomo-tui");
    fs::create_dir_all(&path)?;
    Ok(path)
}

/// Get the data directory path (~/.local/share/pomo-tui/)
pub fn data_dir() -> Result<PathBuf> {
    let base = dirs::data_dir().ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?;
    let path = base.join("pomo-tui");
    fs::create_dir_all(&path)?;
    Ok(path)
}
