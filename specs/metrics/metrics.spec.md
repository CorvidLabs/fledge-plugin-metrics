---
module: metrics
version: 2
status: active
files:
  - src/main.rs

db_tables: []
depends_on: []
---

# Metrics

## Purpose

Report language line counts, file churn from Git history, and a heuristic test-to-source ratio for the current project without requiring an external metrics binary.

## Public API

### Exported Functions and Types

| Export | Description |
|--------|-------------|
| `ChurnEntry` | File path and commit count in a churn report. |
| `TEST_PATTERNS` | Recognized test-file suffixes. |
| `TEST_PREFIX_PATTERNS` | Recognized test-file prefixes. |
| `SOURCE_EXTENSIONS` | Source languages included in ratio calculation. |
| `is_test_file` | Classify a source filename as a test. |
| `is_source_extension` | Classify a file extension as measurable source. |
| `parse_churn` | Count, order, and limit file entries from Git log output. |

| Surface | Behavior |
|---------|----------|
| default LOC | Count lines by language and print totals. |
| churn | Count commits touching each file and return the requested top entries. |
| tests | Count heuristic test and source files and report their ratio. |
| JSON output | Serialize each report as structured JSON. |

## Invariants

1. LOC counts use tokei and exclude languages with no discovered files.
2. Language rows are ordered by descending code lines and then name.
3. Churn requires a Git worktree and counts each non-empty path occurrence from Git history.
4. Churn ordering is deterministic by descending count and then path.
5. Test ratio scans through a gitignore-aware walker and recognizes only the committed language extensions.
6. A project with no source files reports ratio zero rather than dividing by zero.
7. Human and JSON modes represent the same computed metrics.

## Behavioral Examples

```
Given a Git project containing supported source and test files
When the developer requests LOC, churn, or test-ratio metrics
Then the plugin returns deterministic human-readable or JSON results for the selected metric
```

## Error Cases

| Error | When | Behavior |
|-------|------|----------|
| Not a Git worktree | Churn is requested outside Git | Report that churn requires a repository and exit non-zero. |
| Git unavailable | The Git executable cannot be invoked | Surface the invocation failure. |
| Git log failure | History cannot be read | Report the failed history operation. |
| Serialization failure | JSON output cannot be encoded | Return the serialization error and exit non-zero. |

## Dependencies

- Rust 1.85 or later
- `tokei`, `ignore`, `clap`, `serde`, and `serde_json`
- Git for churn reports

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1 | 2026-07-12 | Document existing LOC, churn, and test-ratio behavior for SpecSync 5 adoption. |
| 2026-07-13 | CHG-0001-adopt-specsync-5-0-1-and-trust-1-0-0-governance-for-the-metrics-fledge-plugin: Adopt SpecSync 5.0.1 and Trust 1.0.0 governance for the Metrics Fledge plugin |
