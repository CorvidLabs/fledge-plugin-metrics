---
change: CHG-0002-correct-metrics-rollout-review-findings-and-requirement-evidence
artifact: testing
---

# Testing

Run `fledge lanes run verify` for formatting, Clippy with warnings denied, 20 unit tests, release build, help smoke, and the live tokei-backed JSON report. Run strict SpecSync at non-vacuous 100% file and LOC coverage, require all four agent integrations, and run Trust doctor/verify.

`REQ-metrics-001` is evidenced by the actual JSON report smoke plus the aggregation and deterministic-ordering unit tests. Hosted checks must pass at the exact reviewed head before merge.
