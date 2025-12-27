use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

/// Draw the dashboard view
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

    draw_header(frame, chunks[0], app);
    draw_main_content(frame, chunks[1], app);
    draw_footer(frame, chunks[2]);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    let streak_display = if app.session_history.current_streak > 0 {
        format!(" 🔥{}", app.session_history.current_streak)
    } else {
        String::new()
    };

    let title = Paragraph::new(Line::from(vec![
        Span::styled("🍅 ", Style::default().fg(Color::Red)),
        Span::styled("POMO-TUI", Style::default().fg(Color::Cyan).bold()),
        Span::styled(streak_display, Style::default().fg(Color::Yellow)),
    ]))
    .alignment(Alignment::Left);
    frame.render_widget(title, chunks[0]);

    let mode = Paragraph::new("📊 Dashboard")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center);
    frame.render_widget(mode, chunks[1]);

    let help = Paragraph::new("Press 1 for Timer")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Right);
    frame.render_widget(help, chunks[2]);
}

fn draw_main_content(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Stats cards
            Constraint::Length(12), // Bar chart
            Constraint::Min(5),     // Recent sessions
        ])
        .split(area);

    draw_stats_cards(frame, chunks[0], app);
    draw_weekly_chart(frame, chunks[1], app);
    draw_recent_sessions(frame, chunks[2], app);
}

fn format_duration(secs: u64) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

fn draw_stats_cards(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    // Today
    let today_secs = app.session_history.today_focus_secs();
    let _today_count = app.session_history.today_session_count();
    let (completed, goal) = app.daily_goal_progress();
    let goal_status = if completed >= goal as usize { "✓" } else { "" };
    
    let today_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Today ");
    let today_text = format!("{}\n{}/{} 🎯{}", format_duration(today_secs), completed, goal, goal_status);
    let today = Paragraph::new(today_text)
        .style(Style::default().fg(Color::White).bold())
        .alignment(Alignment::Center)
        .block(today_block);
    frame.render_widget(today, chunks[0]);

    // This Week
    let week_secs = app.session_history.week_focus_secs();
    let week_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title(" This Week ");
    let week_text = format_duration(week_secs);
    let week = Paragraph::new(week_text)
        .style(Style::default().fg(Color::White).bold())
        .alignment(Alignment::Center)
        .block(week_block);
    frame.render_widget(week, chunks[1]);

    // Streak
    let streak_color = if app.session_history.current_streak > 0 {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let streak_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(streak_color))
        .title(" Streak ");
    let streak_text = format!(
        "🔥 {} days\n⭐ Best: {}",
        app.session_history.current_streak,
        app.session_history.longest_streak
    );
    let streak = Paragraph::new(streak_text)
        .style(Style::default().fg(Color::White).bold())
        .alignment(Alignment::Center)
        .block(streak_block);
    frame.render_widget(streak, chunks[2]);

    // All Time
    let total_secs = app.session_history.total_focus_secs();
    let total_sessions = app.session_history.sessions.len();
    let total_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .title(" All Time ");
    let total_text = format!("{}\n{} sessions", format_duration(total_secs), total_sessions);
    let total = Paragraph::new(total_text)
        .style(Style::default().fg(Color::White).bold())
        .alignment(Alignment::Center)
        .block(total_block);
    frame.render_widget(total, chunks[3]);
}

fn draw_weekly_chart(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .title(" Weekly Activity ");

    let data = app.session_history.last_7_days_focus();
    let max_mins = data.iter().map(|(_, s)| s / 60).max().unwrap_or(1).max(1);

    let bars: Vec<Bar> = data
        .iter()
        .map(|(day, secs)| {
            let mins = secs / 60;
            Bar::default()
                .value(mins)
                .label(Line::from(day.clone()))
                .style(Style::default().fg(Color::Cyan))
        })
        .collect();

    let chart = BarChart::default()
        .block(block)
        .bar_width(5)
        .bar_gap(2)
        .group_gap(0)
        .bar_style(Style::default().fg(Color::Cyan))
        .value_style(Style::default().fg(Color::White).bold())
        .data(BarGroup::default().bars(&bars))
        .max(max_mins);

    frame.render_widget(chart, area);
}

fn draw_recent_sessions(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Recent Sessions ");

    let recent = app.session_history.recent_sessions(10);

    if recent.is_empty() {
        let empty = Paragraph::new("No sessions yet. Start a timer!")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(block);
        frame.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec!["Time", "Type", "Dur", "Task", "Note"])
        .style(Style::default().fg(Color::Yellow).bold())
        .bottom_margin(1);

    let rows: Vec<Row> = recent
        .iter()
        .map(|s| {
            let time = s.timestamp.format("%m/%d %H:%M").to_string();
            let session_type = match s.session_type.as_str() {
                "work" => "🍅",
                "short_break" => "☕",
                "long_break" => "🌴",
                _ => "?",
            };
            let duration = format!("{}m", s.duration_secs / 60);
            let task = s.task_name.clone().unwrap_or_else(|| "-".to_string());
            let note = s.note.clone().unwrap_or_else(|| String::new());
            // Truncate note to fit
            let note_display = if note.len() > 20 {
                format!("{}…", &note[..19])
            } else {
                note
            };

            Row::new(vec![
                Cell::from(time),
                Cell::from(session_type),
                Cell::from(duration),
                Cell::from(task),
                Cell::from(Span::styled(note_display, Style::default().fg(Color::DarkGray))),
            ])
            .style(Style::default().fg(Color::White))
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Length(15),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(block);

    frame.render_widget(table, area);
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let hints = Line::from(vec![
        Span::styled("[1]", Style::default().fg(Color::Cyan).bold()),
        Span::raw(" Timer  "),
        Span::styled("[2]", Style::default().fg(Color::Yellow).bold()),
        Span::raw(" Dashboard  "),
        Span::styled("[3]", Style::default().fg(Color::Magenta).bold()),
        Span::raw(" Settings  "),
        Span::styled("[q]", Style::default().fg(Color::Red).bold()),
        Span::raw(" Quit"),
    ]);

    let footer = Paragraph::new(hints)
        .alignment(Alignment::Center);

    frame.render_widget(footer, area);
}
