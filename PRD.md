# Product Requirements Document: Pomo-TUI (Rust)

## 1. Project Overview

**Name:** `pomo-tui`  
**Goal:** A high-performance, aesthetically pleasing Terminal User Interface (TUI) Pomodoro timer for Linux with comprehensive session tracking and beautiful metrics visualization.  
**Aesthetic:** Minimalist, "Spotify-TUI" inspired. Dark mode default. Uses block characters and smooth rendering.  
**Core Function:** Track productivity using Pomodoro intervals (25/5/15) or a flexible timer mode, with task management, session persistence, and a metrics dashboard.

---

## 2. Technical Stack

| Component | Technology | Notes |
|-----------|------------|-------|
| **OS** | Linux (Fedora target) | Future: macOS, Windows |
| **Language** | Rust (Edition 2021) | |
| **UI Framework** | `ratatui` (latest) | Terminal rendering |
| **Backend** | `crossterm` | Raw terminal handling |
| **Time** | `std::time` | Monotonic time (drift prevention) |
| **Audio** | `rodio` | **Optional feature flag** |
| **Notifications** | `notify-rust` | Desktop alerts |
| **Styling** | `tui-big-text` | Large timer display |
| **Persistence** | `serde` + `serde_json` | Session/config storage |
| **Directories** | `dirs` | XDG-compliant paths |

### Cargo Feature Flags

```toml
[features]
default = ["notifications"]
audio = ["rodio"]  # Optional: requires libalsa on Linux
notifications = ["notify-rust"]
```

---

## 3. Timer Modes

The application supports two distinct timer modes:

### 3.1 Pomodoro Mode (Default)

Classic Pomodoro Technique with auto-cycling:

| State | Duration | Description |
|-------|----------|-------------|
| **Work** | 25 min | Focused work session |
| **Short Break** | 5 min | Quick rest |
| **Long Break** | 15 min | Extended rest after 4 work sessions |

**Auto-Cycling Logic:**
```
Work → Short Break → Work → Short Break → Work → Short Break → Work → Long Break → (repeat)
```

The cycle counter resets after a Long Break. Users can skip states manually with `n`.

### 3.2 Timer Mode (Flexible)

A simple countdown timer with user-defined duration:
- No automatic state transitions
- User sets duration manually (1 min – 180 min)
- Useful for non-Pomodoro tasks (meetings, cooking, etc.)

---

## 4. UI Layout & Architecture

The UI uses `ratatui::layout` with multiple views accessible via tabs.

### 4.1 Views

| View | Description |
|------|-------------|
| **Timer View** | Main timer display with task list (default) |
| **Dashboard View** | Metrics and session history |
| **Settings View** | In-app configuration editor |

### 4.2 Timer View Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  POMO-TUI          [● Pomodoro Mode]           Press ? for help │  ← Header (10%)
├───────────────────────────────────────┬─────────────────────────┤
│                                       │  ┌─────────────────────┐│
│            ██  █████                  │  │ ▶ Write PRD        ││
│           ███  ██                     │  │   Review code       ││
│            ██     ██                  │  │   Test feature      ││
│            ██  █████                  │  │   Deploy app        ││
│                                       │  └─────────────────────┘│
│         ━━━━━━━━━━━━━━━━━━━░░░░░░░░   │                         │  ← Main (80%)
│              ▶ RUNNING                │     Sessions: 3/4       │
│                                       │     Cycle: Work         │
│                                       │                         │
├───────────────────────────────────────┴─────────────────────────┤
│  [q] Quit  [Space] Toggle  [r] Reset  [n] Skip  [Tab] Switch    │  ← Footer (10%)
└─────────────────────────────────────────────────────────────────┘
```

**Left Pane (Timer, 60%):**
- Large digital clock (MM:SS) using `tui-big-text`
- Progress gauge bar
- Status label (Running/Paused)

**Right Pane (Tasks/Info, 40%):**
- Scrollable task list with active task highlighted
- Current session count (e.g., "3/4" work sessions before long break)
- Current cycle state indicator

### 4.3 Dashboard View Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  POMO-TUI          [Dashboard]                 Press ? for help │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   TODAY           THIS WEEK         ALL TIME                    │
│   ┌─────────┐     ┌─────────┐       ┌─────────┐                 │
│   │  4h 25m │     │ 18h 30m │       │ 142h    │                 │
│   │ 8 sess. │     │ 42 sess.│       │ 340 ses.│                 │
│   └─────────┘     └─────────┘       └─────────┘                 │
│                                                                 │
│   WEEKLY ACTIVITY (Last 7 Days)                                 │
│   ████                                                          │
│   ████ ████                                                     │
│   ████ ████      ████                                           │
│   ████ ████ ████ ████ ████                                      │
│   Mon  Tue  Wed  Thu  Fri  Sat  Sun                             │
│                                                                 │
│   RECENT SESSIONS                                               │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │ 2024-01-15 14:30  │ Work    │ 25m │ ✓ Completed        │   │
│   │ 2024-01-15 14:00  │ Work    │ 25m │ ✓ Completed        │   │
│   │ 2024-01-15 13:30  │ Break   │ 5m  │ ✓ Completed        │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  [1] Timer  [2] Dashboard  [3] Settings  [q] Quit               │
└─────────────────────────────────────────────────────────────────┘
```

**Metrics Displayed:**
- **Summary Cards:** Today, This Week, All Time (focus time + session count)
- **Weekly Bar Chart:** Visual activity histogram for the past 7 days
- **Recent Sessions Table:** Scrollable list of completed sessions with timestamps

### 4.4 Settings View Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  POMO-TUI          [Settings]                  Press ? for help │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   TIMER SETTINGS                                                │
│   ├─ Work Duration          [25] min        ◀ ▶                 │
│   ├─ Short Break            [5]  min        ◀ ▶                 │
│   ├─ Long Break             [15] min        ◀ ▶                 │
│   └─ Sessions before Long   [4]             ◀ ▶                 │
│                                                                 │
│   MODE                                                          │
│   ├─ Default Mode           [Pomodoro ▼]                        │
│   └─ Auto-start Breaks      [Yes / No]                          │
│                                                                 │
│   NOTIFICATIONS                                                 │
│   ├─ Desktop Notifications  [Enabled]                           │
│   └─ Sound Alerts           [Disabled] (requires --audio flag)  │
│                                                                 │
│   APPEARANCE                                                    │
│   └─ Theme                  [Dark ▼]                            │
│                                                                 │
│   DATA                                                          │
│   ├─ Export Sessions        [JSON]                              │
│   └─ Clear All Data         [Confirm...]                        │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│  [↑/↓] Navigate  [←/→] Adjust  [Enter] Select  [Esc] Back       │
└─────────────────────────────────────────────────────────────────┘
```

All settings are editable in-app and persisted to the config file automatically.

---

## 5. Data Architecture

### 5.1 Application State (`App` Struct)

```rust
struct App {
    // Timer State
    timer_mode: TimerMode,          // Pomodoro or Timer
    timer_state: TimerState,        // Work, ShortBreak, LongBreak, Custom
    remaining_time: Duration,
    is_paused: bool,
    session_count: u8,              // Tracks work sessions (0-4)
    
    // Task Management
    tasks: Vec<Task>,
    selected_task_index: usize,
    
    // Navigation
    current_view: View,             // Timer, Dashboard, Settings
    active_pane: Pane,              // Left (Timer) or Right (Tasks)
    
    // Persistence
    session_history: Vec<Session>,
    config: Config,
}

enum TimerMode {
    Pomodoro,
    Timer(Duration),                // Custom duration
}

enum TimerState {
    Work,
    ShortBreak,
    LongBreak,
    Custom,
}

enum View {
    Timer,
    Dashboard,
    Settings,
}
```

### 5.2 Persistence Schema

**Config File:** `~/.config/pomo-tui/config.json`

```json
{
  "work_duration_mins": 25,
  "short_break_mins": 5,
  "long_break_mins": 15,
  "sessions_before_long_break": 4,
  "default_mode": "pomodoro",
  "auto_start_breaks": true,
  "notifications_enabled": true,
  "sound_enabled": false,
  "theme": "dark"
}
```

**Session History:** `~/.local/share/pomo-tui/sessions.json`

```json
{
  "sessions": [
    {
      "id": "uuid-v4",
      "timestamp": "2024-01-15T14:30:00Z",
      "type": "work",
      "duration_secs": 1500,
      "completed": true,
      "task": "Write PRD"
    }
  ]
}
```

**Tasks:** `~/.local/share/pomo-tui/tasks.json`

```json
{
  "tasks": [
    {
      "id": "uuid-v4",
      "name": "Write PRD",
      "created_at": "2024-01-15T10:00:00Z",
      "completed": false,
      "pomodoros_spent": 3
    }
  ]
}
```

---

## 6. Key Features & Controls

### 6.1 Global Keybindings

| Key | Action |
|-----|--------|
| `q` | Quit application |
| `?` | Toggle help overlay |
| `1` | Switch to Timer View |
| `2` | Switch to Dashboard View |
| `3` | Switch to Settings View |

### 6.2 Timer View Keybindings

| Key | Action |
|-----|--------|
| `Space` | Toggle timer (Start/Pause) |
| `r` | Reset current timer |
| `n` | Skip to next state (Work → Break) |
| `m` | Toggle mode (Pomodoro ↔ Timer) |
| `Tab` | Switch focus (Timer ↔ Tasks pane) |
| `j` / `↓` | Navigate down in task list |
| `k` / `↑` | Navigate up in task list |
| `a` | Add new task (opens input popup) |
| `d` | Delete selected task |
| `Enter` | Mark task as active/complete |

### 6.3 Settings View Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Navigate to next setting |
| `k` / `↑` | Navigate to previous setting |
| `h` / `←` | Decrease value / Previous option |
| `l` / `→` | Increase value / Next option |
| `Enter` | Confirm selection |
| `Esc` | Go back / Cancel |

### 6.4 Dashboard View Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down in session history |
| `k` / `↑` | Scroll up in session history |
| `e` | Export sessions to file |

---

## 7. Logic Requirements

### 7.1 Timer Logic

1. **Non-blocking Event Loop:** Use `crossterm::event::poll` with ~100ms tick rate for responsive UI while timer counts down.

2. **Drift Correction:** Calculate elapsed time using `Instant::now()` comparison against stored `start_time`. Never use naive `sleep(1)`.

3. **Auto-Cycling (Pomodoro Mode):**
   ```
   on timer_complete:
       if state == Work:
           session_count += 1
           if session_count >= 4:
               next_state = LongBreak
               session_count = 0
           else:
               next_state = ShortBreak
       else:
           next_state = Work
       
       if config.auto_start_breaks:
           start_timer(next_state)
       else:
           pause_at(next_state)
   ```

4. **Session Recording:** On timer completion, persist session to `sessions.json` with timestamp, duration, type, and associated task.

### 7.2 Notification Logic

1. When `remaining_time` hits zero:
   - Trigger desktop notification via `notify-rust`
   - If `audio` feature enabled and `sound_enabled`: play sound via `rodio`
   - Record completed session

### 7.3 Persistence Logic

1. **On Startup:**
   - Load config from `~/.config/pomo-tui/config.json` (create defaults if missing)
   - Load tasks from `~/.local/share/pomo-tui/tasks.json`
   - Load session history from `~/.local/share/pomo-tui/sessions.json`

2. **On Change:**
   - Debounce writes (max once per second) to prevent excessive I/O
   - Write config immediately on settings change
   - Append sessions on completion

3. **On Exit:**
   - Flush any pending writes
   - Restore terminal mode cleanly (even on panic via `std::panic::set_hook`)

---

## 8. Implementation Roadmap

### Phase 1: Project Skeleton
- [ ] Initialize `cargo new pomo-tui`
- [ ] Configure `Cargo.toml` with dependencies and feature flags
- [ ] Set up `main.rs` boilerplate: terminal setup, raw mode, main loop
- [ ] Create basic `draw` function rendering placeholder blocks
- [ ] Implement clean exit handling (restore terminal on quit/panic)

### Phase 2: Core Timer Logic
- [ ] Implement `App` struct with timer state
- [ ] Create countdown logic with drift correction
- [ ] Implement `tui-big-text` timer display
- [ ] Bind `Space` (pause/start), `r` (reset), `n` (skip)
- [ ] Implement Pomodoro auto-cycling with session counter

### Phase 3: Multi-Pane Layout ("Spotify" Style)
- [ ] Split UI into Header/Main/Footer
- [ ] Implement Left pane (timer + progress bar)
- [ ] Implement Right pane (task list)
- [ ] Add `Tab` navigation between panes
- [ ] Implement task CRUD (add/delete/mark complete)

### Phase 4: Persistence & Configuration
- [ ] Create config schema and defaults
- [ ] Implement XDG-compliant file paths (`dirs` crate)
- [ ] Load/save config, tasks, sessions
- [ ] Add Settings View with in-app editing

### Phase 5: Dashboard & Metrics
- [ ] Implement Dashboard View
- [ ] Create summary statistics (today/week/all-time)
- [ ] Render weekly activity bar chart
- [ ] Display scrollable session history table

### Phase 6: Timer Mode & Polish
- [ ] Implement flexible Timer Mode (custom duration)
- [ ] Add mode toggle (`m` key)
- [ ] Implement help overlay (`?`)
- [ ] Add notifications (`notify-rust`)
- [ ] Add optional audio (`rodio` behind feature flag)

### Phase 7: Release Preparation
- [ ] Write README.md with installation instructions
- [ ] Add CHANGELOG.md
- [ ] Create LICENSE (MIT/Apache-2.0)
- [ ] Set up GitHub Actions for CI (build + test)
- [ ] Create release workflow for binary artifacts

---

## 9. File Structure (Target)

```
pomo-tui/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── CHANGELOG.md
├── LICENSE
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── src/
│   ├── main.rs              # Entry point, terminal setup
│   ├── app.rs               # App state and update logic
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── timer_view.rs    # Timer view rendering
│   │   ├── dashboard_view.rs # Dashboard view rendering
│   │   ├── settings_view.rs # Settings view rendering
│   │   └── components/
│   │       ├── mod.rs
│   │       ├── timer.rs     # Big text timer widget
│   │       ├── progress.rs  # Progress gauge widget
│   │       ├── task_list.rs # Task list widget
│   │       └── chart.rs     # Bar chart widget
│   ├── timer/
│   │   ├── mod.rs
│   │   └── pomodoro.rs      # Pomodoro logic & cycling
│   ├── persistence/
│   │   ├── mod.rs
│   │   ├── config.rs        # Config loading/saving
│   │   ├── sessions.rs      # Session history
│   │   └── tasks.rs         # Task persistence
│   └── utils/
│       ├── mod.rs
│       └── time.rs          # Time formatting helpers
└── assets/
    └── sounds/
        └── bell.wav         # Optional notification sound
```

---

## 10. Success Criteria

1. **Performance:** UI renders at 60fps equivalent with no visible lag
2. **Reliability:** No data loss on crash (debounced writes + flush on exit)
3. **Usability:** All features accessible via keyboard; no mouse required
4. **Aesthetics:** Visually comparable to `spotify-tui` quality
5. **Portability:** Compiles on stable Rust; runs on Fedora Linux
