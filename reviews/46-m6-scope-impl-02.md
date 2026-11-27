# Implementation review — PR #64 (round 2)

- **Revision:** `cb1e63304f6fd781bc9ebed0c1c460db4e3199ed`
- **Pull request/base:** [#64](https://github.com/diegoami/rt3engine/pull/64),
  base `ac10549f083f5c2313101bcfbdaf884f16cd79f1`
- **Files checked (PR file list):**
  - `docs/milestones.md`
  - `reviews/46-m6-scope-impl-01.md`
- **How obtained:** `gh pr view 64 --json headRefOid,baseRefOid,files` reports
  this head and base; local `HEAD` equals the head, and
  `git merge-base main cb1e63304f6fd781bc9ebed0c1c460db4e3199ed` equals the
  reported base. The full base-to-head file list matches the PR list. The
  delta from the round-one head `38cf371` to this revision is only
  `docs/milestones.md`; the other listed file is the already-committed
  round-one review record.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review; the project slot records
  `design: none` and merge delegated on AGREE.

## Round 1 findings

1. **Resolved — AI competitor v0 is included.** C6 adds a declared scripted
   competitor; the not-in-release list excludes behavior beyond that policy,
   and the gate covers C1–C6 (`docs/milestones.md:129-147`).
2. **Resolved — C1 no longer claims a station-to-path link.** It explicitly
   leaves matching path ends to station records open, and C5 says it uses the
   decoded track, not a claimed station link (`docs/milestones.md:95-104,122-128`).
3. **Resolved — the structural readers are prior baseline, not a claim.** C1
   now scopes the claim to the graph decoder and its evidence
   (`docs/milestones.md:95-104`).

## Verification

- The ladder tasks are all represented: graph (C1), movement (C2), consists
  and schedules (C3), commands/order log/checkpoints (C4), and AI competitor
  v0 (C6).
- C6 is explicitly a declared scripted policy, not recovered original AI; its
  engine test, replay hash and future evidence artifact can verify that policy.
  C5's declared revenue/cost model can be tested on the decoded track without
  asserting a station link.
- The promise, not-in-release list and gate otherwise agree with C1–C6. The
  evidence documents named in the claims remain future outputs.
- `make fmt` is not available on this shell's PATH. `$HOME/tools/bin/make fmt`
  passed (`cargo fmt --all -- --check`). PR CI `check` and GitGuardian Security
  Checks passed on this revision. No emulator was booted.

## Findings

1. **Blocking — the promise adds a cash-flow-against-AI result that no claim
   checks** (`docs/milestones.md:89-90,122-134,145-147`). The promise says the
   scripted company reaches positive cash flow “against” the AI competitor.
   C5 checks cash flow on the fixture track without a competitor; C6 checks a
   competitor separately on a synthetic scenario. The gate also treats cash
   flow and deterministic AI as separate results. No named check establishes
   positive cash flow while the competitor is operating. Either remove
   “against a declared AI competitor v0” from the promise, or add a joint
   scripted run and assert its cash-flow result with the competitor active.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
