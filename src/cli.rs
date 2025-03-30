use clap::{Args, Parser, Subcommand, ValueEnum};

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "ratch-job")]
#[command(version, about = "ratch-job cli", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// env file path
    #[arg(short, long, default_value = "")]
    pub env_file: String,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    About,
}
