# Implementation review: M4 save actors

- **Revision:** `08878503f8c21bc75312161b7f556f0c042b8727`
- **Base:** `802db5b2b9f6bef819270dc149a95972115e775b`
- **Files checked:** PR #35's file list from `gh pr view 35 --json files`, cross-checked against `git diff --name-only <base>...<revision>`:
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
  - `reviews/26-save-actors-impl-01.md`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review, round 2; project slot is `design: none`.

## Round 1 finding

1. **Resolved** — `crates/rt3-kit/src/fixed.rs:166` now asserts that every strict prefix is rejected and reports the accepted prefix length if the assertion fails. The earlier round's blocking finding is closed.

## Findings

None.

## Verification

- The implementation commit changes only `crates/rt3-kit/src/fixed.rs`; the PR file list and merge-base diff both contain 14 files.
- Targeted fixed-reader truncation test — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo test --workspace` with `RT3_HOME`, `RT3_SAVE` and `RT3_CONTROL_SAVE` set — 58 `rt3-kit` tests and 5 `rt3ctl` tests passed, including both fixture-backed saves and all shipped-map collection checks.
- `tools/evidence.sh check` with both save fixtures and `RT3_HOME` — passed.
- `git diff --check` — passed.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
