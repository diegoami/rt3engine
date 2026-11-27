# Implementation review — PR #64 (round 1)

- **Revision:** `713848645642813d1add4c9ca31cd99f249afb07`
- **Pull request/base:** [#64](https://github.com/diegoami/rt3engine/pull/64),
  base `ac10549f083f5c2313101bcfbdaf884f16cd79f1`
- **Files checked (PR file list):**
  - `docs/milestones.md`
- **How obtained:** `gh pr view 64 --json headRefOid,baseRefOid,files` reported
  this head, base and one file. Local `HEAD` equals the reported head;
  `git merge-base main 713848645642813d1add4c9ca31cd99f249afb07` equals the
  reported base, and `git diff --name-only <base>..<head>` matches the PR list.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review; the project slot records
  `design: none` and merge delegated on AGREE.

## Verification

- `$HOME/tools/bin/make fmt` passed (`cargo fmt --all -- --check`). `make` is
  not on this shell's PATH.
- PR CI `check` and GitGuardian Security Checks passed on the reviewed head.
- No emulator was booted. The evidence documents named by the proposal are not
  present at this revision and are treated as future outputs.
- The PR body states this is a proposal, that the owner calls the milestone,
  and that no version is assigned.

## Findings

1. **Blocking — the proposal drops AI competitor v0 from the ladder**
   (`docs/milestones.md:18,128-134`). The milestone 6 ladder row explicitly
   includes “AI competitor v0,” while the proposal excludes “AI companies and
   competitor behavior” and has no corresponding claim or gate. The proposal
   therefore does not match the ladder. Include a testable AI competitor v0
   scope, or record an owner-approved ladder re-scope before treating this
   proposal as the milestone scope.

2. **Blocking — C1 claims a station-to-path relationship the decode has not
   established** (`docs/milestones.md:95-102`). The declared origin-step rule
   can define a path among track pieces, and the collection can report the
   station names; neither proves that the path's ends connect to Bath and
   Gloucester. Issue #62 still leaves matching the first/last pieces to those
   station positions as future work. C1's named checks do not specify an
   endpoint-to-station check. Narrow the claim to the declared track-piece
   graph, or state and test an explicit, non-decoded fixture-labeling rule;
   do not present the station linkage as a decoded game semantic.

3. **Blocking — C1 fixes a claim after its structural-reader work landed**
   (`docs/milestones.md:94-100`). The claim includes parsing all five rail
   sections, but PR #63 had already implemented and tested those readers on
   the populated fixture before this scope was recorded. PR #63 is after the
   v0.6.0 candidate and would be in the next milestone range. Under
   `PRINCIPLES.md` (*Milestones*), claims must be fixed before their work.
   Re-scope the already-completed parsing as prior reconnaissance/baseline, so
   the fixed claims cover work that has not started.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
