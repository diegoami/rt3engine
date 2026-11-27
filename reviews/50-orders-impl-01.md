Revision: `e07fa1d00afadeb2eb4beab0061be37ffed0a3fc`

Files checked (PR #68 file list from `gh pr view 68 --json files`; `git merge-base main e07fa1d00afadeb2eb4beab0061be37ffed0a3fc` equals the reported base `d5dae930045452fc2badfd76d4b08aa04ff69636`, and the local `git diff --name-only` matches):

- `crates/engine/src/lib.rs`
- `crates/engine/src/orders.rs`
- `crates/rt3ctl/Cargo.toml`
- `crates/rt3ctl/src/main.rs`
- `crates/rt3ctl/src/orders_cmd.rs`
- `crates/rt3ctl/tests/sim_orders.rs`
- `docs/evidence/README.md`
- `docs/evidence/orders-v1.md`

Reviewer: GPT-6 Luna (`opencode-go/gpt-6-luna#high`)

Mode: OpenCode

Verification: Reproduced the three-order synthetic flow, its per-order hashes, the final hash, checkpoint/log consistency, and byte-identical replay from seq 0 and seq 1. The fixture-backed order test passed with all three station references and declared labels. Failure-path commands rejected without appending; tampered hashes, mismatched log/checkpoint, and out-of-order lines were rejected. `make fmt`, `make clippy`, `make test`, and `make evidence-check` passed with fixtures set; CI passed. Disabling the replay hash comparison made the tampered-log test fail at `tampered log must be rejected`; the source was restored byte-identically.

Findings:

1. **blocking — Distinct train orders collide in the canonical state text.** `crates/engine/src/orders.rs:218-252,257-259`; `crates/rt3ctl/src/orders_cmd.rs:402-429`. The consist encoding uses spaces as delimiters but does not escape them, and `clean_name` accepts spaces. I reproduced two different valid orders—locomotive `A 1` with no cars, versus locomotive `A` with car `0`—producing the same state line and hash. Replacing the first order in a seq-0 log with the second while retaining its recorded hash makes replay succeed and return the different consist. Thus replay can accept a changed order rather than reproduce the logged state.

2. **blocking — A self-consistent checkpoint can contain an impossible track reference.** `crates/rt3ctl/src/orders_cmd.rs:491-499,563-575`. I reproduced a seq-0 checkpoint with station slots 0 and 1 and a track from 0 to nonexistent slot 99, with its state hash recomputed. `sim replay` accepted it. `state_from_json` rebuilds the track but never verifies its endpoints against the checkpoint’s station records; the checkpoint hash only verifies the serialization, not these invariants. Invalid checkpoint state must be rejected before it is used for replay.

3. **non-blocking — The documented SHA-256 values are double-hashed.** `crates/rt3ctl/src/orders_cmd.rs:250-252,316,344,420,434,455`; `docs/evidence/orders-v1.md:28-30`. `OrderState::state_digest` already returns SHA-256 of the canonical state text, but `hex` hashes its input again; the log hash is likewise SHA-256 of a SHA-256 digest. For the reproduced final state, SHA-256 of the canonical text is `5e8942a7722e8822829d867527616de9614435e69d9783172ecabfc665e580f1`, while the recorded value `7a79e9705f907df1c941f9d6e2bcd49ef80b8f092d8b80bb509007599c4e2bab` is the hash of that digest. The replay remains deterministic, but the `*_sha256` names and evidence description do not describe the value being stored.

4. **non-blocking — No regression test exercises a log/checkpoint mismatch.** `crates/rt3ctl/tests/sim_orders.rs:203-325`. The tests cover invalid orders and a tampered state hash, but do not alter a log prefix after saving a mid-sequence checkpoint and assert rejection without appending. I reproduced that behavior manually for both a new order command and resumed replay; keeping it in the integration suite would protect the checkpoint-prefix check from regressions.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
