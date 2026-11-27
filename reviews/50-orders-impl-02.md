Revision: `ceb387af92a5562977b4e8334519246e21b82bf3`

Files checked (PR #68 file list from `gh pr view 68 --json files`; `git merge-base main ceb387af92a5562977b4e8334519246e21b82bf3` equals the reported base `d5dae930045452fc2badfd76d4b08aa04ff69636`, and the local `git diff --name-only` matches):

- `crates/engine/src/lib.rs`
- `crates/engine/src/orders.rs`
- `crates/engine/tests/orders.rs`
- `crates/rt3ctl/Cargo.toml`
- `crates/rt3ctl/src/main.rs`
- `crates/rt3ctl/src/orders_cmd.rs`
- `crates/rt3ctl/tests/sim_orders.rs`
- `docs/evidence/README.md`
- `docs/evidence/orders-v1.md`
- `reviews/50-orders-impl-01.md`

Reviewer: GPT-6 Luna (`opencode-go/gpt-6-luna#high`)

Mode: OpenCode

Round 1 outcomes:

- **Finding 1 — fixed.** The space escape distinguishes the two reported consists. I also reproduced different canonical lines and hashes for a space versus the literal backslash sequence `\s`.
- **Finding 2 — fixed for the reported case.** A self-consistent 0→99 track checkpoint is now rejected by `state.validate`; the regression test passes. Finding 1 below records a separate remaining checkpoint-validation gap.
- **Finding 3 — fixed.** State digests now equal SHA-256 of canonical text; the synthetic checkpoint’s log hash equals SHA-256 of the log bytes.
- **Finding 4 — fixed.** The new integration test rejects a changed prefix for both an order command and resumed replay without appending.

Verification: The synthetic per-order hashes and CLI golden match the updated evidence; replay from seq 0, seq 1 and the final checkpoint produces byte-identical output. The fixture-backed station test passes with Bath, Gloucester and Tower. The documented fixture final hash did not reproduce with the C3-declared fixture consist and default dwell (Finding 2 below). `make fmt`, `make clippy`, `make test` and `make evidence-check` passed with fixtures set and no checks skipped; CI passed.

Findings:

1. **blocking — Checkpoint validation still accepts a consist no valid order can create.** `crates/engine/src/orders.rs:146-159,204-266`; `crates/rt3ctl/src/orders_cmd.rs:581-601,650`. I reproduced a seq-0 checkpoint with valid station references, no track, and an empty locomotive name, with correct SHA-256 values for the state and empty log. `sim replay --json` accepts it. `OrderState::apply` rejects an empty locomotive, but `validate` does not inspect consist names, and `state_from_json` constructs the consist before calling that incomplete validation. A self-consistent checkpoint can therefore still resume an order-impossible state.

2. **non-blocking — The fixture result hash is not reproducible from the documented fixture setup.** `docs/evidence/orders-v1.md:67-74`. Using the stated station labels and 2,054.776-unit track, plus the C3 fixture consist (PlanetL and the first four parsed car display names) with default dwell, I reproduced final hash `c53351cdb88921370785c32a2b9857f718c5147f15a3c9c3739cb0e5f463f857`, not the documented `3e63bacf4bff67423bdf7235319f2c9ae6675dd860b26e0b1343f79d114d08b5`. The orders evidence does not specify the fixture car list or dwell; add the exact inputs or correct the hash so this result can be independently checked.

3. **non-blocking — The stated unique-label invariant is not enforced.** `crates/engine/src/orders.rs:200-217`; `docs/evidence/orders-v1.md:85-87`. `validate` checks that labels are non-empty and station slots are unique, but does not check label uniqueness. I reproduced `sim init` accepting two distinct slots with the same label, despite the source comment and evidence describing unique station labels. Either enforce that invariant or describe labels as non-empty rather than unique.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
