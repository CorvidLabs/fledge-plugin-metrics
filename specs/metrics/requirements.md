---
spec: metrics.spec.md
---

## User Stories

- As a developer, I want quick deterministic project metrics without configuring a separate analysis service.

## Acceptance Criteria

### REQ-metrics-001

The default report SHALL count files, lines, code, comments, and blanks per discovered language and in total.

Acceptance Criteria

- The native smoke executes the live tokei-backed JSON report successfully.
- Unit tests validate total aggregation and deterministic language ordering over constructed language entries.

### REQ-metrics-002

Churn SHALL count Git-history occurrences per path, order ties deterministically, and honor the configured result limit.

### REQ-metrics-003

Test ratio SHALL classify only supported source extensions using committed filename heuristics and a gitignore-aware walk.

### REQ-metrics-004

Every metric SHALL support equivalent human-readable and structured JSON output.

### REQ-metrics-005

Missing Git context or failed history access SHALL fail explicitly rather than emit an empty successful churn report.

## Constraints

- Test classification is filename-based and does not measure executed test coverage.

## Out of Scope

- Cyclomatic complexity, runtime profiling, and semantic test coverage.
