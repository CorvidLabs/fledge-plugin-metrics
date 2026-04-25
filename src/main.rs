//! fledge-metrics — code metrics for the current project.
//!
//! Replaces the pre-v0.15 in-core `fledge metrics` command. LOC counting
//! comes from the `tokei` crate (no external binary required); churn comes
//! from `git log`; the test ratio comes from a filename heuristic over a
//! gitignore-aware walker.

use std::collections::HashMap;
use std::process::Command;

use anyhow::{Context, Result};
use clap::Parser;
use ignore::WalkBuilder;
use serde::Serialize;
use tokei::{Config, Languages};

#[derive(Parser, Debug)]
#[command(
    name = "fledge-metrics",
    about = "Project code metrics (LOC, churn, test ratio)",
    long_about = None,
    disable_version_flag = true,
)]
struct Cli {
    /// Show file churn from git history (commits per file).
    #[arg(long, conflicts_with = "tests")]
    churn: bool,

    /// Show test/source file ratio (filename heuristic).
    #[arg(long)]
    tests: bool,

    /// Max entries when using --churn.
    #[arg(short = 'l', long, default_value_t = 20)]
    limit: usize,

    /// Output as JSON.
    #[arg(long)]
    json: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.churn {
        run_churn(cli.limit, cli.json)
    } else if cli.tests {
        run_tests(cli.json)
    } else {
        run_loc(cli.json)
    }
}

// MARK: - LOC summary

#[derive(Serialize)]
struct LocLanguage {
    name: String,
    files: usize,
    lines: usize,
    code: usize,
    comments: usize,
    blanks: usize,
}

#[derive(Serialize)]
struct LocTotals {
    files: usize,
    lines: usize,
    code: usize,
    comments: usize,
    blanks: usize,
}

#[derive(Serialize)]
struct LocReport {
    languages: Vec<LocLanguage>,
    totals: LocTotals,
}

fn run_loc(as_json: bool) -> Result<()> {
    let mut languages = Languages::new();
    let config = Config::default();
    languages.get_statistics(&["."], &[], &config);

    let mut entries: Vec<LocLanguage> = languages
        .iter()
        .map(|(language_type, language)| LocLanguage {
            name: language_type.name().to_string(),
            files: language.reports.len(),
            lines: language.lines(),
            code: language.code,
            comments: language.comments,
            blanks: language.blanks,
        })
        .filter(|entry| entry.files > 0)
        .collect();

    entries.sort_by(|a, b| b.code.cmp(&a.code).then(a.name.cmp(&b.name)));

    let totals = LocTotals {
        files: entries.iter().map(|e| e.files).sum(),
        lines: entries.iter().map(|e| e.lines).sum(),
        code: entries.iter().map(|e| e.code).sum(),
        comments: entries.iter().map(|e| e.comments).sum(),
        blanks: entries.iter().map(|e| e.blanks).sum(),
    };

    if as_json {
        let report = LocReport { languages: entries, totals };
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!(
        "{:<20} {:>8} {:>10} {:>10} {:>10} {:>10}",
        "LANGUAGE", "FILES", "LINES", "CODE", "COMMENTS", "BLANKS"
    );
    println!("{}", "-".repeat(72));
    for entry in &entries {
        println!(
            "{:<20} {:>8} {:>10} {:>10} {:>10} {:>10}",
            entry.name, entry.files, entry.lines, entry.code, entry.comments, entry.blanks
        );
    }
    println!("{}", "-".repeat(72));
    println!(
        "{:<20} {:>8} {:>10} {:>10} {:>10} {:>10}",
        "TOTAL", totals.files, totals.lines, totals.code, totals.comments, totals.blanks
    );
    Ok(())
}

// MARK: - Churn

#[derive(Serialize)]
struct ChurnEntry {
    path: String,
    commits: u32,
}

fn run_churn(limit: usize, as_json: bool) -> Result<()> {
    let inside = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .context("failed to invoke git — is git installed?")?;
    if !inside.status.success() {
        anyhow::bail!("fledge-metrics: --churn requires a git repository.");
    }

    let log = Command::new("git")
        .args(["log", "--name-only", "--pretty=format:"])
        .output()
        .context("failed to run `git log`")?;
    if !log.status.success() {
        anyhow::bail!("fledge-metrics: `git log` failed");
    }
    let stdout = String::from_utf8_lossy(&log.stdout);

    let mut counts: HashMap<String, u32> = HashMap::new();
    for line in stdout.lines() {
        let path = line.trim();
        if path.is_empty() {
            continue;
        }
        *counts.entry(path.to_string()).or_insert(0) += 1;
    }

    let mut sorted: Vec<ChurnEntry> = counts
        .into_iter()
        .map(|(path, commits)| ChurnEntry { path, commits })
        .collect();
    sorted.sort_by(|a, b| b.commits.cmp(&a.commits).then(a.path.cmp(&b.path)));
    sorted.truncate(limit);

    if as_json {
        println!("{}", serde_json::to_string_pretty(&sorted)?);
        return Ok(());
    }

    println!("{:<60} COMMITS", "FILE");
    for entry in &sorted {
        println!("{:<60} {}", entry.path, entry.commits);
    }
    Ok(())
}

// MARK: - Test/source ratio

#[derive(Serialize)]
struct TestsReport {
    test_files: usize,
    source_files: usize,
    ratio: f64,
}

const TEST_PATTERNS: &[&str] = &[
    "_test.rs",
    ".test.ts",
    ".test.tsx",
    ".test.js",
    ".test.jsx",
    ".spec.ts",
    ".spec.js",
    "Test.java",
    "Tests.swift",
    "_test.py",
    "_test.go",
];

const TEST_PREFIX_PATTERNS: &[&str] = &["test_"];

const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "py", "swift", "kt", "java", "go", "rb",
];

fn run_tests(as_json: bool) -> Result<()> {
    let mut test_files = 0usize;
    let mut source_files = 0usize;

    let walker = WalkBuilder::new(".").hidden(false).build();
    for entry in walker.flatten() {
        let path = entry.path();
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else { continue };

        let is_source = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| SOURCE_EXTENSIONS.contains(&ext));
        if !is_source {
            continue;
        }
        source_files += 1;

        let is_test = TEST_PATTERNS.iter().any(|p| file_name.ends_with(p))
            || TEST_PREFIX_PATTERNS.iter().any(|p| file_name.starts_with(p));
        if is_test {
            test_files += 1;
        }
    }

    let ratio = if source_files > 0 {
        test_files as f64 / source_files as f64
    } else {
        0.0
    };

    if as_json {
        let report = TestsReport { test_files, source_files, ratio };
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("Test files:    {test_files}");
    println!("Source files:  {source_files}");
    println!("Ratio:         {ratio:.3}");
    Ok(())
}
