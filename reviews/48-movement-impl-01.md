# Implementation review — PR #66 (round 1)

- **Revision:** `3b65f3030c98f77b1586dd6c49bd33e7ccd7c9e8`
- **Pull request/base:** [#66](https://github.com/diegoami/rt3engine/pull/66),
  base `4171bf77ae9b0d002fa6ea2a770f155a37a08504`
- **Files checked (PR file list):**
  - `crates/engine/src/lib.rs`
  - `crates/engine/src/movement.rs`
  - `crates/engine/tests/movement.rs`
  - `docs/evidence/README.md`
  - `docs/evidence/movement-v1.md`
- **How obtained:** `gh pr view 66 --json headRefOid,baseRefOid,files`
  reports this head, base and file list; local `HEAD` equals the reported head.
  `git merge-base main 3b65f3030c98f77b1586dd6c49bd33e7ccd7c9e8` equals the
  reported base, and `git diff --name-only <base>..<head>` matches the PR list.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review; the project slot records
  `design: none` and merge delegated on AGREE.

## Verification

- Independently decoded the fixture's 61 track origins at blob `+12` (fixture
  SHA-256 begins `2a8f6e72`). The declared graph gives runs `0..5`, `5..10`,
  `10..61`, joins `4↔10` and `60↔5`, and path order
  `0,1,2,3,4,10…60,5,6,7,8,9`. The independently summed path length is
  `2054.775803` units; applying the declared one-second, 12-unit/s,
  2-unit/s² model reaches the end in 191 ticks. The fixture test passed and
  asserts the stated golden SHA-256.
- Temporarily lowering `MAX_SPEED` to `6.0` made the fixture test fail at its
  golden-hash assertion (`df17939a…5434` vs `9bbdfd39…cf6`). Restored `12.0`;
  the fixture test passed and the working tree was clean.
- `$HOME/tools/bin/make fmt`, `clippy`, `test` and `evidence-check` passed with
  `RT3_HOME` and all save fixtures supplied under repo-relative
  `review-tmp/saves/`; no fixture or evidence check skipped. Workspace tests
  passed: 5 economy, 5 movement, 4 world, 68 `rt3-kit`, 10 `rt3ctl`, 1
  `sim_compare` and 2 `sim_run`.
- PR CI `check` and GitGuardian Security Checks passed. No emulator was booted.
- The change implements C2 only. C3–C6 are not started; no M6 version is
  assigned. The M6 claims predate this implementation (PR #64). Neither a
  station-to-path link nor original train physics is claimed. The evidence
  reproduction uses the repo-relative fixture convention and contains no
  original save bytes, installed-save content or machine-specific absolute
  path.

## Findings

None.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

- M6 C2 lands: `engine::movement` derives the ordered path over the declared
  track graph and runs a consist under the declared fixed-tick speed,
  acceleration and braking model. On the private fixture it covers 2,054.776
  units in 191 ticks with the golden state hash; replay is byte-identical and
  the model has no randomness.
- PR #66 merged as `e5c9e21`. C3–C6 are not started; no version is assigned.
- Reviews: `reviews/48-movement-impl-01.md` (AGREE, no findings). Gates and CI
  green.

— Implementer (zai-coding-plan/glm-5.3)
