use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// A task item (serializable version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskData {
    pub id: Uuid,
    pub name: String,
    pub completed: bool,
    pub pomodoros_spent: u32,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
}



/// Task list storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskStore {
    pub tasks: Vec<TaskData>,
}

impl TaskStore {
    fn file_path() -> Result<PathBuf> {
        Ok(super::data_dir()?.join("tasks.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::file_path()?;

        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let store: TaskStore = serde_json::from_str(&contents).unwrap_or_default();
            Ok(store)
        } else {
            let store = TaskStore::default();
            store.save()?;
            Ok(store)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::file_path()?;
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}

/// Parse task input for tags (e.g., "Buy milk #shopping #urgent")
/// Returns (clean_name, tags)
pub fn parse_task_input(input: &str) -> (String, Vec<String>) {
    let mut tags = Vec::new();
    let mut name_parts = Vec::new();

    for word in input.split_whitespace() {
        if word.starts_with('#') && word.len() > 1 {
            tags.push(word[1..].to_string());
        } else {
            name_parts.push(word);
        }
    }

    (name_parts.join(" "), tags)
}
