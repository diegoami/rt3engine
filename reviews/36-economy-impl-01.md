# Implementation review — M5 C2 (round 1)

- **Revision:** `5be17f8472ad1243fffba728a7d068573cdcba80`
- **Pull request:** [#47](https://github.com/diegoami/rt3engine/pull/47), base `main` at `6603fab69383f9d0abb94c03f9e26931046884de`
- **Files checked:**
  - `crates/engine/src/economy.rs`
  - `crates/engine/src/lib.rs`
  - `crates/engine/tests/economy.rs`
  - `crates/engine/tests/fixtures/economy-scenario.txt`
  - `docs/evidence/economy-v1.md`
- **How obtained:** `gh pr view 47 --json headRefOid,baseRefOid,files`; local `HEAD` equalled the reported head. `git merge-base main 5be17f8472ad1243fffba728a7d068573cdcba80` equalled the reported base, and `git diff --name-only 6603fab69383f9d0abb94c03f9e26931046884de..5be17f8472ad1243fffba728a7d068573cdcba80` matched the five PR paths above. Read the full merge-base diff.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`; merge delegated on AGREE.

## Findings

1. **blocking — the upper stock cap and price floor are not exercised by the bounds test.** `crates/engine/tests/economy.rs:118-139` runs `synthetic(70, 0)`: 70 cities demand 350 units of each cargo per tick and no buildings produce. With noise bounded by ±2, stock only falls from 1,000 to 0; it never approaches the documented 10,000 stock cap (`docs/evidence/economy-v1.md:29-37`). The price therefore stays between base and at most 1.3× base for these cargo factors, never reaching the 0.5× floor. I independently recomputed this test path: maximum stock is 1,000 and both stocks end at 0. The committed fixture also has 10 cities and 8 buildings, giving each cargo 40 production against 50 demand per tick, so it does not exercise those limits either. Consequently the current 120-month range assertions would still pass if the upper stock clamp or the lower price clamp in `crates/engine/src/economy.rs:138-151` were removed or broken. Add a committed high-production, zero-demand case that reaches the stock cap and asserts the resulting price floor.

## Verification

- `cargo test -p engine --test economy`: 4 passed.
- Independently recomputed the committed fixture with noise disabled: each cargo gets 40 production and 50 demand per tick, so stock is `1,000 + 30 × (40 − 50) = 700` after one month.
- Independently rebuilt the 12-month fixture dump; SHA-256 was `aebe7267b327db3ed1b0be96425d5b8f2909c6f78b021cff6df50123a8c0a322`, matching the committed golden. Same-seed dumps matched byte-for-byte; seed 43 differed from seed 42.
- The rule table matches the default tick, production, demand, stock update, noise, price formula, and dump behavior. Provenance links resolve to the typed-format and map evidence; the document clearly labels this as a declared model, not recovered original behavior.
- `$HOME/tools/bin/make fmt`, `clippy`, and `test` passed. `make test` used `RT3_HOME` and the fixtures under `review-tmp/saves/`; the workspace tests and evidence check passed without drift.
- CI checks passed (`check` and GitGuardian Security Checks). No emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
