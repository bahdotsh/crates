mod api;
mod app;
mod event;
mod ui;

use app::{App, AppResult};
use event::{Event, EventHandler};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> AppResult<()> {
    // Setup terminal
    let mut terminal = setup_terminal()?;

    // Create app state
    let mut app = App::new();

    // Initialize event handler
    let events = EventHandler::new(250);

    // Main loop
    while app.running {
        // Draw UI
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Handle events
        match events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => app.handle_key_event(key_event),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Cleanup and restore terminal
    restore_terminal(&mut terminal)?;
    Ok(())
}

fn setup_terminal() -> AppResult<Terminal<CrosstermBackend<io::Stdout>>> {
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> AppResult<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;

    Ok(())
}
