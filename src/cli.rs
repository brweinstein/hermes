use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hermes")]
#[command(about = "A terminal email client", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Send an email (non-interactive)
    Send {
        #[arg(short, long)]
        to: String,

        #[arg(short, long)]
        subject: String,

        #[arg(short, long)]
        body: String,
    },

    /// Delete an email by subject
    Delete {
        #[arg(short, long)]
        subject: String,
    },

    /// Sync emails with the server
    Sync,
}
