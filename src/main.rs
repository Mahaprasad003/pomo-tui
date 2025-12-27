mod app;
mod persistence;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::{io, panic, time::Duration};

fn main() -> Result<()> {
    // Set up panic hook to restore terminal on crash
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    // Initialize terminal
    let mut terminal = setup_terminal()?;

    // Create app and run
    let mut app = App::new();
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    restore_terminal()?;

    // Handle any error from the app loop
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    const TICK_RATE: Duration = Duration::from_millis(100);

    loop {
        // Draw UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Handle events with timeout
        if event::poll(TICK_RATE)? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events (not release)
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);
                }
            }
        }

        // Check if we should quit
        if app.should_quit {
            return Ok(());
        }

        // Update timer (will be implemented in Phase 2)
        app.tick();
    }
}
