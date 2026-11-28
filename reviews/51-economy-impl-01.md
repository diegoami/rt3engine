Revision: `65a1db74b25634aff6c318dbc85377e0850b21cf`

Files checked, obtained from `gh pr view 69 --json headRefOid,baseRefOid,files` and matched against `git diff --name-only 43dde7fb691ef550f945a3e6c5879d8be85c415d..65a1db74b25634aff6c318dbc85377e0850b21cf`; `git merge-base main 65a1db74b25634aff6c318dbc85377e0850b21cf` was `43dde7fb691ef550f945a3e6c5879d8be85c415d`:

- `crates/engine/src/company.rs`
- `crates/engine/src/lib.rs`
- `crates/engine/tests/company.rs`
- `crates/rt3ctl/src/company_cmd.rs`
- `crates/rt3ctl/src/main.rs`
- `crates/rt3ctl/tests/sim_company.rs`
- `docs/evidence/README.md`
- `docs/evidence/rail-economy-v1.md`

Reviewer: GPT-6 Luna  
Model id: `opencode-go/gpt-6-luna#high`  
Mode: OpenCode implementation review

## Findings

1. **blocking** — `crates/engine/src/company.rs:43-53, 96-100, 144-160, 212-230`. Finite rules can still overflow during calculation, and the resulting non-finite values are accepted into a successful run. Reproduction with a two-car synthetic consist and `fare_per_car_trip = f64::MAX` returned `Some(JointRun)` with non-finite revenue and cash. Separately, `start_tick = u64::MAX` returned `Some` with `horizon_ticks = u64::MAX` although the trip duration makes its end unrepresentable; `saturating_add` hides that overflow. Reject non-finite calculated/accumulated amounts and unrepresentable tick arithmetic instead of returning an invalid run.
2. **non-blocking** — `crates/rt3ctl/tests/sim_company.rs:49-54`. The requested zero-fare guard removal did make the install-backed test fail, but it failed first at the `first_profitable_trip == 1` assertion on line 49; the direct positive-cash assertions on lines 51-54 were not reached. The test catches the mutation, but the failure is not specifically at the positive-cash assertion.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
