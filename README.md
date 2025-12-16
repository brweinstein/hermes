# Hermes

A terminal-based email client with vim-style keybindings built in Rust.

## Features

- **Inbox Management**: View and navigate emails 
- **Email Viewing**: Read email content with scrollable body text
- **Compose Emails**: Create new emails with full vim-style editing
- **Delete Emails**: Remove emails with confirmation prompt
- **File-based Backend**: Store emails as text files with FROM:/SUBJECT:/BODY: format
    - Before backend API

## Usage

```bash
# View inbox
cargo run

# Send email via CLI
cargo run -- send <to> <subject> <body>

# Delete email via CLI
cargo run -- delete <file_path>
```

## Keybindings

### Inbox
- `j/k` or `↑/↓` - Navigate emails
- `Enter` - View selected email
- `n` - Compose new email
- `d` - Delete selected email
- `q` - Quit

### Email View
- `j/k` - Scroll content
- `q/Esc/Enter` - Return to inbox

### Compose (Normal Mode)
- `j/k` - Navigate fields / move within body
- `h/l` - Move cursor left/right
- `w/b` - Move by word
- `0/$` - Jump to line start/end
- `i` - Enter insert mode
- `a` - Append (insert after cursor)
- `o/O` - Open new line below/above
- `dd` - Delete line
- `x` - Delete character
- `>>/<<` - Indent/unindent line
- `:` - Send email
- `q/Esc` - Cancel

### Compose (Insert Mode)
- `Esc` - Return to normal mode
- `Enter` - New line
- `Backspace` - Delete character

## Sample Data

Used for before backend is made

Place email files in the `sample/` directory with the format:

```
FROM:sender@example.com
SUBJECT:Email Subject
BODY:
Email body content here.
Multiple lines supported.
```
