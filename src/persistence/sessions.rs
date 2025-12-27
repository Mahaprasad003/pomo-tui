use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// A completed Pomodoro session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub session_type: String,
    pub duration_secs: u64,
    pub completed: bool,
    pub task_name: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

impl Session {
    pub fn new(session_type: &str, duration_secs: u64, task_name: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            session_type: session_type.to_string(),
            duration_secs,
            completed: true,
            task_name,
            note: None,
        }
    }

    pub fn with_note(session_type: &str, duration_secs: u64, task_name: Option<String>, note: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            session_type: session_type.to_string(),
            duration_secs,
            completed: true,
            task_name,
            note,
        }
    }
}

/// Session history storage with streak tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHistory {
    pub sessions: Vec<Session>,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub last_session_date: Option<NaiveDate>,
}

impl Default for SessionHistory {
    fn default() -> Self {
        Self {
            sessions: Vec::new(),
            current_streak: 0,
            longest_streak: 0,
            last_session_date: None,
        }
    }
}

impl SessionHistory {
    fn file_path() -> Result<PathBuf> {
        Ok(super::data_dir()?.join("sessions.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::file_path()?;

        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            let mut history: SessionHistory = serde_json::from_str(&contents).unwrap_or_default();
            history.recalculate_streak();
            Ok(history)
        } else {
            let history = SessionHistory::default();
            history.save()?;
            Ok(history)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::file_path()?;
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    /// Add a new session and update streak
    pub fn add(&mut self, session: Session) {
        let session_date = session.timestamp.date_naive();
        
        // Only update streak for work sessions
        if session.session_type == "work" {
            self.update_streak(session_date);
        }
        
        self.sessions.push(session);
    }

    /// Update streak based on new session date
    fn update_streak(&mut self, session_date: NaiveDate) {
        let _today = Utc::now().date_naive();
        
        match self.last_session_date {
            Some(last_date) => {
                let days_diff = (session_date - last_date).num_days();
                
                if days_diff == 0 {
                    // Same day, streak continues
                } else if days_diff == 1 {
                    // Consecutive day, increment streak
                    self.current_streak += 1;
                    if self.current_streak > self.longest_streak {
                        self.longest_streak = self.current_streak;
                    }
                } else {
                    // Streak broken, reset
                    self.current_streak = 1;
                }
            }
            None => {
                // First session ever
                self.current_streak = 1;
                self.longest_streak = 1;
            }
        }
        
        self.last_session_date = Some(session_date);
    }

    /// Recalculate streak on load (in case app wasn't used yesterday)
    fn recalculate_streak(&mut self) {
        let today = Utc::now().date_naive();
        
        if let Some(last_date) = self.last_session_date {
            let days_diff = (today - last_date).num_days();
            
            if days_diff > 1 {
                // Streak broken - missed more than one day
                self.current_streak = 0;
            }
            // If days_diff is 0 or 1, streak is still valid
        }
    }

    /// Get today's completed work sessions count
    pub fn today_pomodoro_count(&self) -> usize {
        let today = Utc::now().date_naive();
        self.sessions
            .iter()
            .filter(|s| s.timestamp.date_naive() == today && s.session_type == "work")
            .count()
    }

    pub fn today_focus_secs(&self) -> u64 {
        let today = Utc::now().date_naive();
        self.sessions
            .iter()
            .filter(|s| s.timestamp.date_naive() == today && s.session_type == "work")
            .map(|s| s.duration_secs)
            .sum()
    }

    pub fn today_session_count(&self) -> usize {
        self.today_pomodoro_count()
    }

    pub fn week_focus_secs(&self) -> u64 {
        use chrono::Datelike;
        let now = Utc::now();
        let week_start = now.date_naive()
            - chrono::Duration::days(now.weekday().num_days_from_monday() as i64);

        self.sessions
            .iter()
            .filter(|s| s.timestamp.date_naive() >= week_start && s.session_type == "work")
            .map(|s| s.duration_secs)
            .sum()
    }

    pub fn total_focus_secs(&self) -> u64 {
        self.sessions
            .iter()
            .filter(|s| s.session_type == "work")
            .map(|s| s.duration_secs)
            .sum()
    }

    pub fn last_7_days_focus(&self) -> Vec<(String, u64)> {
        use chrono::Datelike;
        let today = Utc::now().date_naive();
        let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

        (0..7)
            .rev()
            .map(|i| {
                let date = today - chrono::Duration::days(i as i64);
                let day_name = days[date.weekday().num_days_from_monday() as usize];
                let secs: u64 = self
                    .sessions
                    .iter()
                    .filter(|s| s.timestamp.date_naive() == date && s.session_type == "work")
                    .map(|s| s.duration_secs)
                    .sum();
                (day_name.to_string(), secs)
            })
            .collect()
    }

    pub fn recent_sessions(&self, count: usize) -> Vec<&Session> {
        self.sessions.iter().rev().take(count).collect()
    }
}
