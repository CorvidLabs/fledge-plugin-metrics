# fledge-plugin-metrics

Code metrics for [fledge](https://github.com/CorvidLabs/fledge) — LOC, churn, test ratio.

Lived in fledge core through v0.14 as ~600 LOC of file-walking and per-language counting. Moved to this plugin in v0.15 because (1) it overlapped with battle-tested tools like `tokei`/`scc` and (2) it never composed with specs/AI/lanes — it was a standalone read tool.

This plugin is a thin shell over `tokei` and `git`. Smaller surface, better correctness.

## Install

```sh
fledge plugins install CorvidLabs/fledge-plugin-metrics
```

Requires [`tokei`](https://github.com/XAMPPRocky/tokei) for the LOC summary (`cargo install tokei`).

## Commands

### `fledge metrics` (default)

LOC summary by language via `tokei`.

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

Test/source file ratio by filename heuristic (`*_test.rs`, `*.test.ts`, `*Test.java`, etc.).

### `--json` everywhere

Each subcommand has a stable JSON shape. `--json` on `--churn` emits an array of `{path, commits}` objects; `--json` on `--tests` emits `{test_files, source_files, ratio}`; the default `tokei` summary forwards `tokei --output json`.

## License

MIT
