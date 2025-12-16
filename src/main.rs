mod app;
mod backend;
mod cli;
mod tui;

use backend::{EmailBackend, FileBackend};
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let backend = FileBackend::new("sample", "me@hermes.local");

    match cli.command {
        Some(Commands::Send { to, subject, body }) => {
            backend.send_email(&to, &subject, &body)?;
            println!("Email sent successfully");
        }
        Some(Commands::Delete { subject }) => {
            let inbox = backend.fetch_inbox()?;
            let email = inbox.iter().find(|e| e.subject == subject);
            if let Some(email) = email {
                backend.delete_email(email)?;
                println!("Email deleted: {}", subject);
            } else {
                println!("Email not found: {}", subject);
            }
        }
        Some(Commands::Sync) => {
            let inbox = backend.fetch_inbox()?;
            println!("Fetched {} emails", inbox.len());
        }
        None => {
            let inbox = backend.fetch_inbox()?;
            let mut app = app::App::new(inbox);
            tui::run(&mut app, &backend)?;
        }
    }

    Ok(())
}
