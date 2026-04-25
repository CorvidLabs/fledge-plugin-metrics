# fledge-plugin-metrics

Code metrics for [fledge](https://github.com/CorvidLabs/fledge) — LOC, churn, test ratio.

Lived in fledge core through v0.14 as ~600 LOC of file-walking and per-language counting. Moved to this plugin in v0.15 because (1) it overlapped with battle-tested tools like `tokei`/`scc` and (2) it never composed with specs/AI/lanes — it was a standalone read tool.

Rewritten in Rust (v0.2.0+). The previous version shelled out to a separately-installed `tokei` binary; this one links `tokei` as a library so the only build-time requirement is a Rust toolchain. `git` is still required at runtime for `--churn`.

## Install

```sh
fledge plugins install CorvidLabs/fledge-plugin-metrics
```

`fledge plugins install` auto-detects `Cargo.toml` and runs `cargo build --release` for you — no separate `tokei` install needed.

## Commands

### `fledge metrics` (default)

LOC summary by language. Output is a fixed-width table sorted by code lines descending:

```
LANGUAGE                FILES      LINES       CODE   COMMENTS     BLANKS
------------------------------------------------------------------------
Rust                       42      18204      16880        420        904
TOML                        5        128        118          0         10
Markdown                    8        612          0        510        102
------------------------------------------------------------------------
TOTAL                      55      18944      16998        930       1016
```

### `fledge metrics --churn [-l <N>]`

Top-N files by commit count. Defaults to top 20.

```
$ fledge metrics --churn -l 5
FILE                                                         COMMITS
src/lanes.rs                                                 47
src/plugin.rs                                                34
src/main.rs                                                  29
src/spec.rs                                                  21
src/work.rs                                                  18
```

### `fledge metrics --tests`

Test/source file ratio by filename heuristic (`*_test.rs`, `*.test.ts`, `*Test.java`, etc.). Walks with `.gitignore` awareness — `node_modules/`, `target/`, etc. are skipped automatically.

### `--json` everywhere

Stable, plugin-owned JSON shapes (no longer pass-through from `tokei --output json`):

`fledge metrics --json`:
```json
{
  "languages": [
    {"name": "Rust", "files": 42, "lines": 18204, "code": 16880, "comments": 420, "blanks": 904}
  ],
  "totals": {"files": 55, "lines": 18944, "code": 16998, "comments": 930, "blanks": 1016}
}
```

`fledge metrics --churn --json`:
```json
[{"path": "src/lanes.rs", "commits": 47}]
```

`fledge metrics --tests --json`:
```json
{"test_files": 12, "source_files": 42, "ratio": 0.286}
```

## License

MIT
