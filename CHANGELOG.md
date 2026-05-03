# Changelog

## [v0.2.1] - 2026-05-03

### Added

- CI workflow (build/test/clippy/rustfmt on Linux, macOS, Windows).
- Release workflow that cross-compiles binaries for `x86_64-linux`, `x86_64-darwin`, `aarch64-darwin`, `x86_64-windows` and attaches them to the GitHub release on tag push.

### Changed

- Applied `cargo fmt` so the lint job stays green.

## [v0.2.0]

- Rewritten in Rust. Links `tokei` as a library so the only build-time requirement is a Rust toolchain. `git` is still required at runtime for `--churn`. Plugin-owned JSON shapes (no longer pass-through from `tokei --output json`).
