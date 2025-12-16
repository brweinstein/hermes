use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct EmailSummary {
    pub subject: String,
    pub from: String,
    pub body: String,
    pub file_path: Option<PathBuf>,
}

pub trait EmailBackend {
    fn fetch_inbox(&self) -> Result<Vec<EmailSummary>>;
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()>;
    fn delete_email(&self, email: &EmailSummary) -> Result<()>;
}

/// Simple file-backed backend
pub struct FileBackend {
    path: PathBuf,
    user_email: String,
}

impl FileBackend {
    pub fn new(path: impl Into<PathBuf>, user_email: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            user_email: user_email.into(),
        }
    }
}

impl EmailBackend for FileBackend {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        // If path is a directory, create a new file
        if self.path.is_dir() {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();
            let filename = format!("email_{}.txt", timestamp);
            let file_path = self.path.join(filename);
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(&file_path)?;

            writeln!(file, "FROM: {}", self.user_email)?;
            writeln!(file, "TO: {}", to)?;
            writeln!(file, "SUBJECT: {}", subject)?;
            writeln!(file, "BODY:")?;
            writeln!(file, "{body}")?;
        } else {
            // Legacy: append to file
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;

            writeln!(file, "FROM: {}", self.user_email)?;
            writeln!(file, "TO: {}", to)?;
            writeln!(file, "SUBJECT: {}", subject)?;
            writeln!(file, "BODY:")?;
            writeln!(file, "{body}")?;
            writeln!(file, "---")?;
        }

        Ok(())
    }

    fn delete_email(&self, email: &EmailSummary) -> Result<()> {
        if let Some(file_path) = &email.file_path {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    fn fetch_inbox(&self) -> Result<Vec<EmailSummary>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        // Support either a single file (legacy) or a directory of email files
        if self.path.is_dir() {
            let mut inbox = Vec::new();
            for entry in fs::read_dir(&self.path)? {
                let entry = entry?;
                let p = entry.path();
                if p.is_file() && p.extension().map(|e| e == "txt").unwrap_or(false) {
                    if let Ok(mut email) = parse_single_email_file(&p) {
                        email.file_path = Some(p);
                        inbox.push(email);
                    }
                }
            }
            return Ok(inbox);
        }

        // Legacy single-file with multiple emails separated by ---
        let file = fs::File::open(&self.path)?;
        let reader = BufReader::new(file);

        let mut inbox = Vec::new();
        let mut from = String::new();
        let mut subject = String::new();
        let mut body_lines: Vec<String> = Vec::new();
        let mut in_body = false;

        for line in reader.lines() {
            let line = line?;

            if let Some(rest) = line.strip_prefix("FROM: ") {
                from = rest.to_string();
                in_body = false;
                body_lines.clear();
            } else if let Some(rest) = line.strip_prefix("SUBJECT: ") {
                subject = rest.to_string();
                in_body = false;
            } else if line == "BODY:" {
                in_body = true;
                body_lines.clear();
            } else if line == "---" {
                // finalize one email entry
                inbox.push(EmailSummary {
                    from: from.clone(),
                    subject: subject.clone(),
                    body: body_lines.join("\n"),
                    file_path: None,
                });

                from.clear();
                subject.clear();
                body_lines.clear();
                in_body = false;
            } else if in_body {
                body_lines.push(line.to_string());
            }
        }

        // If file ended without trailing --- but we have content, push last email
        if !(from.is_empty() && subject.is_empty() && body_lines.is_empty()) {
            inbox.push(EmailSummary {
                from,
                subject,
                body: body_lines.join("\n"),
                file_path: None,
            });
        }

        Ok(inbox)
    }
}

fn parse_single_email_file(path: &PathBuf) -> Result<EmailSummary> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut from = String::new();
    let mut subject = String::new();
    let mut body_lines: Vec<String> = Vec::new();
    let mut in_body = false;

    for line in reader.lines() {
        let line = line?;
        if let Some(rest) = line.strip_prefix("FROM: ") {
            from = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("SUBJECT: ") {
            subject = rest.to_string();
        } else if line == "BODY:" {
            in_body = true;
        } else if in_body {
            body_lines.push(line.to_string());
        }
    }

    // Trim leading empty lines from body
    while body_lines
        .first()
        .map(|l| l.trim().is_empty())
        .unwrap_or(false)
    {
        body_lines.remove(0);
    }

    // Trim trailing empty lines from body
    while body_lines
        .last()
        .map(|l| l.trim().is_empty())
        .unwrap_or(false)
    {
        body_lines.pop();
    }

    Ok(EmailSummary {
        from,
        subject,
        body: body_lines.join("\n"),
        file_path: None,
    })
}

// Removed preview summarization; full body stored in EmailSummary.
