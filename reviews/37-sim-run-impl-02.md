# Implementation review — M5 C3/C4 (round 2)

- **Revision:** `2251ebb8de912dd1529acc3562f8173836c8af63`
- **Pull request:** [#48](https://github.com/diegoami/rt3engine/pull/48), base `main` at `40494db390e5e661c98b7c97a530c07c747c239f`
- **Files checked (PR file list):**
  - `crates/engine/src/economy.rs`
  - `crates/engine/tests/economy.rs`
  - `crates/engine/tests/fixtures/economy-drain.txt`
  - `crates/rt3ctl/src/main.rs`
  - `crates/rt3ctl/src/sim_cmd.rs`
  - `crates/rt3ctl/tests/sim_run.rs`
  - `docs/evidence/economy-v1.md`
  - `reviews/37-sim-run-impl-01.md`
- **How obtained:** Fetched `origin/m5/run`; `gh pr view 48 --json headRefOid,baseRefOid,files` and local `HEAD` both reported revision `2251ebb8de912dd1529acc3562f8173836c8af63`, with the base and eight paths above. `git merge-base main HEAD` equalled the reported base, and `git diff --name-only 40494db390e5e661c98b7c97a530c07c747c239f..HEAD` matched the PR file list. The seven implementation paths are unchanged from round 1; the current PR list also includes the posted round-1 review record. Read the round-1-to-round-2 parser diff.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`; merge delegated on AGREE.

## Round 1 follow-up

1. **Partly resolved — round 1 finding 1.** The exact malformed line from round 1 (`{"month":1,"cargo":[not-json-seed-42]}`) is now rejected, and `malformed_dump_lines_are_rejected` passes. The parser still accepts other malformed JSON; see round 2 finding 1.

## Findings

1. **blocking — the hand-written parser still accepts invalid JSON.** `crates/rt3ctl/tests/sim_run.rs:40-73,113-118` does not enforce JSON token grammar: `parse_name` skips the first character without requiring it to be a quote, `parse_number` accepts Rust floats such as `+1` and `01`, and the entry loop treats a final comma as end-of-array. I reproduced acceptance of a cargo name beginning `xCargo00"` with no opening quote, `stock:+1`, `price:01`, and a trailing cargo comma. An isolated probe with these cases built 120 lines with 41 unique non-empty cargo entries and finite non-negative values; the parser and the decade test's cargo assertions accepted them, while a JSON parser rejected the line. This leaves the C4 schema check incomplete. Enforce JSON syntax (or use a JSON parser), and add these counterexamples to the rejection test.

## Verification

- `RT3_HOME=… cargo test -p rt3ctl --test sim_run -- --nocapture`: both tests passed. The malformed-line test rejected the round-1 example, missing price, invalid month and trailing garbage. The install-backed test parsed genuine 120-month dumps for both maps and passed the 41-cargo schema checks, month ordering, completed stop reason, same-seed byte replay, changed-seed divergence, and summary digest checks.
- Parser probes confirmed missing price and non-numeric stock/price are rejected, but trailing cargo separators, invalid `+1`/`01` numbers, an unquoted name start and an invalid name escape are accepted.
- `$HOME/tools/bin/make fmt`, `make clippy`, and `make test` passed. `make test` used the requested installation and fixtures under `review-tmp/saves/`; workspace tests and the evidence check passed.
- CI passed for this exact revision (`check`, run `45671612584`). No emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
