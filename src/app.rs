use crate::persistence::{
    config::Config,
    sessions::{Session, SessionHistory},
    tags::TagStore,
    tasks::{parse_task_input, TaskStore},
};
use chrono::Timelike;
use crossterm::event::KeyCode;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Timer mode - Pomodoro with auto-cycling or flexible Timer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerMode {
    Pomodoro,
    Timer(u64),
}

/// Current state in the Pomodoro cycle
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Work,
    ShortBreak,
    LongBreak,
}

impl TimerState {
    pub fn display_name(&self) -> &'static str {
        match self {
            TimerState::Work => "Work",
            TimerState::ShortBreak => "Short Break",
            TimerState::LongBreak => "Long Break",
        }
    }

    pub fn color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            TimerState::Work => Color::Cyan,
            TimerState::ShortBreak => Color::Green,
            TimerState::LongBreak => Color::Magenta,
        }
    }

    pub fn session_type(&self) -> &'static str {
        match self {
            TimerState::Work => "work",
            TimerState::ShortBreak => "short_break",
            TimerState::LongBreak => "long_break",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Tasks,
    Timer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentView {
    Timer,
    Dashboard,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    AddingTask,
    QuickCapture,
    SessionNote,
    ConfirmReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsCategory {
    Timer,
    Goals,
    Appearance,
    Behavior,
    Notifications,
    Danger,
}

impl SettingsCategory {


    pub fn name(&self) -> &'static str {
        match self {
            Self::Timer => "TIMER",
            Self::Goals => "GOALS & STREAKS",
            Self::Appearance => "APPEARANCE",
            Self::Behavior => "BEHAVIOR",
            Self::Notifications => "NOTIFICATIONS",
            Self::Danger => "⚠️ DANGER ZONE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    // Timer
    WorkDuration,
    ShortBreak,
    LongBreak,
    SessionsBeforeLong,
    // Goals
    DailyGoal,
    ShowStreak,
    // Appearance
    BreathingAnimation,
    HideHintsAfter,
    // Behavior
    AutoStartBreaks,
    FocusModeOnStart,
    // Notifications
    NotificationsEnabled,
    // Danger
    ResetData,
}

impl SettingsField {
    pub fn category(&self) -> SettingsCategory {
        match self {
            Self::WorkDuration | Self::ShortBreak | Self::LongBreak | Self::SessionsBeforeLong => {
                SettingsCategory::Timer
            }
            Self::DailyGoal | Self::ShowStreak => SettingsCategory::Goals,
            Self::BreathingAnimation | Self::HideHintsAfter => SettingsCategory::Appearance,
            Self::AutoStartBreaks | Self::FocusModeOnStart => SettingsCategory::Behavior,
            Self::NotificationsEnabled => SettingsCategory::Notifications,
            Self::ResetData => SettingsCategory::Danger,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::WorkDuration,
            Self::ShortBreak,
            Self::LongBreak,
            Self::SessionsBeforeLong,
            Self::DailyGoal,
            Self::ShowStreak,
            Self::BreathingAnimation,
            Self::HideHintsAfter,
            Self::AutoStartBreaks,
            Self::FocusModeOnStart,
            Self::NotificationsEnabled,
            Self::ResetData,
        ]
    }

    pub fn next(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|f| f == self).unwrap_or(0);
        all[(idx + 1) % all.len()]
    }

    pub fn prev(&self) -> Self {
        let all = Self::all();
        let idx = all.iter().position(|f| f == self).unwrap_or(0);
        all[(idx + all.len() - 1) % all.len()]
    }
}

/// A task item
#[derive(Debug, Clone)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub completed: bool,
    pub pomodoros_spent: u32,
    pub tags: Vec<String>,
}

impl Task {


    pub fn with_tags(name: String, tags: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            completed: false,
            pomodoros_spent: 0,
            tags,
        }
    }
}

/// Application state
pub struct App {
    // Timer state
    pub timer_mode: TimerMode,
    pub timer_state: TimerState,
    pub remaining_time: Duration,
    pub is_paused: bool,

    // Drift correction
    start_instant: Option<Instant>,
    start_remaining: Duration,

    // Pomodoro cycle tracking
    pub session_count: u8,
    pub sessions_before_long: u8,

    // Task management
    pub tasks: Vec<Task>,
    pub selected_task_index: usize,

    // Navigation
    pub active_pane: ActivePane,
    pub current_view: CurrentView,
    pub input_mode: InputMode,
    pub input_buffer: String,

    // Focus mode
    pub focus_mode: bool,

    // Breathing animation
    pub breathing_phase: u8, // 0-100 for animation cycle

    // Hint fading
    pub hints_visible: bool,
    pub hint_fade_counter: u8,

    // Settings
    pub selected_setting: SettingsField,
    pub config: Config,

    // Session history
    pub session_history: SessionHistory,

    // Tag autocomplete
    pub tag_store: TagStore,
    pub tag_suggestion: Option<String>,

    // Celebration state
    pub show_celebration: bool,
    pub celebration_message: String,
    pub celebration_timer: u8,

    // Session note (pending session waiting for note)
    pub pending_session: Option<(String, u64, Option<String>)>, // (type, duration, task_name)

    // Control flags
    pub should_quit: bool,
    pub show_help: bool,
    needs_save: bool,
}

impl App {
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();
        let session_history = SessionHistory::load().unwrap_or_default();
        let tag_store = TagStore::load().unwrap_or_default();

        let task_store = TaskStore::load().unwrap_or_default();
        let tasks: Vec<Task> = task_store
            .tasks
            .into_iter()
            .map(|t| Task {
                id: t.id,
                name: t.name,
                completed: t.completed,
                pomodoros_spent: t.pomodoros_spent,
                tags: t.tags,
            })
            .collect();

        let sessions_before_long = config.sessions_before_long_break;
        let work_duration = Duration::from_secs(config.work_duration_mins * 60);

        Self {
            timer_mode: TimerMode::Pomodoro,
            timer_state: TimerState::Work,
            remaining_time: work_duration,
            is_paused: true,

            start_instant: None,
            start_remaining: work_duration,

            session_count: 0,
            sessions_before_long,

            tasks,
            selected_task_index: 0,

            active_pane: ActivePane::Tasks,
            current_view: CurrentView::Timer,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),

            focus_mode: false,
            breathing_phase: 0,
            hints_visible: true,
            hint_fade_counter: 0,

            selected_setting: SettingsField::WorkDuration,
            config,

            session_history,
            tag_store,
            tag_suggestion: None,

            show_celebration: false,
            celebration_message: String::new(),
            celebration_timer: 0,
            pending_session: None,

            should_quit: false,
            show_help: false,
            needs_save: false,
        }
    }

    fn duration_for_state(&self, state: TimerState) -> Duration {
        match state {
            TimerState::Work => Duration::from_secs(self.config.work_duration_mins * 60),
            TimerState::ShortBreak => Duration::from_secs(self.config.short_break_mins * 60),
            TimerState::LongBreak => Duration::from_secs(self.config.long_break_mins * 60),
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        // Reset hint fade counter on any key
        self.hints_visible = true;
        self.hint_fade_counter = 0;

        // Quick capture works anywhere (except when already in input mode)
        if key == KeyCode::Char('/') && self.input_mode == InputMode::Normal && !self.show_help {
            self.input_mode = InputMode::QuickCapture;
            self.input_buffer.clear();
            return;
        }

        match self.current_view {
            CurrentView::Timer => self.handle_timer_view_key(key),
            CurrentView::Dashboard => self.handle_dashboard_key(key),
            CurrentView::Settings => self.handle_settings_key(key),
        }
    }

    fn handle_timer_view_key(&mut self, key: KeyCode) {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_key(key),
            InputMode::AddingTask | InputMode::QuickCapture => self.handle_input_key(key),
            InputMode::SessionNote => self.handle_session_note_key(key),
            InputMode::ConfirmReset => {
                self.input_mode = InputMode::Normal;
            }
        }
    }

    fn handle_normal_key(&mut self, key: KeyCode) {
        if self.show_help {
            self.show_help = false;
            return;
        }

        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.save_all();
                self.should_quit = true;
            }

            KeyCode::Char('?') => {
                self.show_help = true;
            }

            // Focus mode toggle
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.focus_mode = !self.focus_mode;
            }

            // View switching
            KeyCode::Char('1') => {
                self.current_view = CurrentView::Timer;
                self.focus_mode = false;
            }
            KeyCode::Char('2') => {
                self.current_view = CurrentView::Dashboard;
                self.focus_mode = false;
            }
            KeyCode::Char('3') => {
                self.current_view = CurrentView::Settings;
                self.focus_mode = false;
            }

            KeyCode::Char(' ') => self.toggle_pause(),
            KeyCode::Char('r') | KeyCode::Char('R') => self.reset_timer(),
            KeyCode::Char('n') | KeyCode::Char('N') => self.skip_to_next(),
            KeyCode::Char('m') | KeyCode::Char('M') => self.toggle_mode(),

            KeyCode::Tab => {
                if !self.focus_mode {
                    self.active_pane = match self.active_pane {
                        ActivePane::Tasks => ActivePane::Timer,
                        ActivePane::Timer => ActivePane::Tasks,
                    };
                }
            }

            KeyCode::Char('k') | KeyCode::Up => {
                if self.active_pane == ActivePane::Tasks && !self.tasks.is_empty() {
                    if self.selected_task_index > 0 {
                        self.selected_task_index -= 1;
                    } else {
                        self.selected_task_index = self.tasks.len() - 1;
                    }
                }
            }

            KeyCode::Char('j') | KeyCode::Down => {
                if self.active_pane == ActivePane::Tasks && !self.tasks.is_empty() {
                    if self.selected_task_index < self.tasks.len() - 1 {
                        self.selected_task_index += 1;
                    } else {
                        self.selected_task_index = 0;
                    }
                }
            }

            KeyCode::Char('a') | KeyCode::Char('A') => {
                if self.active_pane == ActivePane::Tasks || self.focus_mode {
                    self.input_mode = InputMode::AddingTask;
                    self.input_buffer.clear();
                }
            }

            KeyCode::Char('d') | KeyCode::Char('D') => {
                if (self.active_pane == ActivePane::Tasks || self.focus_mode) && !self.tasks.is_empty() {
                    self.tasks.remove(self.selected_task_index);
                    // Clamp index to valid range
                    if self.tasks.is_empty() {
                        self.selected_task_index = 0;
                    } else if self.selected_task_index >= self.tasks.len() {
                        self.selected_task_index = self.tasks.len() - 1;
                    }
                    self.needs_save = true;
                }
            }

            KeyCode::Char('c') | KeyCode::Char('C') => {
                if self.active_pane == ActivePane::Tasks || self.focus_mode {
                    // Clear all completed tasks
                    self.tasks.retain(|t| !t.completed);
                    // Clamp selected index
                    if self.tasks.is_empty() {
                        self.selected_task_index = 0;
                    } else if self.selected_task_index >= self.tasks.len() {
                        self.selected_task_index = self.tasks.len() - 1;
                    }
                    self.needs_save = true;
                }
            }

            KeyCode::Enter => {
                if (self.active_pane == ActivePane::Tasks || self.focus_mode) && !self.tasks.is_empty() {
                    self.tasks[self.selected_task_index].completed =
                        !self.tasks[self.selected_task_index].completed;
                    self.needs_save = true;
                }
            }

            KeyCode::Esc => {
                if self.focus_mode {
                    self.focus_mode = false;
                }
            }

            _ => {}
        }
    }

    fn handle_input_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                if !self.input_buffer.is_empty() {
                    let (name, tags) = parse_task_input(&self.input_buffer);
                    // Only create task if name is not empty (not just tags)
                    if !name.trim().is_empty() {
                        // Record tag usage
                        if !tags.is_empty() {
                            self.tag_store.record_usage(&tags);
                        }
                        let task = Task::with_tags(name, tags);
                        self.tasks.push(task);
                        self.selected_task_index = self.tasks.len() - 1;
                        self.needs_save = true;
                    }
                }
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
                self.tag_suggestion = None;
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
                self.tag_suggestion = None;
            }
            KeyCode::Tab => {
                // Accept tag suggestion
                if let Some(suggestion) = self.tag_suggestion.take() {
                    self.accept_tag_suggestion(&suggestion);
                }
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
                self.update_tag_suggestion();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
                self.update_tag_suggestion();
            }
            _ => {}
        }
    }

    /// Update tag suggestion based on current input
    fn update_tag_suggestion(&mut self) {
        // Find the last incomplete tag in input
        if let Some(last_hash_pos) = self.input_buffer.rfind('#') {
            let partial_tag = &self.input_buffer[last_hash_pos + 1..];
            // Only suggest if we're still typing (no space after #)
            if !partial_tag.is_empty() && !partial_tag.contains(' ') {
                self.tag_suggestion = self.tag_store.suggest(partial_tag).map(|s| s.to_string());
            } else {
                self.tag_suggestion = None;
            }
        } else {
            self.tag_suggestion = None;
        }
    }

    /// Accept the tag suggestion and replace partial tag
    fn accept_tag_suggestion(&mut self, suggestion: &str) {
        if let Some(last_hash_pos) = self.input_buffer.rfind('#') {
            // Replace everything after # with the suggestion
            self.input_buffer.truncate(last_hash_pos + 1);
            self.input_buffer.push_str(suggestion);
            self.input_buffer.push(' ');
        }
    }

    /// Get recent tags for display
    pub fn recent_tags(&self) -> Vec<&str> {
        self.tag_store.recent_tags(5)
    }

    /// Handle session note input
    fn handle_session_note_key(&mut self, key: KeyCode) {
        match key {
            // Enter -> Save and PAUSE (Wait for user)
            KeyCode::Enter => {
                let note = if self.input_buffer.trim().is_empty() {
                    None
                } else {
                    Some(self.input_buffer.clone())
                };
                self.complete_pending_session(note);
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
                // Ensure we stay paused
                self.is_paused = true;
            }
            // Space -> Save and CONTINUE (Auto-start next timer)
            KeyCode::Char(' ') => {
                // If buffer is empty, it's a command to continue
                // If buffer has text, appending space is annoying if we meant to continue,
                // but "Steve Jobs" mode implies intuitive flow.
                // Let's say: if input is empty, Space continues.
                // If input has text? We might need space for typing.
                // Dilemma: How to type "Good Job"?
                // Solution: We can't use Space for flow IF we are typing.
                // EXCEPT: The requirement was "Spacebar to skip/continue".
                // Let's limit this:
                // If buffer is empty -> Space skips note and auto-starts.
                // If buffer is NOT empty -> Space adds a space character.
                // To auto-start with note, maybe we use a specific combo?
                // OR: The user said "One Button Flow... Spacebar to skip/continue".
                // I will make Space append character normally.
                // But I'll add a check: if input is empty, maybe we treat it as skip?
                // Actually, if I just finished a session, and I hit space to pause/play usually...
                // Let's strictly follow "Spacebar dominance".
                // If I want to type a note, I start typing characters.
                // If I hit Space immediately (buffer empty) -> Start next.
                // If I type "Refactored" then Space -> "Refactored ".
                // How to submit then? Enter.
                // But the requirement says "Spacebar to skip/continue".
                // I'll implement: Empty Buffer + Space = Skip & Start. Non-empty + Space = Type space.
                
                if self.input_buffer.is_empty() {
                    self.complete_pending_session(None);
                    self.input_mode = InputMode::Normal;
                    self.input_buffer.clear();
                    // Auto-start next state
                    self.toggle_pause(); 
                } else {
                     if self.input_buffer.len() < 60 {
                        self.input_buffer.push(' ');
                    }
                }
            }
            KeyCode::Esc => {
                // Skip note, save session without it
                self.complete_pending_session(None);
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
                self.is_paused = true;
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                if self.input_buffer.len() < 60 {
                    self.input_buffer.push(c);
                }
            }
            _ => {}
        }
    }

    /// Get time-based greeting
    pub fn greeting(&self) -> &'static str {
        use chrono::Local;
        let hour = Local::now().hour();
        match hour {
            5..=11 => "Good morning",
            12..=16 => "Good afternoon",
            17..=20 => "Good evening",
            _ => "Good night",
        }
    }

    /// Get estimated end time
    pub fn estimated_end_time(&self) -> String {
        use chrono::Local;
        let now = Local::now();
        let end = now + chrono::Duration::seconds(self.remaining_time.as_secs() as i64);
        end.format("%H:%M").to_string()
    }

    /// Check if it's late night (after 11 PM)
    pub fn is_late_night(&self) -> bool {
        use chrono::Local;
        Local::now().hour() >= 23
    }

    fn handle_dashboard_key(&mut self, key: KeyCode) {
        // Quick capture check is done in handle_key

        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.save_all();
                self.should_quit = true;
            }
            KeyCode::Char('1') => self.current_view = CurrentView::Timer,
            KeyCode::Char('2') => self.current_view = CurrentView::Dashboard,
            KeyCode::Char('3') => self.current_view = CurrentView::Settings,
            KeyCode::Esc => self.current_view = CurrentView::Timer,
            _ => {}
        }
    }

    fn handle_settings_key(&mut self, key: KeyCode) {
        // Handle confirm reset mode
        if self.input_mode == InputMode::ConfirmReset {
            self.handle_confirm_reset_key(key);
            return;
        }

        match key {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.save_all();
                self.should_quit = true;
            }
            KeyCode::Char('1') => self.current_view = CurrentView::Timer,
            KeyCode::Char('2') => self.current_view = CurrentView::Dashboard,
            KeyCode::Char('3') => self.current_view = CurrentView::Settings,
            KeyCode::Esc => self.current_view = CurrentView::Timer,

            KeyCode::Char('j') | KeyCode::Down => {
                self.selected_setting = self.selected_setting.next();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_setting = self.selected_setting.prev();
            }

            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                self.adjust_setting(1);
            }
            KeyCode::Char('h') | KeyCode::Left => {
                self.adjust_setting(-1);
            }

            _ => {}
        }
    }

    fn adjust_setting(&mut self, delta: i64) {
        match self.selected_setting {
            SettingsField::WorkDuration => {
                let new_val = (self.config.work_duration_mins as i64 + delta).clamp(1, 120);
                self.config.work_duration_mins = new_val as u64;
            }
            SettingsField::ShortBreak => {
                let new_val = (self.config.short_break_mins as i64 + delta).clamp(1, 60);
                self.config.short_break_mins = new_val as u64;
            }
            SettingsField::LongBreak => {
                let new_val = (self.config.long_break_mins as i64 + delta).clamp(1, 60);
                self.config.long_break_mins = new_val as u64;
            }
            SettingsField::SessionsBeforeLong => {
                let new_val = (self.config.sessions_before_long_break as i64 + delta).clamp(1, 10);
                self.config.sessions_before_long_break = new_val as u8;
                self.sessions_before_long = new_val as u8;
            }
            SettingsField::DailyGoal => {
                let new_val = (self.config.daily_goal_pomodoros as i64 + delta).clamp(1, 20);
                self.config.daily_goal_pomodoros = new_val as u8;
            }
            SettingsField::ShowStreak => {
                self.config.show_streak = !self.config.show_streak;
            }
            SettingsField::BreathingAnimation => {
                self.config.breathing_enabled = !self.config.breathing_enabled;
            }
            SettingsField::HideHintsAfter => {
                let new_val = (self.config.hide_hints_after_secs as i64 + delta).clamp(0, 10);
                self.config.hide_hints_after_secs = new_val as u8;
            }
            SettingsField::AutoStartBreaks => {
                self.config.auto_start_breaks = !self.config.auto_start_breaks;
            }
            SettingsField::FocusModeOnStart => {
                self.config.focus_mode_on_start = !self.config.focus_mode_on_start;
            }
            SettingsField::NotificationsEnabled => {
                self.config.notifications_enabled = !self.config.notifications_enabled;
            }
            SettingsField::ResetData => {
                // Start confirmation flow
                self.input_mode = InputMode::ConfirmReset;
                self.input_buffer.clear();
                return; // Don't save config
            }
        }
        let _ = self.config.save();
    }

    /// Handle confirm reset input (type DELETE to confirm)
    fn handle_confirm_reset_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                if self.input_buffer == "DELETE" {
                    self.reset_all_data();
                }
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                if self.input_buffer.len() < 10 {
                    self.input_buffer.push(c.to_ascii_uppercase());
                }
            }
            _ => {}
        }
    }

    /// Reset all user data to defaults
    fn reset_all_data(&mut self) {
        // Clear sessions
        self.session_history = SessionHistory::default();
        let _ = self.session_history.save();

        // Clear tasks
        self.tasks.clear();
        self.selected_task_index = 0;
        self.needs_save = true;

        // Clear tags
        self.tag_store = TagStore::default();
        let _ = self.tag_store.save();

        // Reset timer state
        self.session_count = 0;
        self.remaining_time = self.duration_for_state(TimerState::Work);
        self.is_paused = true;
    }

    fn toggle_pause(&mut self) {
        if self.is_paused {
            self.start_instant = Some(Instant::now());
            self.start_remaining = self.remaining_time;
            self.is_paused = false;
            
            // Auto-enter focus mode if configured
            if self.config.focus_mode_on_start {
                self.focus_mode = true;
            }
        } else {
            self.update_remaining_time();
            self.start_instant = None;
            self.is_paused = true;
        }
    }

    fn reset_timer(&mut self) {
        self.remaining_time = self.get_current_duration();
        self.start_remaining = self.remaining_time;
        self.start_instant = None;
        self.is_paused = true;
    }

    fn skip_to_next(&mut self) {
        if self.timer_mode == TimerMode::Pomodoro {
            self.advance_pomodoro_state();
        }
    }

    fn toggle_mode(&mut self) {
        match self.timer_mode {
            TimerMode::Pomodoro => {
                self.timer_mode = TimerMode::Timer(self.config.work_duration_mins * 60);
                self.remaining_time = Duration::from_secs(self.config.work_duration_mins * 60);
            }
            TimerMode::Timer(_) => {
                self.timer_mode = TimerMode::Pomodoro;
                self.timer_state = TimerState::Work;
                self.remaining_time = self.duration_for_state(TimerState::Work);
                self.session_count = 0;
            }
        }
        self.start_remaining = self.remaining_time;
        self.start_instant = None;
        self.is_paused = true;
    }

    fn get_current_duration(&self) -> Duration {
        match self.timer_mode {
            TimerMode::Pomodoro => self.duration_for_state(self.timer_state),
            TimerMode::Timer(secs) => Duration::from_secs(secs),
        }
    }

    fn update_remaining_time(&mut self) {
        if let Some(start) = self.start_instant {
            let elapsed = start.elapsed();
            self.remaining_time = self.start_remaining.saturating_sub(elapsed);
        }
    }

    fn advance_pomodoro_state(&mut self) {
        if self.timer_state == TimerState::Work && !self.tasks.is_empty() {
            self.tasks[self.selected_task_index].pomodoros_spent += 1;
            self.needs_save = true;
        }

        match self.timer_state {
            TimerState::Work => {
                self.session_count += 1;
                if self.session_count >= self.sessions_before_long {
                    self.timer_state = TimerState::LongBreak;
                    self.session_count = 0;
                } else {
                    self.timer_state = TimerState::ShortBreak;
                }
            }
            TimerState::ShortBreak | TimerState::LongBreak => {
                self.timer_state = TimerState::Work;
            }
        }

        self.remaining_time = self.duration_for_state(self.timer_state);
        self.start_remaining = self.remaining_time;
        self.start_instant = None;
        self.is_paused = !self.config.auto_start_breaks || self.timer_state == TimerState::Work;

        if !self.is_paused {
            self.start_instant = Some(Instant::now());
        }
    }

    pub fn tick(&mut self) {
        // Update breathing animation
        if self.config.breathing_enabled && self.is_paused {
            self.breathing_phase = (self.breathing_phase + 2) % 100;
        }

        // Update celebration timer
        if self.show_celebration && self.celebration_timer > 0 {
            self.celebration_timer -= 1;
            if self.celebration_timer == 0 {
                self.show_celebration = false;
            }
        }

        // Update hint fade
        if self.config.hide_hints_after_secs > 0 && self.hints_visible {
            self.hint_fade_counter += 1;
            // Tick is ~100ms, so 10 ticks = 1 second
            if self.hint_fade_counter >= self.config.hide_hints_after_secs * 10 {
                self.hints_visible = false;
            }
        }

        if !self.is_paused {
            self.update_remaining_time();

            if self.remaining_time.is_zero() {
                self.on_timer_complete();
            }
        }

        if self.needs_save {
            self.save_tasks();
            self.needs_save = false;
        }
    }

    fn on_timer_complete(&mut self) {
        let task_name = if !self.tasks.is_empty() {
            Some(self.tasks[self.selected_task_index].name.clone())
        } else {
            None
        };

        // For work sessions, prompt for a note before saving
        if self.timer_state == TimerState::Work {
            // Increment pomodoro count for selected task
            if !self.tasks.is_empty() {
                self.tasks[self.selected_task_index].pomodoros_spent += 1;
                self.needs_save = true;
            }

            self.pending_session = Some((
                self.timer_state.session_type().to_string(),
                self.get_current_duration().as_secs(),
                task_name.clone(),
            ));
            self.input_mode = InputMode::SessionNote;
            self.input_buffer.clear();
            
            // Check for celebration triggers before showing note prompt
            self.check_celebrations();
        } else {
            // Breaks don't need notes
            let session = Session::new(
                self.timer_state.session_type(),
                self.get_current_duration().as_secs(),
                task_name.clone(),
            );
            self.session_history.add(session);
            let _ = self.session_history.save();
        }

        self.send_notification(&task_name);

        if self.timer_mode == TimerMode::Pomodoro {
            self.advance_pomodoro_state();
        } else {
            self.is_paused = true;
        }
    }

    /// Check and trigger celebration messages
    fn check_celebrations(&mut self) {
        let (completed, goal) = self.daily_goal_progress();
        
        // Daily goal reached exactly
        if completed + 1 == goal as usize {
            self.show_celebration = true;
            self.celebration_message = format!("🎉 Daily goal reached! {} pomodoros!", goal);
            self.celebration_timer = 50; // 5 seconds at 100ms tick
            return;
        }

        // Streak milestones
        let streak = self.session_history.current_streak;
        if streak == 7 {
            self.show_celebration = true;
            self.celebration_message = "🔥 Amazing! 7-day streak!".to_string();
            self.celebration_timer = 50;
        } else if streak == 30 {
            self.show_celebration = true;
            self.celebration_message = "⭐ Incredible! 30-day streak!".to_string();
            self.celebration_timer = 50;
        } else if streak == 100 {
            self.show_celebration = true;
            self.celebration_message = "🏆 LEGENDARY! 100-day streak!".to_string();
            self.celebration_timer = 50;
        }

        // Hourly milestone
        let today_mins = self.session_history.today_focus_secs() / 60;
        if today_mins >= 60 && today_mins < 85 {
            self.show_celebration = true;
            self.celebration_message = "💪 1 hour of focus today!".to_string();
            self.celebration_timer = 40;
        } else if today_mins >= 120 && today_mins < 145 {
            self.show_celebration = true;
            self.celebration_message = "🚀 2 hours of focus today!".to_string();
            self.celebration_timer = 40;
        }
    }

    /// Complete pending session with note
    fn complete_pending_session(&mut self, note: Option<String>) {
        if let Some((session_type, duration, task_name)) = self.pending_session.take() {
            let session = Session::with_note(&session_type, duration, task_name, note);
            self.session_history.add(session);
            let _ = self.session_history.save();
        }
    }

    fn save_tasks(&self) {
        use crate::persistence::tasks::{TaskData, TaskStore};

        let tasks: Vec<TaskData> = self
            .tasks
            .iter()
            .map(|t| TaskData {
                id: t.id,
                name: t.name.clone(),
                completed: t.completed,
                pomodoros_spent: t.pomodoros_spent,
                tags: t.tags.clone(),
                created_at: chrono::Utc::now(),
            })
            .collect();

        let store = TaskStore { tasks };
        let _ = store.save();
    }

    fn save_all(&self) {
        self.save_tasks();
        let _ = self.config.save();
        let _ = self.session_history.save();
    }

    #[cfg(feature = "notifications")]
    fn send_notification(&self, task_name: &Option<String>) {
        if !self.config.notifications_enabled {
            return;
        }

        let title = match self.timer_state {
            TimerState::Work => "🍅 Work session complete!",
            TimerState::ShortBreak => "☕ Short break over!",
            TimerState::LongBreak => "🌴 Long break over!",
        };

        let body = match task_name {
            Some(name) => format!("Task: {}", name),
            None => "Time for the next phase!".to_string(),
        };

        let _ = notify_rust::Notification::new()
            .summary(title)
            .body(&body)
            .timeout(5000)
            .show();
    }

    #[cfg(not(feature = "notifications"))]
    fn send_notification(&self, _task_name: &Option<String>) {}

    pub fn formatted_time(&self) -> String {
        let total_secs = self.remaining_time.as_secs();
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    pub fn progress(&self) -> f64 {
        let total = self.get_current_duration().as_secs_f64();
        let remaining = self.remaining_time.as_secs_f64();
        if total > 0.0 {
            1.0 - (remaining / total)
        } else {
            0.0
        }
    }



    pub fn mode_display(&self) -> String {
        match self.timer_mode {
            TimerMode::Pomodoro => format!("● Pomodoro: {}", self.timer_state.display_name()),
            TimerMode::Timer(_) => "○ Timer Mode".to_string(),
        }
    }

    /// Get daily goal progress
    pub fn daily_goal_progress(&self) -> (usize, u8) {
        let completed = self.session_history.today_pomodoro_count();
        let goal = self.config.daily_goal_pomodoros;
        (completed, goal)
    }

    /// Get breathing color modifier (0.0 to 1.0)
    pub fn breathing_intensity(&self) -> f32 {
        // Sine wave from 0.5 to 1.0
        let phase = (self.breathing_phase as f32 / 100.0) * std::f32::consts::PI * 2.0;
        0.5 + 0.5 * phase.sin().abs()
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
