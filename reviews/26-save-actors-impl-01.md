# Implementation review: M4 save actors

- **Revision:** `12471e05eb5d68ff2816d9bf7a619bf0b8af99c0`
- **Base:** `802db5b2b9f6bef819270dc149a95972115e775b`
- **Files checked:** PR #35's file list from `gh pr view 35 --json files`, cross-checked against `git diff --name-status <base>...<revision>`:
  - `crates/rt3-kit/src/companies.rs`
  - `crates/rt3-kit/src/fixed.rs`
  - `crates/rt3-kit/src/lib.rs`
  - `crates/rt3-kit/src/players.rs`
  - `crates/rt3-kit/src/save.rs`
  - `crates/rt3ctl/src/map_cmd.rs`
  - `crates/rt3ctl/src/save_cmd.rs`
  - `docs/evidence/README.md`
  - `docs/evidence/controlled-save-inventory-v1.json`
  - `docs/evidence/gmc-save-v1.md`
  - `docs/evidence/gmp-actors-v1.md`
  - `docs/evidence/save-inventory-v1.json`
  - `docs/evidence/terrain-scan-v1.json`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review, round 1; project slot is `design: none`.

## Findings

1. **blocking** — `crates/rt3-kit/src/fixed.rs:165-167`: the truncation loop discards every parse result. It therefore passes even if a truncated prefix is incorrectly accepted as a collection; it only catches a panic. Assert that each strict prefix is rejected (or cover the relevant truncation boundaries explicitly). This is necessary to verify malformed fixed-record sections are rejected before `SaveGame` accepts them.

## Verification

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo test --workspace` with `RT3_HOME`, `RT3_SAVE` and `RT3_CONTROL_SAVE` set — 58 `rt3-kit` tests and 5 `rt3ctl` tests passed, including both fixture-backed saves and all shipped-map collection checks.
- `tools/evidence.sh check` with both save fixtures and `RT3_HOME` — passed.
- Both `rt3ctl save inspect --json` outputs match their committed JSON artifacts exactly. The artifacts preserve all prior save fields; the terrain artifact preserves all prior per-map fields, has 43 maps, and totals 37 companies / 145 players.
- `git diff --check` — passed. No original save bytes or private fixture files are tracked.
- The Ghidra headless check was not practical here: `analyzeHeadless` could not find a Java runtime.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
