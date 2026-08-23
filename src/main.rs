//! codequality-rs: Fast code quality analyzer for Python.
//!
//! Usage:
//!   codequality [OPTIONS] [PATH]

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod analyzer;
mod reporters;

#[derive(Parser, Debug)]
#[command(name = "codequality", version, about)]
struct Cli {
    /// Path to scan (file or directory)
    path: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = Format::Markdown)]
    format: Format,

    /// Only check files with these extensions (comma-separated)
    #[arg(long, value_delimiter = ',', default_value = "py")]
    extensions: Vec<String>,

    /// Skip cache and rescan
    #[arg(long)]
    no_cache: bool,

    /// Only show these rule IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    only: Vec<String>,

    /// Skip these rule IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    skip: Vec<String>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum Format {
    Markdown,
    Json,
    Sarif,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let start = std::time::Instant::now();

    let files = collect_files(&cli.path, &cli.extensions);
    let mut all_findings = Vec::new();

    for file in files {
        match analyzer::analyze_file(&file) {
            Ok(findings) => {
                for f in findings {
                    if !cli.only.is_empty() && !cli.only.contains(&f.rule_id) {
                        continue;
                    }
                    if cli.skip.contains(&f.rule_id) {
                        continue;
                    }
                    all_findings.push(f);
                }
            }
            Err(e) => {
                eprintln!("⚠ Skipped {}: {}", file.display(), e);
            }
        }
    }

    let elapsed = start.elapsed();
    let output = match cli.format {
        Format::Markdown => reporters::render_markdown(&all_findings, &cli.path.display().to_string()),
        Format::Json => reporters::render_json(&all_findings)?,
        Format::Sarif => reporters::render_sarif(&all_findings, &cli.path.display().to_string())?,
    };

    print!("{}", output);

    eprintln!(
        "\n✓ {} issue(s) found in {:?}",
        all_findings.len(),
        elapsed
    );

    Ok(())
}

fn collect_files(path: &PathBuf, extensions: &[String]) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.clone()];
    }

    if !path.is_dir() {
        return Vec::new();
    }

    let mut files = Vec::new();
    let walker = ignore::WalkBuilder::new(path)
        .standard_filters(true)
        .build();

    for entry in walker.flatten() {
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            if let Some(ext) = entry.path().extension() {
                let ext_str = ext.to_string_lossy().to_string();
                if extensions.iter().any(|e| e == &ext_str) {
                    files.push(entry.path().to_path_buf());
                }
            }
        }
    }

    files
}
