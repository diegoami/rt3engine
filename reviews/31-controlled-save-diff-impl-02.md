# Implementation review — controlled save diff, round 02

- **Revision:** `bbb672dd923c67f36172a56233e8c0f7d9189bfb`
- **Target:** PR #40, base `main` at `51e1c0552f78bbce2cb5998f425ec9d65f2557ad`, head `bbb672dd923c67f36172a56233e8c0f7d9189bfb`. Confirmed with `gh pr view 40 --json files,baseRefOid,headRefOid`; held revision equals PR head.
- **Files checked:** the PR file list from GitHub, cross-checked against `git diff --name-only 51e1c0552f78bbce2cb5998f425ec9d65f2557ad..bbb672dd923c67f36172a56233e8c0f7d9189bfb`:
  - `crates/rt3-kit/src/lib.rs`
  - `crates/rt3ctl/src/main.rs`
  - `crates/rt3ctl/src/save_diff.rs`
  - `docs/environment.md`
  - `docs/evidence/README.md`
  - `docs/evidence/controlled-save-diff-v1.json`
  - `docs/evidence/controlled-save-diff-v1.md`
  - `docs/evidence/gmc-save-v1.md`
  - `reviews/31-controlled-save-diff-impl-01.md`
  - `tools/evidence.sh`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`), independent re-review.
- **Mode:** OpenCode; `design: none`.

## Verification

- Inspected the PR head and implementation delta from round 01. The fix adds `validate_actor_counts` and invokes it before either CLI output mode and at the start of direct JSON rendering. The remaining `zip` comparison is now reached only after equal company and player counts are confirmed. The other file added since the original implementation commit is the round-01 review record.
- Reproduced the original count-mismatch pair using the private controlled save and valid original-save fixture. Both text and JSON modes now exit 1, write zero stdout bytes, and report `actor record counts differ`; text mode no longer reports truncated change counts. The new unit test also checks the validator and direct JSON-render error.
- Re-ran the equal-count controlled pair. Text output remains `companies 1/3 changed, players 2/3 changed`; JSON is valid and byte-for-byte matches `docs/evidence/controlled-save-diff-v1.json`.
- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo test --workspace` passed: 59 `rt3-kit` tests and 10 `rt3ctl` tests.
- `CARGO="$(command -v cargo)" sh tools/evidence.sh check` passed with the install and both controlled fixtures available.
- `git diff --check` passed for the PR range and the round-01-to-head range. `make` is unavailable in this environment, so I ran the gate commands directly.

## Findings

No blocking findings remain.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
