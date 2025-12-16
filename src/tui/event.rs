use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::io::Result;
use std::time::Duration;

pub fn poll_event() -> Result<Option<Event>> {
    if event::poll(Duration::from_millis(250))? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}

pub fn handle_key(key: crossterm::event::KeyEvent, app: &mut crate::app::App) {
    use crate::app::Mode;

    match app.mode {
        Mode::Normal => match key.code {
            // Quit
            KeyCode::Char('q') => app.quit(),
            // Vim movement
            KeyCode::Up | KeyCode::Char('k') => app.on_up(),
            KeyCode::Down | KeyCode::Char('j') => app.on_down(),
            KeyCode::Char('g') => app.selected = 0,
            KeyCode::Char('G') => {
                if !app.inbox.is_empty() {
                    app.selected = app.inbox.len() - 1;
                }
            }
            // Enter opens selected email
            KeyCode::Enter => app.open_selected(),
            // Open command mode with ':'
            KeyCode::Char(':') => app.start_command(),
            // Delete selected email with 'd'
            KeyCode::Char('d') => app.show_delete_confirm(),
            // New email with 'n'
            KeyCode::Char('n') => app.start_compose(),
            _ => {}
        },
        Mode::Command => match key.code {
            KeyCode::Esc => app.close_overlay(),
            KeyCode::Enter => app.submit_command(),
            KeyCode::Backspace => app.pop_command_char(),
            KeyCode::Char(ch) => {
                // ignore Ctrl chars, accept regular input
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT {
                    app.push_command_char(ch);
                }
            }
            _ => {}
        },
        Mode::Help => match key.code {
            KeyCode::Esc | KeyCode::Char('q') => app.close_overlay(),
            _ => {}
        },
        Mode::Viewing => match key.code {
            // Close viewing with q or Esc or Enter
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => app.close_overlay(),
            // j/k scroll within the email content
            KeyCode::Up | KeyCode::Char('k') => app.view_scroll_up(),
            KeyCode::Down | KeyCode::Char('j') => {
                let total_lines = if let Some(email) = app.inbox.get(app.selected) {
                    email.body.lines().count()
                } else {
                    0
                };
                app.view_scroll_down(total_lines, app.view_height);
            }
            _ => {}
        },
        Mode::Compose => {
            use crate::app::{ComposeField, ComposeMode};
            match app.compose_mode {
                ComposeMode::Normal => match key.code {
                    // Navigation with j/k - between fields or within body
                    KeyCode::Char('j') | KeyCode::Down => {
                        if matches!(app.compose_field, ComposeField::Body) {
                            app.compose_move_down();
                        } else {
                            app.compose_next_field();
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if matches!(app.compose_field, ComposeField::Body) {
                            app.compose_move_up();
                        } else {
                            app.compose_prev_field();
                        }
                    }
                    // Horizontal movement
                    KeyCode::Char('h') | KeyCode::Left => app.compose_move_left(),
                    KeyCode::Char('l') | KeyCode::Right => app.compose_move_right(),
                    KeyCode::Char('w') => app.compose_move_word_forward(),
                    KeyCode::Char('b') => app.compose_move_word_backward(),
                    KeyCode::Char('0') => app.compose_move_line_start(),
                    KeyCode::Char('$') => app.compose_move_line_end(),
                    // Enter insert mode variations
                    KeyCode::Char('i') => app.compose_enter_insert(),
                    KeyCode::Char('a') => app.compose_append(),
                    KeyCode::Char('A') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        app.compose_append_end()
                    }
                    KeyCode::Char('I') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        app.compose_insert_start()
                    }
                    KeyCode::Char('o') => app.compose_open_below(),
                    KeyCode::Char('O') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        app.compose_open_above()
                    }
                    // Deletion
                    KeyCode::Char('x') => app.compose_delete_char(),
                    KeyCode::Char('d') => {
                        // Simple dd for delete line
                        app.compose_delete_line();
                    }
                    // Indentation
                    KeyCode::Char('>') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        app.compose_indent_right();
                    }
                    KeyCode::Char('<') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        app.compose_indent_left();
                    }
                    // Visual mode
                    KeyCode::Char('v') => app.compose_enter_visual(),
                    // Send with :wq or ZZ
                    KeyCode::Char(':') => {
                        app.needs_refresh = true;
                        app.close_overlay();
                    }
                    KeyCode::Char('Z') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                        app.needs_refresh = true;
                        app.close_overlay();
                    }
                    // Quit without sending
                    KeyCode::Esc | KeyCode::Char('q') => app.close_overlay(),
                    _ => {}
                },
                ComposeMode::Insert => match key.code {
                    KeyCode::Esc => app.compose_exit_insert(),
                    KeyCode::Backspace => app.compose_pop_char(),
                    KeyCode::Enter => app.compose_insert_newline(),
                    KeyCode::Left => app.compose_move_left(),
                    KeyCode::Right => app.compose_move_right(),
                    KeyCode::Up => app.compose_move_up(),
                    KeyCode::Down => app.compose_move_down(),
                    KeyCode::Char(ch) => {
                        if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT {
                            app.compose_push_char(ch);
                        }
                    }
                    _ => {}
                },
                ComposeMode::Visual => match key.code {
                    // Movement in visual mode
                    KeyCode::Char('h') | KeyCode::Left => app.compose_move_left(),
                    KeyCode::Char('l') | KeyCode::Right => app.compose_move_right(),
                    KeyCode::Char('j') | KeyCode::Down => app.compose_move_down(),
                    KeyCode::Char('k') | KeyCode::Up => app.compose_move_up(),
                    KeyCode::Char('w') => app.compose_move_word_forward(),
                    KeyCode::Char('b') => app.compose_move_word_backward(),
                    KeyCode::Char('0') => app.compose_move_line_start(),
                    KeyCode::Char('$') => app.compose_move_line_end(),
                    // Delete selection
                    KeyCode::Char('d') | KeyCode::Char('x') => app.compose_delete_visual(),
                    // Exit visual mode
                    KeyCode::Esc => app.compose_exit_visual(),
                    _ => {}
                },
            }
        }
        Mode::DeleteConfirm => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_delete(),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.close_overlay(),
            _ => {}
        },
    }
}
