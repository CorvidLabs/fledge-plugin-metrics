---
spec: metrics.spec.md
---

## Context

This plugin replaces the pre-0.15 in-core Fledge metrics command with a standalone cross-platform Rust binary.

## Related Modules

- Fledge plugin command registration.
- Git history and gitignore conventions.

## Design Decisions

- Embed tokei so LOC does not depend on a separately installed binary.
- Keep test ratios explicitly heuristic and deterministic.
