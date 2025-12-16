pub mod event;
pub mod ui;

use crate::app::App;
use crate::backend::EmailBackend;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    cursor::{SetCursorStyle, Show, Hide},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::stdout;

pub fn run(app: &mut App, backend: &impl EmailBackend) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Show, SetCursorStyle::SteadyBlock)?;

    let term_backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(term_backend)?;

    let mut prev_mode = app.mode;

    while !app.should_quit {
        // Show blinking block cursor only in compose mode
        match app.mode {
            crate::app::Mode::Compose => {
                execute!(terminal.backend_mut(), Show, SetCursorStyle::BlinkingBlock)?;
            }
            _ => {
                execute!(terminal.backend_mut(), Hide)?;
            }
        }
        
        terminal.draw(|f| ui::draw(f, app))?;

        if let Some(event) = event::poll_event()? {
            if let crossterm::event::Event::Key(key) = event {
                event::handle_key(key, app);

                // Handle delete confirmation
                if app.needs_refresh && prev_mode == crate::app::Mode::DeleteConfirm {
                    // Delete was confirmed - use the stored email
                    if let Some(email) = &app.email_to_delete {
                        backend.delete_email(email)?;
                    }
                    app.email_to_delete = None;
                    app.needs_refresh = false;
                }

                // Handle compose send
                if app.needs_refresh && prev_mode == crate::app::Mode::Compose {
                    // Send email
                    let (to, subject, body) = app.get_compose_data();
                    if !to.is_empty() && !subject.is_empty() {
                        backend.send_email(to, subject, body)?;
                        // Refresh inbox
                        app.inbox = backend.fetch_inbox()?;
                    }
                    app.needs_refresh = false;
                }

                prev_mode = app.mode;
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
