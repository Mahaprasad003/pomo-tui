mod dashboard_view;
mod settings_view;
mod timer_view;

use crate::app::{App, CurrentView};
use ratatui::Frame;

/// Main draw function that renders the current view
pub fn draw(frame: &mut Frame, app: &App) {
    match app.current_view {
        CurrentView::Timer => timer_view::draw(frame, app),
        CurrentView::Dashboard => dashboard_view::draw(frame, app),
        CurrentView::Settings => settings_view::draw(frame, app),
    }
}
