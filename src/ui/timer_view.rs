use crate::app::{ActivePane, App, InputMode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph},
    Frame,
};
use tui_big_text::{BigText, PixelSize};

/// Draw the timer view with header, main content, and footer
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    if app.focus_mode {
        draw_focus_mode(frame, area, app);
    } else {
        draw_normal_mode(frame, area, app);
    }

    // Draw overlays (always on top, in order of priority)
    if app.input_mode == InputMode::AddingTask {
        draw_input_popup(frame, app, "Add Task");
    } else if app.input_mode == InputMode::EditingTask {
        draw_input_popup(frame, app, "Edit Task");
    } else if app.input_mode == InputMode::QuickCapture {
        draw_input_popup(frame, app, "Quick Capture");
    }
    // Session note popup removed for inline flow (Steve Jobs polish)

    // Celebration overlay (top priority)
    if app.show_celebration {
        draw_celebration_overlay(frame, app);
    }

    if app.show_help {
        draw_help_overlay(frame);
    }
}

/// Draw normal mode with all panes
fn draw_normal_mode(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(area);

    draw_header(frame, chunks[0], app);
    draw_main_content(frame, chunks[1], app);
    
    if app.hints_visible {
        draw_footer(frame, chunks[2], app);
    }
}

/// Draw focus mode - full screen timer
fn draw_focus_mode(frame: &mut Frame, area: Rect, app: &App) {
    let state_color = get_breathing_color(app);

    // Minimal header
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    // Minimal header with exit hint
    let header = Paragraph::new(Line::from(vec![
        Span::styled("Focus Mode", Style::default().fg(state_color).bold()),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled("Esc", Style::default().fg(Color::DarkGray)),
        Span::styled(" to exit", Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(header, chunks[0]);

    // Large centered timer
    draw_focus_timer(frame, chunks[1], app);

    // Daily goal progress
    draw_daily_goal_bar(frame, chunks[2], app);
}

/// Draw the big timer in focus mode
fn draw_focus_timer(frame: &mut Frame, area: Rect, app: &App) {
    let state_color = get_breathing_color(app);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ])
        .split(area);

    // Big timer
    let time_str = app.formatted_time();
    let big_text = BigText::builder()
        .pixel_size(PixelSize::Full)
        .style(Style::default().fg(state_color).bold())
        .lines(vec![time_str.into()])
        .centered()
        .build();
    frame.render_widget(big_text, inner[1]);

    // Status
    let (status_text, status_icon) = if app.is_paused {
        ("PAUSED", "⏸")
    } else {
        ("RUNNING", "▶")
    };
    let status_color = if app.is_paused { Color::Yellow } else { Color::Green };
    let status = Paragraph::new(format!("{} {}", status_icon, status_text))
        .style(Style::default().fg(status_color).bold())
        .alignment(Alignment::Center);
    frame.render_widget(status, inner[2]);

    // Session dots
    if matches!(app.timer_mode, crate::app::TimerMode::Pomodoro) {
        let session_dots = get_session_dots(app);
        let dots = Paragraph::new(session_dots)
            .style(Style::default().fg(state_color))
            .alignment(Alignment::Center);
        frame.render_widget(dots, inner[3]);
    }
}

/// Get color with breathing effect applied
fn get_breathing_color(app: &App) -> Color {
    let base_color = app.timer_state.color();
    
    if app.config.breathing_enabled && app.is_paused {
        let intensity = app.breathing_intensity();
        // Modulate brightness
        match base_color {
            Color::Cyan => Color::Rgb(
                (0.0 + 100.0 * intensity) as u8,
                (139.0 + 80.0 * intensity) as u8,
                (139.0 + 80.0 * intensity) as u8,
            ),
            Color::Green => Color::Rgb(
                (0.0 + 50.0 * intensity) as u8,
                (128.0 + 100.0 * intensity) as u8,
                (0.0 + 50.0 * intensity) as u8,
            ),
            Color::Magenta => Color::Rgb(
                (139.0 + 80.0 * intensity) as u8,
                (0.0 + 50.0 * intensity) as u8,
                (139.0 + 80.0 * intensity) as u8,
            ),
            _ => base_color,
        }
    } else {
        base_color
    }
}

/// Get session progress dots
fn get_session_dots(app: &App) -> String {
    (0..app.sessions_before_long)
        .map(|i| {
            if i < app.session_count {
                "●"
            } else if i == app.session_count {
                "◐"
            } else {
                "○"
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Draw daily goal progress bar
fn draw_daily_goal_bar(frame: &mut Frame, area: Rect, app: &App) {
    let (completed, goal) = app.daily_goal_progress();
    let progress = (completed as f64 / goal as f64).min(1.0);
    
    let goal_text = if completed >= goal as usize {
        format!("🎯 Daily Goal Complete! ({}/{})", completed, goal)
    } else {
        format!("🎯 {}/{} pomodoros today", completed, goal)
    };
    
    let color = if completed >= goal as usize {
        Color::Green
    } else {
        Color::Yellow
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area);

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(color).bg(Color::Rgb(40, 40, 40)))
        .ratio(progress)
        .label(goal_text);
    
    frame.render_widget(gauge, chunks[1]);
}

/// Draw the header with title, mode indicator, and help hint
fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    // Left: Greeting with streak
    let streak_display = if app.config.show_streak && app.session_history.current_streak > 0 {
        format!(" 🔥{}", app.session_history.current_streak)
    } else {
        String::new()
    };
    
    let greeting = app.greeting();
    let title = Paragraph::new(Line::from(vec![
        Span::styled(format!("{} ", greeting), Style::default().fg(Color::DarkGray)),
        Span::styled("│ ", Style::default().fg(Color::DarkGray)),
        Span::styled("🍅", Style::default().fg(Color::Red)),
        Span::styled(streak_display, Style::default().fg(Color::Yellow)),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(title, chunks[0]);

    // Center: Mode + estimated end time (when running)
    let (completed, goal) = app.daily_goal_progress();
    let end_time_display = if !app.is_paused {
        format!(" → {}", app.estimated_end_time())
    } else {
        String::new()
    };
    let mode_color = get_breathing_color(app);
    
    let mode = Paragraph::new(Line::from(vec![
        Span::styled(app.mode_display(), Style::default().fg(mode_color)),
        Span::styled(end_time_display, Style::default().fg(Color::DarkGray)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(mode, chunks[1]);

    // Right: Goal progress or late night warning
    let right_content = if app.is_late_night() {
        Line::from(vec![
            Span::styled("🌙 ", Style::default().fg(Color::Yellow)),
            Span::styled("It's late!", Style::default().fg(Color::Yellow)),
        ])
    } else {
        Line::from(vec![
            Span::styled(format!("{}/{} ", completed, goal), Style::default().fg(Color::DarkGray)),
            Span::styled("🎯", Style::default().fg(Color::Yellow)),
        ])
    };
    let help = Paragraph::new(right_content)
        .alignment(Alignment::Right);
    frame.render_widget(help, chunks[2]);
}

/// Draw the main content area with task and timer panes
fn draw_main_content(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    draw_task_pane(frame, chunks[0], app);
    draw_timer_pane(frame, chunks[1], app);
}

/// Draw the timer pane with countdown display
fn draw_timer_pane(frame: &mut Frame, area: Rect, app: &App) {
    let state_color = get_breathing_color(app);
    let is_focused = app.active_pane == ActivePane::Timer;

    let border_color = if is_focused { state_color } else { Color::DarkGray };
    let title = format!(" {} Timer ", get_state_icon(app));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title)
        .title_style(Style::default().fg(state_color).bold());

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Steve Jobs Polish: Massive whitespace and vertical centering
    
    // Calculate vertical centering
    // We want the big timer to be optically centered.
    // Total height needed for elements: 
    // Spacer(prev 1) + Timer(8) + Spacer(2) + Progress(1) + Spacer(1) + Status/Note(1) + Spacer(1) + Sessions(1)
    // Approx 16 lines of content.
    
    let vertical_center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(2), // Top spacer (expands)
            Constraint::Length(16), // Content area
            Constraint::Min(2), // Bottom spacer (expands)
        ])
        .split(inner_area);

    let content_area = vertical_center[1];

    let timer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Big Timer
            Constraint::Length(2), // Spacer
            Constraint::Length(1), // Progress
            Constraint::Length(2), // Spacer
            Constraint::Length(1), // Status or Inline Note
            Constraint::Length(2), // Spacer
            Constraint::Length(1), // Session info
        ])
        .split(content_area);

    // Big timer
    draw_big_timer(frame, timer_chunks[0], app);
    
    // Progress
    draw_enhanced_progress(frame, timer_chunks[2], app);
    
    // Status OR Inline Note
    if app.input_mode == InputMode::SessionNote {
        draw_inline_note_input(frame, timer_chunks[4], app);
    } else {
        draw_status(frame, timer_chunks[4], app);
    }
    
    // Session info
    draw_session_info(frame, timer_chunks[6], app);
}

fn draw_inline_note_input(frame: &mut Frame, area: Rect, app: &App) {
    let input_text = if app.input_buffer.is_empty() {
        "What did you achieve? (Type, or Space to skip)"
    } else {
        &app.input_buffer
    };
    
    let text_color = if app.input_buffer.is_empty() {
        Color::DarkGray
    } else {
        Color::Yellow
    };

    let cursor = if app.celebration_timer % 10 < 5 { "█" } else { " " }; // Pulsing cursor using existing timer
    
    // Only show cursor if buffer is not empty or we are focused
    let display_text = if app.input_buffer.is_empty() {
         format!("{} {}", input_text, cursor)
    } else {
         format!("{} {}", input_text, cursor)
    };

    let p = Paragraph::new(display_text)
        .style(Style::default().fg(text_color))
        .alignment(Alignment::Center);
        
    frame.render_widget(p, area);
}



fn get_state_icon(app: &App) -> &'static str {
    match app.timer_state {
        crate::app::TimerState::Work => "🍅",
        crate::app::TimerState::ShortBreak => "☕",
        crate::app::TimerState::LongBreak => "🌴",
    }
}

fn draw_big_timer(frame: &mut Frame, area: Rect, app: &App) {
    let time_str = app.formatted_time();
    let state_color = get_breathing_color(app);

    let big_text = BigText::builder()
        .pixel_size(PixelSize::Quadrant)
        .style(Style::default().fg(state_color).bold())
        .lines(vec![time_str.into()])
        .centered()
        .build();

    frame.render_widget(big_text, area);
}

fn draw_enhanced_progress(frame: &mut Frame, area: Rect, app: &App) {
    let state_color = get_breathing_color(app);
    let progress = app.progress();
    let percentage = (progress * 100.0) as u16;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Left spacer
            Constraint::Percentage(6),  // Percentage text
            Constraint::Percentage(48), // Gauge
            Constraint::Percentage(26), // Right spacer
        ])
        .split(area);

    let left_pct = Paragraph::new(format!("{}%", percentage))
        .style(Style::default().fg(state_color).bold())
        .alignment(Alignment::Right);
    frame.render_widget(left_pct, chunks[1]);

    let gauge_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(chunks[2])[1];

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(state_color).bg(Color::Rgb(40, 40, 40)))
        .ratio(progress)
        .label("");
    frame.render_widget(gauge, gauge_area);
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let (status_text, status_color, status_icon) = if app.is_paused {
        ("PAUSED", Color::Yellow, "⏸")
    } else {
        ("RUNNING", Color::Green, "▶")
    };

    let status_line = Line::from(vec![
        Span::styled(format!(" {} ", status_icon), Style::default().fg(status_color)),
        Span::styled(status_text, Style::default().fg(status_color).bold()),
        Span::styled(format!(" {} ", status_icon), Style::default().fg(status_color)),
    ]);

    let status = Paragraph::new(status_line).alignment(Alignment::Center);
    frame.render_widget(status, area);
}

fn draw_session_info(frame: &mut Frame, area: Rect, app: &App) {
    if !matches!(app.timer_mode, crate::app::TimerMode::Pomodoro) {
        let timer_info = Line::from(vec![
            Span::styled("⏱ ", Style::default().fg(Color::Blue)),
            Span::styled("Timer Mode", Style::default().fg(Color::Gray)),
            Span::styled(" ⏱", Style::default().fg(Color::Blue)),
        ]);
        let widget = Paragraph::new(timer_info).alignment(Alignment::Center);
        frame.render_widget(widget, area);
        return;
    }

    let session_dots = get_session_dots(app);
    let state_color = get_breathing_color(app);

    let session_line = Line::from(vec![
        Span::styled("Sessions: ", Style::default().fg(Color::DarkGray)),
        Span::styled(session_dots, Style::default().fg(state_color)),
        Span::styled(
            format!(" │ {}", app.timer_state.display_name()),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let session_widget = Paragraph::new(session_line).alignment(Alignment::Center);
    frame.render_widget(session_widget, area);
}

/// Draw the task pane with task list
fn draw_task_pane(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.active_pane == ActivePane::Tasks;
    let border_color = if is_focused { Color::Magenta } else { Color::DarkGray };

    let task_count = app.tasks.len();
    let completed_count = app.tasks.iter().filter(|t| t.completed).count();

    let title = if is_focused {
        format!(" 📋 Tasks ({}/{}) ", completed_count, task_count)
    } else {
        format!(" Tasks ({}/{}) ", completed_count, task_count)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title)
        .title_style(Style::default().fg(Color::Magenta).bold());

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if app.tasks.is_empty() {
        let empty_lines = vec![
            Line::from(""),
            Line::from(Span::styled("No tasks yet", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("Press 'a' or '/' to add", Style::default().fg(Color::Yellow))),
        ];
        let empty_msg = Paragraph::new(empty_lines).alignment(Alignment::Center);
        frame.render_widget(empty_msg, inner_area);
        return;
    }

    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let is_selected = i == app.selected_task_index;

            let checkbox = if task.completed { "✓" } else { "○" };
            let checkbox_color = if task.completed { Color::Green } else { Color::DarkGray };
            let pointer = if is_selected { "▸" } else { " " };
            let pointer_color = if is_selected { Color::Magenta } else { Color::DarkGray };

            let pomodoro_display = if task.pomodoros_spent > 0 {
                format!(" 🍅×{}", task.pomodoros_spent)
            } else {
                String::new()
            };

            // Build tag display
            let tags_display: Vec<Span> = task
                .tags
                .iter()
                .map(|tag| {
                    Span::styled(
                        format!(" #{}", tag),
                        Style::default().fg(Color::Blue).add_modifier(Modifier::DIM),
                    )
                })
                .collect();

            let name_style = if is_selected {
                if task.completed {
                    Style::default().fg(Color::Green).bold()
                } else {
                    Style::default().fg(Color::White).bold()
                }
            } else if task.completed {
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT)
            } else {
                Style::default().fg(Color::Gray)
            };

            // Truncate long task names
            let display_name = if task.name.len() > 25 {
                format!("{}…", &task.name[..24])
            } else {
                task.name.clone()
            };

            let mut spans = vec![
                Span::styled(format!("{} ", pointer), Style::default().fg(pointer_color)),
                Span::styled(format!("{} ", checkbox), Style::default().fg(checkbox_color)),
                Span::styled(display_name, name_style),
                Span::styled(pomodoro_display, Style::default().fg(Color::Red)),
            ];
            spans.extend(tags_display);

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner_area);
}

/// Draw input popup for adding a new task
fn draw_input_popup(frame: &mut Frame, app: &App, title: &str) {
    let area = frame.area();

    let popup_width = 58.min(area.width.saturating_sub(4));
    let popup_height = 9;

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(format!(" ✏️ {} ", title))
        .title_style(Style::default().fg(Color::Yellow).bold());

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Prompt
            Constraint::Length(1), // Input with ghost text
            Constraint::Length(1), // Recent tags
            Constraint::Length(2), // Hints
        ])
        .split(inner_area);

    // Prompt
    let prompt = Paragraph::new("Task name (use #tag for tags):")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(prompt, chunks[0]);

    // Input with ghost text suggestion
    let mut input_spans = vec![
        Span::styled(&app.input_buffer, Style::default().fg(Color::White)),
    ];

    // Show ghost text if there's a suggestion
    if let Some(ref suggestion) = app.tag_suggestion {
        // Find what we've already typed after #
        if let Some(last_hash_pos) = app.input_buffer.rfind('#') {
            let typed_part = &app.input_buffer[last_hash_pos + 1..];
            if suggestion.len() > typed_part.len() {
                let ghost_part = &suggestion[typed_part.len()..];
                input_spans.push(Span::styled(
                    ghost_part,
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                ));
            }
        }
    }

    input_spans.push(Span::styled("│", Style::default().fg(Color::Yellow)));

    let input = Paragraph::new(Line::from(input_spans));
    frame.render_widget(input, chunks[1]);

    // Recent tags row
    let recent_tags = app.recent_tags();
    if !recent_tags.is_empty() {
        let mut tag_spans = vec![
            Span::styled("Recent: ", Style::default().fg(Color::DarkGray)),
        ];
        for (i, tag) in recent_tags.iter().take(5).enumerate() {
            if i > 0 {
                tag_spans.push(Span::raw(" "));
            }
            tag_spans.push(Span::styled(
                format!("#{}", tag),
                Style::default().fg(Color::Blue),
            ));
        }
        let recent = Paragraph::new(Line::from(tag_spans));
        frame.render_widget(recent, chunks[2]);
    }

    // Hints - show Tab if suggestion available
    let hint_text = if app.tag_suggestion.is_some() {
        "Tab ▸ accept │ Enter ▸ save │ Esc ▸ cancel"
    } else {
        "Enter ▸ save │ Esc ▸ cancel"
    };
    let hint = Paragraph::new(hint_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[3]);
}

/// Draw the footer with keybinding hints
fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let hints = if app.active_pane == ActivePane::Tasks {
        Line::from(vec![
            Span::styled("[a]", Style::default().fg(Color::Green).bold()),
            Span::raw(" Add  "),
            Span::styled("[e]", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" Edit  "),
            Span::styled("[d]", Style::default().fg(Color::Red).bold()),
            Span::raw(" Del  "),
            Span::styled("[c]", Style::default().fg(Color::Magenta).bold()),
            Span::raw(" Clear  "),
            Span::styled("[⏎]", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" Done  "),
            Span::styled("[f]", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" Focus"),
        ])
    } else {
        Line::from(vec![
            Span::styled("[␣]", Style::default().fg(Color::Green).bold()),
            Span::raw(" Play  "),
            Span::styled("[r]", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" Reset  "),
            Span::styled("[n]", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" Skip  "),
            Span::styled("[f]", Style::default().fg(Color::Magenta).bold()),
            Span::raw(" Focus  "),
            Span::styled("[/]", Style::default().fg(Color::Blue).bold()),
            Span::raw(" Quick"),
        ])
    };

    let footer = Paragraph::new(hints)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}

/// Draw the help overlay popup
fn draw_help_overlay(frame: &mut Frame) {
    let area = frame.area();

    let popup_width = 40.min(area.width.saturating_sub(4));
    let popup_height = 20.min(area.height.saturating_sub(4));

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" ⌨ Shortcuts ")
        .title_style(Style::default().fg(Color::Cyan).bold());

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let help_items = vec![
        ("Space", "Start / Pause"),
        ("r", "Reset timer"),
        ("n", "Skip to next"),
        ("m", "Toggle mode"),
        ("f", "Focus mode"),
        ("Tab", "Switch pane"),
        ("j / k", "Navigate"),
        ("a", "Add task"),
        ("e", "Edit task"),
        ("d", "Delete task"),
        ("c", "Clear completed"),
        ("/", "Quick capture"),
        ("Enter", "Toggle done"),
        ("1 2 3", "Switch view"),
        ("q", "Quit"),
    ];

    let help_lines: Vec<Line> = help_items
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(format!("{:>8}", key), Style::default().fg(Color::Yellow).bold()),
                Span::styled("  ", Style::default()),
                Span::styled(*desc, Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let help = Paragraph::new(help_lines);
    frame.render_widget(help, inner_area);
}



/// Draw celebration overlay with confetti
fn draw_celebration_overlay(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let popup_width = 45.min(area.width.saturating_sub(4));
    let popup_height = 9;

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    // Confetti characters that animate based on timer
    let confetti_chars = ['✦', '✧', '★', '☆', '✨', '⭐', '🌟'];
    let phase = app.celebration_timer as usize % confetti_chars.len();
    
    let confetti_line: String = (0..popup_width as usize - 2)
        .map(|i| confetti_chars[(i + phase) % confetti_chars.len()])
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::DOUBLE)
        .border_style(Style::default().fg(Color::Yellow));

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let celebration_text = vec![
        Line::from(Span::styled(
            &confetti_line,
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled(
            &app.celebration_message,
            Style::default().fg(Color::White).bold().add_modifier(Modifier::SLOW_BLINK),
        )),
        Line::from(""),
        Line::from(Span::styled(
            &confetti_line,
            Style::default().fg(Color::Magenta),
        )),
    ];

    let celebration = Paragraph::new(celebration_text)
        .alignment(Alignment::Center);
    
    frame.render_widget(celebration, inner_area);
}
