use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    use crate::app::Mode;
    let area = f.size();

    let mut cursor_pos: Option<(u16, u16)> = None;

    match app.mode {
        Mode::Normal | Mode::Command => {
            // Base inbox list
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0)].as_ref())
                .split(area);

            let items: Vec<ListItem> = app
                .inbox
                .iter()
                .map(|email| ListItem::new(format!("{} — {}", email.from, email.subject)))
                .collect();

            let list = List::new(items)
                .block(Block::default().title("Inbox").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[0], &mut app_state(app));

            if app.mode == Mode::Command {
                // Draw command line overlay at bottom like Vim
                let cmd_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
                    .split(area);
                let cmd_text = format!(":{}", app.command_buf);
                let cmd_block = Block::default().borders(Borders::TOP).title("Command");
                let cmd_list = List::new(vec![ListItem::new(cmd_text)]).block(cmd_block);
                f.render_widget(cmd_list, cmd_chunks[1]);
            }
        }
        Mode::Help => {
            // Centered help box
            let help_area = centered_rect(60, 60, area);
            let help_items = vec![
                ListItem::new("Keybinds (Vim style):"),
                ListItem::new("  j / Down   — move down"),
                ListItem::new("  k / Up     — move up"),
                ListItem::new("  g          — go to top"),
                ListItem::new("  G          — go to bottom"),
                ListItem::new("  Enter      — open selected email"),
                ListItem::new("  d          — delete selected email"),
                ListItem::new("  n          — compose new email"),
                ListItem::new("  :help      — show this help"),
                ListItem::new("  Esc/q      — close overlay / quit help"),
                ListItem::new("  q          — quit app"),
            ];

            let help = List::new(help_items)
                .block(Block::default().title("Help").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));
            f.render_widget(help, help_area);
        }
        Mode::Viewing => {
            // Show selected email summary in a centered box
            let view_area = centered_rect(70, 50, area);

            // Calculate visible height for body content (minus borders and header lines)
            // Subtract 2 for borders, 3 for header lines (From, Subject, blank line)
            let visible_body_height = view_area.height.saturating_sub(5) as usize;
            app.view_height = visible_body_height;

            let mut lines = Vec::new();
            if let Some(email) = app.inbox.get(app.selected) {
                // Header lines always visible
                lines.push(ListItem::new(format!("From: {}", email.from)));
                lines.push(ListItem::new(format!("Subject: {}", email.subject)));
                lines.push(ListItem::new(""));

                if email.body.is_empty() {
                    lines.push(ListItem::new("(No body)"));
                } else {
                    // Apply view_offset only to body content
                    for (idx, line) in email.body.lines().enumerate() {
                        if idx >= app.view_offset {
                            lines.push(ListItem::new(line.to_string()));
                        }
                    }
                }
            } else {
                lines.push(ListItem::new("No email selected"));
            }
            let view =
                List::new(lines).block(Block::default().title("Email").borders(Borders::ALL));
            f.render_widget(view, view_area);

            // Render the close hint just below the bordered box, outside of it
            let hint_area = line_below(area, view_area);
            if hint_area.height > 0 {
                let hint =
                    List::new(vec![ListItem::new("[Esc/q/Enter] Close")]).block(Block::default());
                f.render_widget(hint, hint_area);
            }
        }
        Mode::Compose => {
            // Show compose form in centered box
            let compose_area = centered_rect(80, 60, area);
            use crate::app::{ComposeField, ComposeMode};

            let mode_indicator = match app.compose_mode {
                ComposeMode::Normal => "-- NORMAL --",
                ComposeMode::Insert => "-- INSERT --",
                ComposeMode::Visual => "-- VISUAL --",
            };

            let mut lines = Vec::new();
            lines.push(ListItem::new(format!("Compose Email {}", mode_indicator)));
            lines.push(ListItem::new(""));

            // To field
            let to_marker = if matches!(app.compose_field, ComposeField::To) {
                ">"
            } else {
                " "
            };
            lines.push(ListItem::new(format!("{}To: {}", to_marker, &app.compose_to)));
            
            if matches!(app.compose_field, ComposeField::To) {
                let char_idx = app.compose_to[..app.compose_cursor.min(app.compose_to.len())].chars().count();
                cursor_pos = Some((
                    compose_area.x + 1 + 1 + 4 + char_idx as u16, // border + marker + "To: " + offset
                    compose_area.y + 1 + 2, // border + title line + blank line + this line
                ));
            }

            // Subject field
            let subj_marker = if matches!(app.compose_field, ComposeField::Subject) {
                ">"
            } else {
                " "
            };
            lines.push(ListItem::new(format!(
                "{}Subject: {}",
                subj_marker, &app.compose_subject
            )));
            
            if matches!(app.compose_field, ComposeField::Subject) {
                let char_idx = app.compose_subject[..app.compose_cursor.min(app.compose_subject.len())].chars().count();
                cursor_pos = Some((
                    compose_area.x + 1 + 1 + 9 + char_idx as u16, // border + marker + "Subject: " + offset
                    compose_area.y + 1 + 3, // border + title + blank + to + this line
                ));
            }

            lines.push(ListItem::new(""));

            // Body field
            let body_marker = if matches!(app.compose_field, ComposeField::Body) {
                ">"
            } else {
                " "
            };
            lines.push(ListItem::new(format!("{}Body:", body_marker)));

            let body_lines: Vec<&str> = app.compose_body.lines().collect();
            for line in &body_lines {
                lines.push(ListItem::new(format!("  {}", line)));
            }
            if body_lines.is_empty() {
                lines.push(ListItem::new("  "));
            }
            
            if matches!(app.compose_field, ComposeField::Body) {
                cursor_pos = Some((
                    compose_area.x + 1 + 2 + app.compose_col as u16, // border + "  " indent + column
                    compose_area.y + 1 + 6 + app.compose_line as u16, // border + title + blank + to + subject + blank + "Body:" + line
                ));
            }

            let compose =
                List::new(lines).block(Block::default().title("New Email").borders(Borders::ALL));
            f.render_widget(compose, compose_area);

            let hint_area = line_below(area, compose_area);
            if hint_area.height > 0 {
                let hint_text = match app.compose_mode {
                    ComposeMode::Normal => {
                        "[j/k] Navigate  [i/a/o] Insert  [v] Visual  [x/dd] Delete  [>/<] Indent  [:] Send"
                    }
                    ComposeMode::Insert => "[Esc] Normal  [h/j/k/l] Move",
                    ComposeMode::Visual => "[h/l] Move  [d/x] Delete  [Esc] Exit",
                };
                let hint = List::new(vec![ListItem::new(hint_text)]).block(Block::default());
                f.render_widget(hint, hint_area);
            }
        }
        Mode::DeleteConfirm => {
            // Show delete confirmation in centered box
            let confirm_area = centered_rect(50, 30, area);

            let mut lines = Vec::new();
            if let Some(email) = app.inbox.get(app.selected) {
                lines.push(ListItem::new("Delete this email?"));
                lines.push(ListItem::new(""));
                lines.push(ListItem::new(format!("From: {}", email.from)));
                lines.push(ListItem::new(format!("Subject: {}", email.subject)));
                lines.push(ListItem::new(""));
                lines.push(ListItem::new("Press Y to confirm, N or Esc to cancel"));
            } else {
                lines.push(ListItem::new("No email selected"));
            }

            let confirm = List::new(lines).block(
                Block::default()
                    .title("Confirm Delete")
                    .borders(Borders::ALL),
            );
            f.render_widget(confirm, confirm_area);
        }
    }
    
    // Set hardware cursor position if in compose mode
    if let Some((x, y)) = cursor_pos {
        f.set_cursor(x, y);
    }
}

// Helper to keep ListState creation clean
fn app_state(app: &App) -> ratatui::widgets::ListState {
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.selected));
    state
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1]);

    horizontal[1]
}

fn line_below(full: ratatui::layout::Rect, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let y = r.y.saturating_add(r.height);
    let max_y = full.y.saturating_add(full.height);
    if y >= max_y {
        return ratatui::layout::Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        };
    }
    let width = r.width.min(full.width.saturating_sub(r.x));
    ratatui::layout::Rect {
        x: r.x,
        y,
        width,
        height: 1,
    }
}
