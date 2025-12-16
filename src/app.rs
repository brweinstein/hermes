use crate::backend::EmailSummary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Command,
    Help,
    Viewing,
    Compose,
    DeleteConfirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeField {
    To,
    Subject,
    Body,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeMode {
    Normal,
    Insert,
    Visual,
}

#[derive(Debug)]
pub struct App {
    pub inbox: Vec<EmailSummary>,
    pub selected: usize,
    pub should_quit: bool,
    pub mode: Mode,
    pub command_buf: String,
    pub view_offset: usize,
    pub view_height: usize,
    pub compose_to: String,
    pub compose_subject: String,
    pub compose_body: String,
    pub compose_field: ComposeField,
    pub compose_mode: ComposeMode,
    pub compose_cursor: usize,
    pub compose_line: usize,
    pub compose_col: usize,
    pub compose_visual_start: Option<usize>,
    pub needs_refresh: bool,
    pub email_to_delete: Option<EmailSummary>,
}

impl App {
    pub fn new(inbox: Vec<EmailSummary>) -> Self {
        Self {
            inbox,
            selected: 0,
            should_quit: false,
            mode: Mode::Normal,
            command_buf: String::new(),
            view_offset: 0,
            view_height: 0,
            compose_to: String::new(),
            compose_subject: String::new(),
            compose_body: String::new(),
            compose_field: ComposeField::To,
            compose_mode: ComposeMode::Normal,
            compose_cursor: 0,
            compose_line: 0,
            compose_col: 0,
            compose_visual_start: None,
            needs_refresh: false,
            email_to_delete: None,
        }
    }

    pub fn on_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn on_down(&mut self) {
        if self.selected + 1 < self.inbox.len() {
            self.selected += 1;
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn open_selected(&mut self) {
        if self.selected < self.inbox.len() {
            self.mode = Mode::Viewing;
            self.view_offset = 0;
        }
    }

    pub fn close_overlay(&mut self) {
        self.mode = Mode::Normal;
        self.command_buf.clear();
        self.view_offset = 0;
        self.email_to_delete = None;
    }

    pub fn view_scroll_up(&mut self) {
        if self.view_offset > 0 {
            self.view_offset -= 1;
        }
    }

    pub fn view_scroll_down(&mut self, total_lines: usize, visible_height: usize) {
        if total_lines == 0 || visible_height == 0 {
            return;
        }
        // Calculate max offset: when total lines fit in view, don't scroll
        // Otherwise, allow scrolling until last line is at bottom of view
        let max_offset = total_lines.saturating_sub(visible_height);
        if self.view_offset < max_offset {
            self.view_offset += 1;
        }
    }

    pub fn start_command(&mut self) {
        self.mode = Mode::Command;
        self.command_buf.clear();
    }

    pub fn push_command_char(&mut self, ch: char) {
        self.command_buf.push(ch);
    }

    pub fn pop_command_char(&mut self) {
        self.command_buf.pop();
    }

    pub fn submit_command(&mut self) {
        let cmd = self.command_buf.trim();
        match cmd {
            "help" => {
                self.mode = Mode::Help;
            }
            _ => {
                // Unknown command: just return to normal
                self.mode = Mode::Normal;
            }
        }
        self.command_buf.clear();
    }

    pub fn show_delete_confirm(&mut self) {
        if self.selected < self.inbox.len() {
            self.email_to_delete = Some(self.inbox[self.selected].clone());
            self.mode = Mode::DeleteConfirm;
        }
    }

    pub fn confirm_delete(&mut self) {
        if self.selected < self.inbox.len() {
            self.inbox.remove(self.selected);
            if self.selected >= self.inbox.len() && self.selected > 0 {
                self.selected -= 1;
            }
            self.needs_refresh = true;
        }
        self.mode = Mode::Normal;
    }

    pub fn start_compose(&mut self) {
        self.mode = Mode::Compose;
        self.compose_to.clear();
        self.compose_subject.clear();
        self.compose_body.clear();
        self.compose_field = ComposeField::To;
        self.compose_mode = ComposeMode::Normal;
        self.compose_cursor = 0;
        self.compose_line = 0;
        self.compose_col = 0;
        self.compose_visual_start = None;
    }

    pub fn compose_next_field(&mut self) {
        if matches!(self.compose_mode, ComposeMode::Normal | ComposeMode::Visual) {
            self.compose_field = match self.compose_field {
                ComposeField::To => ComposeField::Subject,
                ComposeField::Subject => ComposeField::Body,
                ComposeField::Body => ComposeField::To,
            };
            self.compose_cursor = 0;
            self.clamp_cursor();
        }
    }

    pub fn compose_prev_field(&mut self) {
        if matches!(self.compose_mode, ComposeMode::Normal | ComposeMode::Visual) {
            self.compose_field = match self.compose_field {
                ComposeField::To => ComposeField::Body,
                ComposeField::Subject => ComposeField::To,
                ComposeField::Body => ComposeField::Subject,
            };
            self.compose_cursor = 0;
            self.clamp_cursor();
        }
    }

    pub fn compose_enter_insert(&mut self) {
        self.compose_mode = ComposeMode::Insert;
        self.compose_visual_start = None;
    }

    pub fn compose_enter_visual(&mut self) {
        self.compose_mode = ComposeMode::Visual;
        self.compose_visual_start = Some(self.compose_cursor);
    }

    pub fn compose_exit_insert(&mut self) {
        self.compose_mode = ComposeMode::Normal;
    }

    pub fn compose_exit_visual(&mut self) {
        self.compose_mode = ComposeMode::Normal;
        self.compose_visual_start = None;
    }

    fn get_current_field_text(&self) -> &String {
        match self.compose_field {
            ComposeField::To => &self.compose_to,
            ComposeField::Subject => &self.compose_subject,
            ComposeField::Body => &self.compose_body,
        }
    }

    fn get_current_field_text_mut(&mut self) -> &mut String {
        match self.compose_field {
            ComposeField::To => &mut self.compose_to,
            ComposeField::Subject => &mut self.compose_subject,
            ComposeField::Body => &mut self.compose_body,
        }
    }

    fn clamp_cursor(&mut self) {
        let text = self.get_current_field_text();
        let len = text.len();
        if self.compose_cursor > len {
            self.compose_cursor = len;
        }
        // Ensure cursor is at a valid UTF-8 boundary
        let text_copy = self.get_current_field_text().clone();
        while self.compose_cursor > 0 && !text_copy.is_char_boundary(self.compose_cursor) {
            self.compose_cursor -= 1;
        }

        // Update line and col from cursor position for body field
        if matches!(self.compose_field, ComposeField::Body) {
            self.update_line_col_from_cursor();
        }
    }

    fn update_line_col_from_cursor(&mut self) {
        let text = &self.compose_body;
        let mut pos = 0;
        self.compose_line = 0;
        self.compose_col = 0;

        for line in text.lines() {
            let line_len = line.len();
            if pos + line_len >= self.compose_cursor {
                self.compose_col = self.compose_cursor - pos;
                break;
            }
            pos += line_len + 1; // +1 for newline
            self.compose_line += 1;
        }
    }

    fn update_cursor_from_line_col(&mut self) {
        if !matches!(self.compose_field, ComposeField::Body) {
            return;
        }

        let lines: Vec<&str> = self.compose_body.lines().collect();
        if lines.is_empty() {
            self.compose_cursor = 0;
            return;
        }

        // Clamp line
        if self.compose_line >= lines.len() {
            self.compose_line = lines.len().saturating_sub(1);
        }

        // Calculate cursor position
        let mut pos = 0;
        for i in 0..self.compose_line {
            pos += lines[i].len() + 1; // +1 for newline
        }

        // Clamp column
        let line = lines[self.compose_line];
        if self.compose_col > line.len() {
            self.compose_col = line.len();
        }

        self.compose_cursor = pos + self.compose_col;
    }

    pub fn compose_move_up(&mut self) {
        if matches!(self.compose_field, ComposeField::Body) {
            if self.compose_line > 0 {
                self.compose_line -= 1;
                self.update_cursor_from_line_col();
            } else {
                // At top of body, move to previous field
                self.compose_prev_field();
            }
        }
    }

    pub fn compose_move_down(&mut self) {
        if matches!(self.compose_field, ComposeField::Body) {
            let lines: Vec<&str> = self.compose_body.lines().collect();
            if self.compose_line + 1 < lines.len() {
                self.compose_line += 1;
                self.update_cursor_from_line_col();
            }
        }
    }

    pub fn compose_move_left(&mut self) {
        if matches!(self.compose_field, ComposeField::Body) {
            if self.compose_col > 0 {
                self.compose_col -= 1;
                self.update_cursor_from_line_col();
            } else if self.compose_line > 0 {
                // Move to end of previous line
                self.compose_line -= 1;
                let lines: Vec<&str> = self.compose_body.lines().collect();
                self.compose_col = lines.get(self.compose_line).map(|l| l.len()).unwrap_or(0);
                self.update_cursor_from_line_col();
            }
        } else if self.compose_cursor > 0 {
            self.compose_cursor -= 1;
            self.clamp_cursor();
        }
    }

    pub fn compose_move_right(&mut self) {
        if matches!(self.compose_field, ComposeField::Body) {
            let lines: Vec<&str> = self.compose_body.lines().collect();
            if let Some(line) = lines.get(self.compose_line) {
                if self.compose_col < line.len() {
                    self.compose_col += 1;
                    self.update_cursor_from_line_col();
                } else if self.compose_line + 1 < lines.len() {
                    // Move to start of next line
                    self.compose_line += 1;
                    self.compose_col = 0;
                    self.update_cursor_from_line_col();
                }
            }
        } else {
            let text = self.get_current_field_text();
            if self.compose_cursor < text.len() {
                self.compose_cursor += 1;
                self.clamp_cursor();
            }
        }
    }

    pub fn compose_move_word_forward(&mut self) {
        let text = self.get_current_field_text();
        let chars: Vec<char> = text.chars().collect();
        let mut char_pos = text[..self.compose_cursor].chars().count();
        while char_pos < chars.len() && !chars[char_pos].is_whitespace() {
            char_pos += 1;
        }
        while char_pos < chars.len() && chars[char_pos].is_whitespace() {
            char_pos += 1;
        }
        // Convert char position back to byte position
        self.compose_cursor = text
            .char_indices()
            .nth(char_pos)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
    }

    pub fn compose_move_word_backward(&mut self) {
        if self.compose_cursor == 0 {
            return;
        }
        let text = self.get_current_field_text();
        let chars: Vec<char> = text.chars().collect();
        let mut char_pos = text[..self.compose_cursor].chars().count();
        if char_pos > 0 {
            char_pos -= 1;
        }
        while char_pos > 0 && chars[char_pos].is_whitespace() {
            char_pos -= 1;
        }
        while char_pos > 0 && !chars[char_pos - 1].is_whitespace() {
            char_pos -= 1;
        }
        // Convert char position back to byte position
        self.compose_cursor = text
            .char_indices()
            .nth(char_pos)
            .map(|(i, _)| i)
            .unwrap_or(0);
    }

    pub fn compose_move_line_start(&mut self) {
        if matches!(self.compose_field, ComposeField::Body) {
            self.compose_col = 0;
            self.update_cursor_from_line_col();
        } else {
            self.compose_cursor = 0;
        }
    }

    pub fn compose_move_line_end(&mut self) {
        if matches!(self.compose_field, ComposeField::Body) {
            let lines: Vec<&str> = self.compose_body.lines().collect();
            if let Some(line) = lines.get(self.compose_line) {
                self.compose_col = line.len();
                self.update_cursor_from_line_col();
            }
        } else {
            let text = self.get_current_field_text();
            self.compose_cursor = text.len();
        }
    }

    pub fn compose_delete_char(&mut self) {
        let cursor = self.compose_cursor;
        let text = self.get_current_field_text_mut();
        if cursor < text.len() {
            text.remove(cursor);
        }
    }

    pub fn compose_delete_line(&mut self) {
        let text = self.get_current_field_text_mut();
        text.clear();
        self.compose_cursor = 0;
    }

    pub fn compose_indent_right(&mut self) {
        if matches!(self.compose_field, ComposeField::Body) {
            let text = &mut self.compose_body;
            text.insert_str(0, "  ");
            self.compose_cursor += 2;
        }
    }

    pub fn compose_indent_left(&mut self) {
        if matches!(self.compose_field, ComposeField::Body) {
            let text = &mut self.compose_body;
            if text.starts_with("  ") {
                text.drain(0..2);
                self.compose_cursor = self.compose_cursor.saturating_sub(2);
            }
        }
    }

    pub fn compose_append(&mut self) {
        self.compose_move_right();
        self.compose_enter_insert();
    }

    pub fn compose_append_end(&mut self) {
        self.compose_move_line_end();
        self.compose_enter_insert();
    }

    pub fn compose_insert_start(&mut self) {
        self.compose_move_line_start();
        self.compose_enter_insert();
    }

    pub fn compose_open_below(&mut self) {
        if matches!(self.compose_field, ComposeField::Body) {
            self.compose_move_line_end();
            self.compose_body.push('\n');
            self.compose_cursor = self.compose_body.len();
            self.compose_line += 1;
            self.compose_col = 0;
            self.compose_enter_insert();
        }
    }

    pub fn compose_open_above(&mut self) {
        if matches!(self.compose_field, ComposeField::Body) {
            self.compose_move_line_start();
            self.compose_body.insert_str(self.compose_cursor, "\n");
            // Stay on the new line above
            self.compose_col = 0;
            self.compose_enter_insert();
        }
    }

    pub fn compose_insert_newline(&mut self) {
        if matches!(self.compose_field, ComposeField::Body) {
            self.compose_push_char('\n');
        }
    }

    pub fn compose_delete_visual(&mut self) {
        if let Some(start) = self.compose_visual_start {
            let cursor = self.compose_cursor;
            let (begin, end) = if start < cursor {
                (start, cursor)
            } else {
                (cursor, start)
            };
            let text = self.get_current_field_text_mut();
            text.drain(begin..end.min(text.len()));
            self.compose_cursor = begin;
        }
        self.compose_exit_visual();
    }

    pub fn compose_push_char(&mut self, ch: char) {
        let cursor = self.compose_cursor;
        let text = self.get_current_field_text_mut();
        if cursor <= text.len() && text.is_char_boundary(cursor) {
            text.insert(cursor, ch);
            self.compose_cursor += ch.len_utf8();
            if matches!(self.compose_field, ComposeField::Body) {
                if ch == '\n' {
                    self.compose_line += 1;
                    self.compose_col = 0;
                } else {
                    self.compose_col += 1;
                }
            }
        }
    }

    pub fn compose_pop_char(&mut self) {
        if self.compose_cursor > 0 {
            self.compose_cursor -= 1;
            let cursor = self.compose_cursor;
            let text = self.get_current_field_text_mut();
            let removed_char = text.remove(cursor);
            if matches!(self.compose_field, ComposeField::Body) {
                if removed_char == '\n' {
                    if self.compose_line > 0 {
                        self.compose_line -= 1;
                        let lines: Vec<&str> = self.compose_body.lines().collect();
                        self.compose_col =
                            lines.get(self.compose_line).map(|l| l.len()).unwrap_or(0);
                    }
                } else if self.compose_col > 0 {
                    self.compose_col -= 1;
                }
            }
        }
    }

    pub fn get_compose_data(&self) -> (&str, &str, &str) {
        (&self.compose_to, &self.compose_subject, &self.compose_body)
    }
}
