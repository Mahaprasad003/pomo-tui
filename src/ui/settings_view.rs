use crate::app::{App, InputMode, SettingsCategory, SettingsField};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Draw the settings view
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(area);

    draw_header(frame, chunks[0]);
    draw_settings_content(frame, chunks[1], app);
    draw_footer(frame, chunks[2]);

    // Confirm reset popup
    if app.input_mode == InputMode::ConfirmReset {
        draw_confirm_reset_popup(frame, app);
    }
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    let title = Paragraph::new(Line::from(vec![
        Span::styled("🍅 ", Style::default().fg(Color::Red)),
        Span::styled("POMO-TUI", Style::default().fg(Color::Cyan).bold()),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(title, chunks[0]);

    let mode = Paragraph::new("⚙ Settings")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center);
    frame.render_widget(mode, chunks[1]);

    let help = Paragraph::new("Press 1 for Timer")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Right);
    frame.render_widget(help, chunks[2]);
}

fn draw_settings_content(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" Settings ");

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Calculate total lines needed
    let mut lines = Vec::new();
    let all_fields = SettingsField::all();
    let mut current_category: Option<SettingsCategory> = None;

    for field in &all_fields {
        let field_category = field.category();
        
        // Add category header if changed
        if current_category.map(|c| c != field_category).unwrap_or(true) {
            if current_category.is_some() {
                lines.push(Line::from("")); // Spacing
            }
            lines.push(Line::from(Span::styled(
                field_category.name(),
                Style::default().fg(Color::Cyan).bold(),
            )));
            current_category = Some(field_category);
        }

        // Add setting row
        let is_selected = *field == app.selected_setting;
        lines.push(make_setting_line(field, app, is_selected));
    }

    // Add streak info (read-only)
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "STATS (read-only)",
        Style::default().fg(Color::Cyan).bold(),
    )));
    lines.push(Line::from(vec![
        Span::styled("  Current Streak", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("                  🔥 {} days", app.session_history.current_streak),
            Style::default().fg(Color::Yellow),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Longest Streak", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("                  ⭐ {} days", app.session_history.longest_streak),
            Style::default().fg(Color::Magenta),
        ),
    ]));

    let settings = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE));
    
    frame.render_widget(settings, inner_area);
}

fn make_setting_line(field: &SettingsField, app: &App, is_selected: bool) -> Line<'static> {
    let pointer = if is_selected { "▸ " } else { "  " };
    let label_style = if is_selected {
        Style::default().fg(Color::White).bold()
    } else {
        Style::default().fg(Color::Gray)
    };

    let value_style = if is_selected {
        Style::default().fg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let (label, value) = match field {
        SettingsField::WorkDuration => (
            "Work Duration",
            format!("{} min", app.config.work_duration_mins),
        ),
        SettingsField::ShortBreak => (
            "Short Break",
            format!("{} min", app.config.short_break_mins),
        ),
        SettingsField::LongBreak => (
            "Long Break",
            format!("{} min", app.config.long_break_mins),
        ),
        SettingsField::SessionsBeforeLong => (
            "Sessions before Long",
            format!("{}", app.config.sessions_before_long_break),
        ),
        SettingsField::DailyGoal => (
            "Daily Goal",
            format!("{} pomodoros", app.config.daily_goal_pomodoros),
        ),
        SettingsField::ShowStreak => (
            "Show Streak",
            if app.config.show_streak { "Yes" } else { "No" }.to_string(),
        ),
        SettingsField::BreathingAnimation => (
            "Breathing Animation",
            if app.config.breathing_enabled { "On" } else { "Off" }.to_string(),
        ),
        SettingsField::HideHintsAfter => (
            "Hide Hints After",
            if app.config.hide_hints_after_secs == 0 {
                "Never".to_string()
            } else {
                format!("{} sec", app.config.hide_hints_after_secs)
            },
        ),
        SettingsField::AutoStartBreaks => (
            "Auto-start Breaks",
            if app.config.auto_start_breaks { "Yes" } else { "No" }.to_string(),
        ),
        SettingsField::FocusModeOnStart => (
            "Focus Mode on Start",
            if app.config.focus_mode_on_start { "Yes" } else { "No" }.to_string(),
        ),
        SettingsField::NotificationsEnabled => (
            "Desktop Notifications",
            if app.config.notifications_enabled { "Enabled" } else { "Disabled" }.to_string(),
        ),
        SettingsField::ResetData => (
            "🗑 Reset All Data",
            "Press Enter to reset...".to_string(),
        ),
    };

    let arrows = if is_selected { "◀ " } else { "  " };
    let arrows_end = if is_selected { " ▶" } else { "  " };

    // Calculate padding
    let label_len = label.len() + pointer.len();
    let _value_len = value.len() + 4; // arrows
    let padding = 35_usize.saturating_sub(label_len);

    Line::from(vec![
        Span::styled(pointer, label_style),
        Span::styled(label, label_style),
        Span::raw(" ".repeat(padding)),
        Span::styled(arrows, value_style),
        Span::styled(value, value_style),
        Span::styled(arrows_end, value_style),
    ])
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let hints = Line::from(vec![
        Span::styled("[j/k]", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" Navigate  "),
        Span::styled("[←/→]", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" Adjust  "),
        Span::styled("[1]", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Timer  "),
        Span::styled("[2]", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Dashboard  "),
        Span::styled("[q]", Style::default().fg(Color::Red).bold()),
        Span::raw(" Quit"),
    ]);

    let footer = Paragraph::new(hints)
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}

/// Draw confirm reset popup
fn draw_confirm_reset_popup(frame: &mut Frame, app: &App) {
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

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::DOUBLE)
        .border_style(Style::default().fg(Color::Red))
        .title(" ⚠️ RESET ALL DATA ")
        .title_style(Style::default().fg(Color::Red).bold());

    let inner_area = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .split(inner_area);

    let warning = Paragraph::new("This will delete all sessions, tasks,\nand stats. This cannot be undone!")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center);
    frame.render_widget(warning, chunks[0]);

    // Input with feedback
    let input_color = if app.input_buffer == "DELETE" {
        Color::Green
    } else {
        Color::White
    };
    let input_text = format!("Type DELETE: {}│", app.input_buffer);
    let input = Paragraph::new(input_text)
        .style(Style::default().fg(input_color))
        .alignment(Alignment::Center);
    frame.render_widget(input, chunks[1]);

    let hint = Paragraph::new("Enter ▸ confirm │ Esc ▸ cancel")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[2]);
}
