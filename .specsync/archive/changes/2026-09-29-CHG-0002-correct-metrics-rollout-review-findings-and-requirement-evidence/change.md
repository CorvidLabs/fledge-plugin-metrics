---
id: CHG-0002-correct-metrics-rollout-review-findings-and-requirement-evidence
state: archived
type: feature
base_commit: 6e3fbb985f23c9218a7c4202b092a6b1019f6ce1
---

# Correct Metrics rollout review findings and requirement evidence

## Intent

Correct Metrics rollout review findings and requirement evidence

## Affected Canonical Specs

- `metrics`

## Acceptance Criteria

- Generated agent commands preserve user arguments and full free-text descriptions; governance and public documentation paths are lifecycle-gated; REQ-metrics-001 states only the actual tokei smoke and aggregation evidence; exact-head native verification and real 100% coverage pass

## No-spec Rationale

Not applicable

## Migration Note

Migrated by hand to SpecSync 6 per Leif's decision (2026-09-28); the 6.0.0 tool refused to archive this legacy record (`` exact-only delivery input `.github/workflows/trust.yml` changed after acceptance and requires an audited reopen; run `specsync change reopen CHG-0002-correct-metrics-rollout-review-findings-and-requirement-evidence` to re-verify the accepted change, or supersede it from a later change under a module granted the path by `owns` in `.specsync/config.toml` ``).

- Workflow v1 (SpecSync 5) record, accepted on 2026-07-13 by the closing approval already stored in `approvals.json`. SpecSync 6.0.0 reports its accepted evidence as stale, for the reason quoted above.
- Moved by hand from `.specsync/changes/CHG-0002-correct-metrics-rollout-review-findings-and-requirement-evidence/` into the layout `specsync change archive` writes: `accepted-state.json` is the unchanged accepted `state.json`, `state.json` is marked `archived`, and this file's front matter says `archived`.
- `approvals.json`, `verification.json` and every other artifact are the original SpecSync 5 evidence, unchanged. `verification.json` verifies commit `b66b9035cee15d9f31fd743a911af77307c33451`, not the tree this record was archived from.
- There is no `verification-attempts.json`: SpecSync 5 did not write one for this record, and this migration does not invent attempt history.
- This migration added no verification evidence, test result, attempt history, or approval. It is a manual migration, not a fresh re-verification.
- Closing it through the tool takes `specsync change reopen`, `specsync change verify`, then `specsync change accept`, which writes a new closing approval. Per Leif's decision it was archived by hand instead, so no reopen or new approval is recorded.
