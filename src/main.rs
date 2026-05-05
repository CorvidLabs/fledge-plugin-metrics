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
        let report = LocReport {
            languages: entries,
            totals,
        };
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
pub struct ChurnEntry {
    pub path: String,
    pub commits: u32,
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
    let sorted = parse_churn(&stdout, limit);

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

pub const TEST_PATTERNS: &[&str] = &[
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

pub const TEST_PREFIX_PATTERNS: &[&str] = &["test_"];

pub const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "ts", "tsx", "js", "jsx", "py", "swift", "kt", "java", "go", "rb",
];

/// Returns true if `file_name` looks like a test file based on suffix/prefix heuristics.
pub fn is_test_file(file_name: &str) -> bool {
    TEST_PATTERNS.iter().any(|p| file_name.ends_with(p))
        || TEST_PREFIX_PATTERNS
            .iter()
            .any(|p| file_name.starts_with(p))
}

/// Returns true if a file extension belongs to a recognised source language.
pub fn is_source_extension(ext: &str) -> bool {
    SOURCE_EXTENSIONS.contains(&ext)
}

/// Parse `git log --name-only --pretty=format:` output into (path, commit_count) pairs,
/// sorted descending by count and truncated to `limit`.
pub fn parse_churn(log_output: &str, limit: usize) -> Vec<ChurnEntry> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for line in log_output.lines() {
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
    sorted
}

fn run_tests(as_json: bool) -> Result<()> {
    let mut test_files = 0usize;
    let mut source_files = 0usize;

    let walker = WalkBuilder::new(".").hidden(true).build();
    for entry in walker.flatten() {
        let path = entry.path();
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let is_source = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(is_source_extension);
        if !is_source {
            continue;
        }

        if is_test_file(file_name) {
            test_files += 1;
        } else {
            source_files += 1;
        }
    }

    let ratio = if source_files > 0 {
        test_files as f64 / source_files as f64
    } else {
        0.0
    };

    if as_json {
        let report = TestsReport {
            test_files,
            source_files,
            ratio,
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("Test files:    {test_files}");
    println!("Source files:  {source_files}");
    println!("Ratio:         {ratio:.3}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- is_test_file --

    #[test]
    fn test_rust_test_file() {
        assert!(is_test_file("parser_test.rs"));
    }

    #[test]
    fn test_ts_test_file() {
        assert!(is_test_file("utils.test.ts"));
    }

    #[test]
    fn test_tsx_test_file() {
        assert!(is_test_file("Button.test.tsx"));
    }

    #[test]
    fn test_spec_ts_file() {
        assert!(is_test_file("app.spec.ts"));
    }

    #[test]
    fn test_spec_js_file() {
        assert!(is_test_file("index.spec.js"));
    }

    #[test]
    fn test_java_test_file() {
        assert!(is_test_file("UserServiceTest.java"));
    }

    #[test]
    fn test_swift_tests_file() {
        assert!(is_test_file("ModelTests.swift"));
    }

    #[test]
    fn test_python_test_file() {
        assert!(is_test_file("cli_test.py"));
    }

    #[test]
    fn test_go_test_file() {
        assert!(is_test_file("handler_test.go"));
    }

    #[test]
    fn test_prefix_test_file() {
        assert!(is_test_file("test_helpers.py"));
    }

    #[test]
    fn test_regular_source_not_classified_as_test() {
        assert!(!is_test_file("main.rs"));
        assert!(!is_test_file("utils.ts"));
        assert!(!is_test_file("App.tsx"));
        assert!(!is_test_file("server.py"));
        assert!(!is_test_file("Controller.java"));
    }

    // -- is_source_extension --

    #[test]
    fn test_recognised_source_extensions() {
        for ext in &[
            "rs", "ts", "tsx", "js", "jsx", "py", "swift", "kt", "java", "go", "rb",
        ] {
            assert!(is_source_extension(ext), "{ext} should be recognised");
        }
    }

    #[test]
    fn test_non_source_extensions() {
        for ext in &["md", "toml", "json", "yaml", "lock", "txt"] {
            assert!(!is_source_extension(ext), "{ext} should not be recognised");
        }
    }

    // -- parse_churn --

    #[test]
    fn test_parse_churn_basic() {
        let log = "src/main.rs\nsrc/lib.rs\nsrc/main.rs\n\nsrc/main.rs\n";
        let result = parse_churn(log, 10);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].path, "src/main.rs");
        assert_eq!(result[0].commits, 3);
        assert_eq!(result[1].path, "src/lib.rs");
        assert_eq!(result[1].commits, 1);
    }

    #[test]
    fn test_parse_churn_respects_limit() {
        let log = "a.rs\nb.rs\nc.rs\nd.rs\n";
        let result = parse_churn(log, 2);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_churn_empty_input() {
        let result = parse_churn("", 10);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_churn_only_blank_lines() {
        let result = parse_churn("\n\n\n", 10);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_churn_tie_breaking_alphabetical() {
        let log = "b.rs\na.rs\n";
        let result = parse_churn(log, 10);
        // Both have 1 commit; should be sorted alphabetically
        assert_eq!(result[0].path, "a.rs");
        assert_eq!(result[1].path, "b.rs");
    }

    // -- LocTotals aggregation logic --

    #[test]
    fn test_loc_totals_aggregation() {
        let entries = vec![
            LocLanguage {
                name: "Rust".into(),
                files: 10,
                lines: 1000,
                code: 800,
                comments: 100,
                blanks: 100,
            },
            LocLanguage {
                name: "Python".into(),
                files: 5,
                lines: 500,
                code: 400,
                comments: 50,
                blanks: 50,
            },
        ];
        let totals = LocTotals {
            files: entries.iter().map(|e| e.files).sum(),
            lines: entries.iter().map(|e| e.lines).sum(),
            code: entries.iter().map(|e| e.code).sum(),
            comments: entries.iter().map(|e| e.comments).sum(),
            blanks: entries.iter().map(|e| e.blanks).sum(),
        };
        assert_eq!(totals.files, 15);
        assert_eq!(totals.lines, 1500);
        assert_eq!(totals.code, 1200);
        assert_eq!(totals.comments, 150);
        assert_eq!(totals.blanks, 150);
    }

    // -- LocLanguage sorting --

    #[test]
    fn test_loc_entries_sort_by_code_desc() {
        let mut entries = vec![
            LocLanguage {
                name: "Python".into(),
                files: 5,
                lines: 500,
                code: 400,
                comments: 50,
                blanks: 50,
            },
            LocLanguage {
                name: "Rust".into(),
                files: 10,
                lines: 1000,
                code: 800,
                comments: 100,
                blanks: 100,
            },
        ];
        entries.sort_by(|a, b| b.code.cmp(&a.code).then(a.name.cmp(&b.name)));
        assert_eq!(entries[0].name, "Rust");
        assert_eq!(entries[1].name, "Python");
    }
}
