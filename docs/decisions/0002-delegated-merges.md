# Decision 0002 — delegated merges and autonomous change loop

Date: 2026-11-22. Owner: diegoami. Status: decided.

The owner authorized the implementer (the main OpenCode session) to run the
per-change loop autonomously:

1. **Merge delegation.** Once a change's independent review ends in AGREE,
   the implementer merges the pull request with a merge commit (preserving
   reviewed SHAs) and records the completion note. `PRINCIPLES.md`'s
   owner-merge rule is satisfied by this standing owner decision; the
   project slot records it.
2. **Scope.** This covers ordinary milestone work: implementation, review
   rounds, fixes, merges, issue bookkeeping and evidence.
3. **What stays with the owner.** Milestone/release reviews (owner-launched
   per `PRINCIPLES.md`), tags, human-acceptance gates named in
   `docs/milestones.md` (e.g. observing the observer run, playing the
   client slice), new owner decisions, and anything a BLOCK escalates.

Consequence: progress runs at review cadence until a milestone boundary;
the owner is notified at each boundary and for any decision.
