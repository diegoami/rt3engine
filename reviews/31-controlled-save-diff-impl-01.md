# Implementation review — controlled save diff

- **Revision:** `31d9df51a1320c42d8374392619aed538a1c11bc`
- **Target:** PR #40, base `main` at `51e1c0552f78bbce2cb5998f425ec9d65f2557ad`, head `31d9df51a1320c42d8374392619aed538a1c11bc`. Confirmed with `gh pr view 40 --json files,baseRefOid,headRefOid`; held revision equals PR head.
- **Files checked:** the PR file list from GitHub, cross-checked against `git diff --name-only 51e1c0552f78bbce2cb5998f425ec9d65f2557ad..31d9df51a1320c42d8374392619aed538a1c11bc`:
  - `crates/rt3-kit/src/lib.rs`
  - `crates/rt3ctl/src/main.rs`
  - `crates/rt3ctl/src/save_diff.rs`
  - `docs/environment.md`
  - `docs/evidence/README.md`
  - `docs/evidence/controlled-save-diff-v1.json`
  - `docs/evidence/controlled-save-diff-v1.md`
  - `docs/evidence/gmc-save-v1.md`
  - `tools/evidence.sh`
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`), fresh independent reviewer context.
- **Mode:** OpenCode; `design: none`.

## Verification

- Verified the private fixtures without copying or modifying them: the first is 10,285,120 bytes, SHA-256 `f892bc28ea1fb3bb1ff60f3ed443e82de7818d3fd7a7c26a93e974cfb69ac6bc`; the second is 10,315,724 bytes, SHA-256 `225a98ff8939ae3b4ff5fb6f36cc4b730042017d83f379333d55d783ab0e75b6`.
- Ran `rt3ctl save diff` on that pair in JSON mode. Output is byte-for-byte equal to `docs/evidence/controlled-save-diff-v1.json` and parses as JSON. Confirmed the `+30,604` file delta, matching source map, building deltas `+23` records / `+20` capacity, the shared `+30,595` company/player/city/event section shifts, company offsets `[55]`, player offsets `[485, 486, 487]` and `[486, 487]`, and no raw-record fields in the output.
- Independently inspected both saves: source maps, company/player counts and names, and company founding years match the evidence.
- `cargo fmt --all -- --check` passed.
- `cargo clippy --workspace --all-targets -- -D warnings` passed.
- `cargo test --workspace` passed: 59 `rt3-kit` tests and 9 `rt3ctl` tests, including the second controlled fixture test.
- `CARGO="$(command -v cargo)" sh tools/evidence.sh check` passed with the install and both controlled fixtures available. `make` is unavailable in this environment, so I ran the gate commands directly.
- Reproduced the count-mismatch behavior using the controlled save and the valid original-save fixture: text mode exits 0 and prints `companies 0/3 changed, players 3/3 changed`; JSON mode exits 1 with zero stdout bytes and `actor record counts differ`.

## Finding

1. **Blocking — reject or correctly account for unequal actor counts in text mode.** `crates/rt3ctl/src/save_diff.rs:84-89` compares only `left.iter().zip(right)`, silently discarding unmatched company/player records. `run` uses this helper for the non-JSON path at lines 36-49, and that path reports success. On the reproduced valid pair with three companies on the left and none on the right, it reports `companies 0/3 changed`; the additional player record on the right is also omitted. This makes the reported raw-record difference counts incorrect for valid saves with unequal actor counts, while JSON mode already treats that condition as an error. Apply consistent count validation to text mode or explicitly report additions/removals; do not return a successful truncated comparison.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
