//! codequality-rs library

pub mod analyzer;
pub mod reporters;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "codequality", version, about)]
pub struct Cli {
    pub path: PathBuf,
    #[arg(short, long, value_enum, default_value_t = Format::Markdown)]
    pub format: Format,
    #[arg(long, value_delimiter = ',', default_value = "py")]
    pub extensions: Vec<String>,
    #[arg(long)]
    pub no_cache: bool,
    #[arg(long, value_delimiter = ',')]
    pub only: Vec<String>,
    #[arg(long, value_delimiter = ',')]
    pub skip: Vec<String>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Format {
    Markdown,
    Json,
    Sarif,
}
