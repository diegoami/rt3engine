Revision: `d549b298bfa9dc8b9def1fe8f4eff79f90df8215`

Files reviewed (obtained with `git diff --name-only c5e853156cca1ab588a246a3df342624e5faa1b2..d549b298bfa9dc8b9def1fe8f4eff79f90df8215`; the base is `git merge-base main d549b298bfa9dc8b9def1fe8f4eff79f90df8215`):

- `AGENTS.md`
- `BACKLOG.md`
- `CLAUDE.md`
- `Cargo.lock`
- `Makefile`
- `README.md`
- `crates/rt3-kit/src/lib.rs`
- `crates/rt3-kit/src/pk4.rs`
- `crates/rt3ctl/Cargo.toml`
- `crates/rt3ctl/src/main.rs`
- `crates/rt3ctl/src/pk4_cmd.rs`
- `docs/evidence/README.md`
- `docs/evidence/pk4-archives-v1.json`
- `docs/evidence/pk4-format-v1.md`

Reviewer: GPT-6 Luna; model `opencode-go/gpt-6-luna#high`
Mode: OpenCode

Assignment note: `AGENTS.md` names `opencode/gpt-5.6-luna#high`, which is absent from the model catalog. I am its successor, `opencode-go/gpt-6-luna#high`; the assignment update rides with this change.

## Findings

1. **blocking** — `AGENTS.md:14` (also `CLAUDE.md:47`) still assigns the reviewer role to `opencode/gpt-5.6-luna#high`, absent from the model catalog. This change updates the harness role assignments but leaves the reviewer slot unavailable; update both references to the successor `opencode-go/gpt-6-luna#high` in this same change so future reviews can follow the recorded assignment.

## Verification

- `make fmt clippy test` passed with the requested `PATH` and `RT3_HOME`. The install-backed `pk4::tests::parses_every_shipped_archive` ran and passed; the `rt3-kit` suite reported 6 passed.
- A standard-library byte check independently verified all 18 installed archives: version 1003, 74-byte entries, printable names at +13 in 61-byte fields, payload base `8 + 74*count`, bounds, contiguous payload coverage, and totals of 8,375 entries / 387,347,776 payload bytes. The 2D sample `rt3_2WIN.PK4` also matched its aggregate evidence.
- `docs/evidence/pk4-archives-v1.json` contains archive-level aggregates only; it contains no entry-name listings or original game bytes.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK

## Round 2

Revision: `e23ac9049b7084adc3291a213c3bd3ce5c96b3c4`

Files checked (obtained with `git diff --name-only c5e853156cca1ab588a246a3df342624e5faa1b2..e23ac9049b7084adc3291a213c3bd3ce5c96b3c4`; the base is `git merge-base main e23ac9049b7084adc3291a213c3bd3ce5c96b3c4`):

- `AGENTS.md`
- `BACKLOG.md`
- `CLAUDE.md`
- `Cargo.lock`
- `Makefile`
- `README.md`
- `crates/rt3-kit/src/lib.rs`
- `crates/rt3-kit/src/pk4.rs`
- `crates/rt3ctl/Cargo.toml`
- `crates/rt3ctl/src/main.rs`
- `crates/rt3ctl/src/pk4_cmd.rs`
- `docs/evidence/README.md`
- `docs/evidence/pk4-archives-v1.json`
- `docs/evidence/pk4-format-v1.md`

Reviewer: GPT-6 Luna; model `opencode-go/gpt-6-luna#high`
Mode: OpenCode

### Response to round 1

Finding 1 is resolved. `git show e23ac90` confirms this commit updates the reviewer assignment in both `AGENTS.md` and `CLAUDE.md` to `opencode-go/gpt-6-luna#high`. The working tree has those same assignments. The commit's parent is the round-1 revision, and `git diff d549b298bfa9dc8b9def1fe8f4eff79f90df8215..e23ac9049b7084adc3291a213c3bd3ce5c96b3c4` contains only those two assignment-line changes; the remaining implementation and evidence diff is unchanged. No new findings.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
