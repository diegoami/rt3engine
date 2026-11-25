# Implementation review — M5 engine world (round 1)

- **Revision:** `5846bd234ca7391cff2603e276c69a7738803642`
- **Pull request:** [#46](https://github.com/diegoami/rt3engine/pull/46), base `main` at `6289df1c339e51a2d7b08d3be43db9d962195c4f`
- **Files checked:** `Cargo.lock`; `crates/engine/Cargo.toml`; `crates/engine/src/lib.rs`; `crates/engine/src/world.rs`; `crates/engine/tests/world.rs`; `crates/rt3ctl/Cargo.toml`; `crates/rt3ctl/src/main.rs`; `crates/rt3ctl/src/sim_cmd.rs`.
- **How obtained:** `gh pr view 46 --json headRefOid,baseRefOid,files`; local `HEAD` matched the reported head. `git merge-base main <head>` matched the PR base, and `git diff --name-only <merge-base>..<head>` matched the eight PR files. Read the complete merge-base diff.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`.

## Findings

1. **Blocking — malformed version-9701 files can be loaded as worlds.** `crates/engine/src/world.rs:63-77` checks only `GmpMap::peek_version` before scanning terrain, city and building collections; it never validates the map prelude with the structural map parser. Reproduced by copying a shipped map to a temporary file and removing the required NUL terminator from its prelude while leaving its structural sections intact: `rt3ctl sim inspect <temporary-file> --json` exited 0 and reported a world. The map prelude parser rejects this input. Reject malformed preludes before constructing a `World`.
2. **Non-blocking — ordinary non-map files are reported as unsupported map versions.** `crates/engine/src/world.rs:65-71` treats any file of at least four bytes as having a map version because `peek_version` only reads its first word. A temporary text file was rejected with exit 1 and its path in the error, but the diagnostic called its first four bytes (`1768190575`) a “map version.” This is misleading, though it did not load the file.

## Verification

- `RT3_HOME`-backed `cargo test -p engine` passed (2 install-backed tests). An independent temporary `rt3-kit` reader over the raw installed map bytes re-derived: British Isles 513×321, 68 cities, 0 buildings, 0.0–760.5 m; California Island 257×257, 46 cities, 0 buildings, −454.0–3051.1 m; Greenland Growing 193×257, 42 cities, 54 buildings, −130.1–1177.5 m. These match the committed `terrain-scan-v1.json` rows. It parsed 41 cargo records, including `Alcohol`, matching `typed-tables-v1.json`.
- Ran `rt3ctl sim inspect` in text and JSON for all three maps. JSON parsed and matched the expected schema and figures; repeated runs returned the same cargo-name order.
- Probed a missing map (exit 2), an ambiguous map name constructed with two temporary symlinks (exit 2), legacy `Tutorial_1.gmp` v9609 (exit 1), and a temporary non-map text file (exit 1). Each returned a named error without panic. The malformed-prelude acceptance is finding 1.
- `$HOME/tools/bin/make fmt`, `clippy`, and `test` passed with the installation and all three requested save fixtures set. Workspace tests: 59 `rt3-kit`, 2 `engine`, 10 `rt3ctl`; evidence check passed. CI `check` and GitGuardian Security Checks passed.
- Dependency trees are acyclic (`engine → rt3-kit`; `rt3ctl → engine, rt3-kit`). The change uses no `unsafe`; the loader and inspection path only read the installation. No emulator was booted. The output and comments claim structural record counts only, not city or building semantics.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
