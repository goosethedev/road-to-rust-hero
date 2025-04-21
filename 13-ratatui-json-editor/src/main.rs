mod app;
mod ui;

use app::App;
use ui::render_screen;

use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
    Terminal,
};
use std::error::Error;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    // Disables default terminal functionality
    enable_raw_mode()?;

    // Print UI to stderr to not interfere with the JSON output at stdout
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    // Setup the terminal on stderr
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // Setup the App and run in loop
    let mut app = App::default();
    let res = run_app(&mut terminal, &mut app);

    // When app ends, restore the terminal original state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Check if JSON can be generated from the app
    match res {
        Ok(do_print) if do_print => app.print_json()?,
        Err(e) => println!("{e:?}"),
        _ => {}
    }

    Ok(())
}

// Executor function is backend agnostic :D
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        // Draw the UI for the current screen as a single frame
        terminal.draw(|f| render_screen(f, app))?;

        // Handle keyboard events
        if let Event::Key(key) = event::read()? {
            // Skip Windows-only "Release" events
            if key.kind == KeyEventKind::Release {
                continue;
            };
            // If exit is requested, end loop
            if let Some(b) = app.handle_key_event(key) {
                return Ok(b);
            }
        }
    }
}
