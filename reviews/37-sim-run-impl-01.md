# Implementation review — M5 C3/C4 (round 1)

- **Revision:** `3b4603c19a67787a61aa73164cfca3e4ea84ae46`
- **Pull request:** [#48](https://github.com/diegoami/rt3engine/pull/48), base `main` at `40494db390e5e661c98b7c97a530c07c747c239f`
- **Files checked (PR file list):**
  - `crates/engine/src/economy.rs`
  - `crates/engine/tests/economy.rs`
  - `crates/engine/tests/fixtures/economy-drain.txt`
  - `crates/rt3ctl/src/main.rs`
  - `crates/rt3ctl/src/sim_cmd.rs`
  - `crates/rt3ctl/tests/sim_run.rs`
  - `docs/evidence/economy-v1.md`
- **How obtained:** `gh pr view 48 --json headRefOid,baseRefOid,files` reported this head, base and seven paths; local `HEAD` equalled the reported head. `git merge-base main 3b4603c19a67787a61aa73164cfca3e4ea84ae46` equalled the reported base, and `git diff --name-only 40494db390e5e661c98b7c97a530c07c747c239f..3b4603c19a67787a61aa73164cfca3e4ea84ae46` matched the PR file list above. Read the full merge-base diff.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`; merge delegated on AGREE.

## Findings

1. **blocking — the install-backed test does not validate the dump JSON schema.** `crates/rt3ctl/tests/sim_run.rs:40-53` only checks a line's prefix, presence of the `"cargo":[` substring, suffix, and parseability of the month token. I reproduced that 120 synthetic lines of the form `{"month":n,"cargo":[not-json-seed-42]}` satisfy the test's line/order, replay/divergence, stop/month and digest predicates, although a JSON parser rejects them. Thus the C4 test can accept a malformed monthly dump; its replay and digest checks establish stable bytes, not valid schema. Parse each full line and assert the month and cargo fields and their value types.

## Verification

- Independently built `rt3ctl` and ran 120-month dumps for British Isles and California Island. Each emitted 120 parseable JSON lines with months `1..=120`, and each summary reported `completed` with a SHA-256 matching its dump file.
- Ran each map twice with seed 42 and once with seed 43. Same-seed files matched byte-for-byte; seed-43 digests differed (`2e31ca59d4030b5ee1aa5adc9108c6cf6281e24de12f18698e4843a9f5f18545` vs `b3fde4b310b7ee992dadfa02709ef7bc579d58f4c35b2aeca729dc1785775644` for British Isles; `be781edff27cc811e66a4ae4d97d3d03c78b66beec09690935896b62c6e6d8d4` vs `77f3d0006e29cb813fdf557444b906e1aca42945e28d37cb3f46a7aa50102398` for California Island). The dumps contain no seed field; seed 43 changed 4,919 stock values on each map, plus 4,911 price values on British Isles and 4,920 on California Island, confirming actual state divergence.
- Reproduced the schema-check gap with synthetic malformed dumps that satisfy the integration test's other assertions while failing JSON parsing.
- The rescaled defaults in `crates/engine/src/economy.rs:61-73` match `docs/evidence/economy-v1.md:28-38`. The C2 identity test remains unsaturated; the cap and drain fixtures still reach stock and price clamps; the 120-month bounds and updated golden-hash tests pass. On the two maps, seed-42 stock ranged 29.28–9,934.36 and 3,197.28–9,960.76 respectively, with no observations at either stock bound.
- `--months 0` and bad seed/month/tick values returned named errors with exit 2; a missing map returned a named error with exit 2; an unwritable dump path returned a named error with exit 1. None panicked.
- The integration test uses `env!("CARGO_BIN_EXE_rt3ctl")`, so it runs the built executable. Its stop-reason, digest and 120-line/month-sequence assertions are exercised by the successful install-backed `make test`.
- `$HOME/tools/bin/make fmt`, `make clippy`, and `make test` passed. The test run used the requested installation and fixtures under `review-tmp/saves/`; the workspace tests and evidence check passed. CI passed (`check` and GitGuardian Security Checks). No emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
