use anyhow::Result;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A learned tag with usage metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    pub name: String,
    pub last_used: NaiveDate,
    pub count: u32,
}

impl TagInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            last_used: Utc::now().date_naive(),
            count: 1,
        }
    }
}

/// Tag storage with learning
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TagStore {
    pub tags: Vec<TagInfo>,
}

impl TagStore {
    fn file_path() -> Result<PathBuf> {
        Ok(super::data_dir()?.join("tags.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::file_path()?;

        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let mut store: TagStore = serde_json::from_str(&contents).unwrap_or_default();
            store.cleanup_old_tags();
            Ok(store)
        } else {
            let store = TagStore::default();
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

    /// Remove tags not used in 30 days
    fn cleanup_old_tags(&mut self) {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(30);
        self.tags.retain(|t| t.last_used >= cutoff);
    }

    /// Record usage of tags (learn new ones, update existing)
    pub fn record_usage(&mut self, tag_names: &[String]) {
        let today = Utc::now().date_naive();

        for name in tag_names {
            let name_lower = name.to_lowercase();
            if let Some(tag) = self.tags.iter_mut().find(|t| t.name.to_lowercase() == name_lower) {
                tag.count += 1;
                tag.last_used = today;
            } else {
                self.tags.push(TagInfo::new(name.clone()));
            }
        }

        // Sort by count (most used first)
        self.tags.sort_by(|a, b| b.count.cmp(&a.count));

        let _ = self.save();
    }

    /// Get recent/frequent tags (top N)
    pub fn recent_tags(&self, count: usize) -> Vec<&str> {
        self.tags.iter().take(count).map(|t| t.name.as_str()).collect()
    }

    /// Find matching tags for autocomplete (fuzzy prefix match)
    pub fn suggest(&self, partial: &str) -> Option<&str> {
        if partial.is_empty() {
            return None;
        }

        let partial_lower = partial.to_lowercase();

        // Exact prefix match first (sorted by frequency)
        for tag in &self.tags {
            if tag.name.to_lowercase().starts_with(&partial_lower) {
                return Some(&tag.name);
            }
        }

        // Fuzzy: contains match
        for tag in &self.tags {
            if tag.name.to_lowercase().contains(&partial_lower) {
                return Some(&tag.name);
            }
        }

        None
    }


}
