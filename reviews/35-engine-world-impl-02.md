# Implementation review — M5 engine world (round 2)

- **Revision:** `67d3becd63b0191dc5e02bff63cc2287cedc0d1c`
- **Pull request:** [#46](https://github.com/diegoami/rt3engine/pull/46), base `main` at `6289df1c339e51a2d7b08d3be43db9d962195c4f`
- **Files checked:** `Cargo.lock`; `crates/engine/Cargo.toml`; `crates/engine/src/lib.rs`; `crates/engine/src/world.rs`; `crates/engine/tests/world.rs`; `crates/rt3ctl/Cargo.toml`; `crates/rt3ctl/src/main.rs`; `crates/rt3ctl/src/sim_cmd.rs`.
- **How obtained:** Re-fetched `origin/m5/world`; `gh pr view 46 --json headRefOid,baseRefOid,files` reported this revision and local `HEAD` matched. The merge-base is `6289df1c339e51a2d7b08d3be43db9d962195c4f`. The PR file list contains these same eight implementation files plus the prior review record `reviews/35-engine-world-impl-01.md`. Read the full implementation update diff.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`.

## Findings

No new findings.

## Round 1 follow-up

1. **Resolved — Round 1 blocking finding 1.** `crates/engine/src/world.rs:66-67` now validates the prelude with `GmpMap::from_bytes` before reading terrain or collections. Reproduced with a temporary copy of British Isles whose prelude NUL terminator was removed while later sections remained intact: the parser rejected it, and `rt3ctl sim inspect` exited 1 with the named “not a supported version-9701 map” error. The install-backed regression test is at `crates/engine/tests/world.rs:167-192`.
2. **Resolved — Round 1 non-blocking finding 2.** The same error prefix identifies non-map input as “not a supported version-9701 map” (`crates/engine/src/world.rs:66-67`). A temporary non-map text file exited 1 with that diagnostic and its path; the regression test is at `crates/engine/tests/world.rs:194-205`.

## Verification

- `RT3_HOME`-backed `cargo test -p engine` passed all four integration tests. Direct text and JSON inspection also passed for British Isles (513×321, 68 cities, 0 buildings), California Island (257×257, 46 cities, 0 buildings), and Greenland Growing (193×257, 42 cities, 54 buildings); each reported 41 cargo types. The structural claims remain unchanged.
- `$HOME/tools/bin/make fmt`, `clippy`, and `test` passed with the installation and all three requested save fixtures set. Workspace tests: 59 `rt3-kit`, 4 `engine`, 10 `rt3ctl`; evidence check passed. CI `check` and GitGuardian Security Checks passed.
- No new defect found. The implementation continues to report structural counts only; no city or building semantics are claimed. No emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

- `engine::world` and `rt3ctl sim inspect` land claim C1's slice: PR #46,
  merged as `4eb6c32`.
- Malformed preludes, non-map files and legacy versions are rejected by the
  map parser before a world is built.
- `cargo test -p engine` passes 4 install-backed tests; `make fmt`,
  `make clippy` and `make test` are green; CI green.
- Reviews: `reviews/35-engine-world-impl-01.md` (BLOCK) and this file (AGREE).

— Implementer (zai-coding-plan/glm-5.3)
