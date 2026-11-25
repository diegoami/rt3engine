# Implementation review — M5 C3/C4 (round 3)

- **Revision:** `0f5e2ca92872eeda4388985e5fe25ad1c01532c8`
- **Pull request:** [#48](https://github.com/diegoami/rt3engine/pull/48), base `main` at `40494db390e5e661c98b7c97a530c07c747c239f`
- **Files checked (PR file list):**
  - `Cargo.lock`
  - `crates/engine/src/economy.rs`
  - `crates/engine/tests/economy.rs`
  - `crates/engine/tests/fixtures/economy-drain.txt`
  - `crates/rt3ctl/Cargo.toml`
  - `crates/rt3ctl/src/main.rs`
  - `crates/rt3ctl/src/sim_cmd.rs`
  - `crates/rt3ctl/tests/sim_run.rs`
  - `docs/evidence/economy-v1.md`
  - `reviews/37-sim-run-impl-01.md`
  - `reviews/37-sim-run-impl-02.md`
- **How obtained:** Fetched `origin/m5/run`; `gh pr view 48 --json headRefOid,baseRefOid,files` and local `HEAD` both reported revision `0f5e2ca92872eeda4388985e5fe25ad1c01532c8`, with the base and eleven paths above. `git merge-base main HEAD` equalled the reported base, and `git diff --name-only 40494db390e5e661c98b7c97a530c07c747c239f..HEAD` matched the PR file list. Compared with round 2, the implementation file list adds only `crates/rt3ctl/Cargo.toml` and `Cargo.lock`; read the dependency and parser changes.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`; merge delegated on AGREE.

## Round 2 follow-up

1. **Resolved — round 2 finding 1.** `serde_json::from_str` now rejects the previous false accepts, including trailing separators, `+1`/`01`, invalid name syntax and invalid escapes. The decade test parses genuine dump lines with the same JSON parser before validating their fields.

## Findings

No findings.

## Verification

- `RT3_HOME=… cargo test -p rt3ctl --test sim_run -- --nocapture` passed both tests. The malformed-line test rejected the round-2 cases and the added trailing junk, object/array commas, `1.`, wrong-typed fields, missing field and raw control character. The install-backed test parsed the genuine 120-month dumps on both maps and passed month order, 41 distinct cargo names, finite non-negative values, completed stop reason, same-seed replay, changed-seed divergence and summary digest checks.
- Probed the schema logic with additional JSON: wrong-typed or missing required fields and `NaN`/`Infinity` spellings are rejected. Extra properties are accepted; I do not count this as a false accept because the documented schema requires the named fields and does not prohibit additive properties. An empty cargo array parses in isolation, but the install-backed decade assertion rejects it because each month must contain 41 cargo entries.
- `serde_json` is under `[dev-dependencies]` in `crates/rt3ctl/Cargo.toml:16-17`; `cargo tree -e normal -p rt3ctl` contains no `serde_json`, while `cargo tree -e all -p rt3ctl` shows it under dev-dependencies. `cargo check -p rt3ctl --all-targets` passed; no runtime dependency was added.
- `$HOME/tools/bin/make fmt`, `make clippy`, and `make test` passed. The test run used the requested installation and fixtures under `review-tmp/saves/`; workspace tests and the evidence check passed.
- CI passed for this exact revision (`check`, run `45673058759`). No emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
