# Implementation review — actor names

- **Revision:** `4db29def8fb9c4fc16f9efd4555cb07d55a2644b`
- **Base:** `1889eb2f565294f8885f2fbb69471474a1bebed5`
- **Pull request:** [#38](https://github.com/diegoami/rt3engine/pull/38), `m4/actor-names` → `main`
- **Files checked:** PR #38's file list from `gh pr view 38 --json files`, cross-checked against `git diff --name-status <merge-base>..<revision>`:
  - `crates/rt3-kit/src/companies.rs`
  - `crates/rt3-kit/src/players.rs`
  - `crates/rt3-kit/src/save.rs`
  - `crates/rt3ctl/src/save_cmd.rs`
  - `docs/evidence/controlled-save-inventory-v1.json`
  - `docs/evidence/gmp-actors-v1.md`
  - `docs/evidence/save-inventory-v1.json`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review, round 1; project slot is `design: none`.

## Findings

1. **non-blocking** — `crates/rt3ctl/src/save_cmd.rs:86-103`: JSON output starts at line 52, before the new name validation. When a malformed name is rejected, the command exits with status 1 and reports the error, but leaves a partial, invalid JSON document on stdout. Consider validating names before emitting the document.

## Verification

- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `cargo test --workspace` with `RT3_HOME`, `RT3_SAVE` and `RT3_CONTROL_SAVE` set — 59 `rt3-kit` tests and 5 `rt3ctl` tests passed, including fixture-backed save and shipped-map checks. `make` is unavailable in this environment; the underlying gate commands were run directly.
- `CARGO=<cargo> sh tools/evidence.sh check` with the installation and both save fixtures set — passed.
- Ran `rt3ctl save inspect --json` on both fixtures; the parsed outputs equal their committed JSON artifacts. The arrays preserve record order, and the supplied save has `companies.names: []`.
- On private copies of the controlled fixture, a company name containing a quote and backslash round-tripped through valid JSON. Empty and control-byte company/player labels each returned an error without a panic; rejected output was partial JSON as noted above. The source fixtures were read-only.
- `git diff --check` — passed. No `.gmc`/`.gms` save files or `review-tmp` files are tracked. No emulator/native game launch or original-file writes were used.
- `gh pr checks 38` — both `check` and GitGuardian Security Checks passed.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
