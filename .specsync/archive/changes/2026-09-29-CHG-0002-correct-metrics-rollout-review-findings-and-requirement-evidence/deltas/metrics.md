## MODIFIED

### REQUIREMENT REQ-metrics-001

The default report SHALL count files, lines, code, comments, and blanks per discovered language and in total.

Acceptance Criteria

- The native smoke executes the live tokei-backed JSON report successfully.
- Unit tests validate total aggregation and deterministic language ordering over constructed language entries.
