Revision: `43d857663895ffba252554426baad826648db980`

Files checked, obtained from `gh pr view 69 --json headRefOid,baseRefOid,files` and matched against `git diff --name-only 43dde7fb691ef550f945a3e6c5879d8be85c415d..43d857663895ffba252554426baad826648db980`; `git merge-base main 43d857663895ffba252554426baad826648db980` was `43dde7fb691ef550f945a3e6c5879d8be85c415d`:

- `crates/engine/src/company.rs`
- `crates/engine/src/lib.rs`
- `crates/engine/tests/company.rs`
- `crates/rt3ctl/src/company_cmd.rs`
- `crates/rt3ctl/src/main.rs`
- `crates/rt3ctl/tests/sim_company.rs`
- `docs/evidence/README.md`
- `docs/evidence/rail-economy-v1.md`
- `reviews/51-economy-impl-01.md`

Reviewer: GPT-6 Luna  
Model id: `opencode-go/gpt-6-luna#high`  
Mode: OpenCode implementation review

## Verification

- `make fmt`, `make clippy`, `make test` and a separate evidence check passed with `RT3_HOME` and all supplied fixtures set; no fixture or evidence check skipped. CI passed.
- The engine tests reject non-finite fare/cost results and an unrepresentable competitor start. A separate synthetic run with finite `1e300` rules succeeds with finite totals.
- The synthetic golden remains `cf2fbff569f06dee8dc1948a8384d80adfb4bcc8f3395a51f3597135c663eb0e`. The fixture CLI results and golden remain unchanged; two CLI outputs compared byte-identically, with digest `2d517847f5b1250fd9fc75cc3f8962b7605df9b92868153337894be8056f7e4c`.
- With `fare_per_car_trip` temporarily set to zero, the install-backed test failed at `player cash must be positive`; the default was restored and the worktree is clean.

## Findings

1. **non-blocking — resolved (round-1 blocking finding)** — `crates/engine/src/company.rs:96-110, 155-170, 223-239`; `crates/engine/tests/company.rs:183-218`. The trip calculation and accumulated company totals now reject non-finite values, and checked tick arithmetic rejects unrepresentable player/competitor end times. The new `f64::MAX` fare/cost and `u64::MAX` start tests return `None`; the finite `1e300` probe still succeeds.
2. **non-blocking — resolved (round-1 non-blocking finding)** — `crates/rt3ctl/tests/sim_company.rs:46-49`. The direct positive-cash assertion now precedes the other result assertions. The zero-fare mutation reproducibly fails at that assertion with `player cash must be positive`.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
