---
spec: metrics.spec.md
---

## Test Plan

### Unit Tests

- Test filename and extension classification across supported languages.
- Churn parsing, ordering, limiting, and empty input.

### Integration Tests

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- `cargo build --release`
- Exercise help and JSON metric output.
