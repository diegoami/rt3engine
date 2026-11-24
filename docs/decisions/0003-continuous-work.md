# Decision 0003 — continuous ordinary-work loop

Date: 2026-11-24. Owner: diegoami. Status: decided.

The owner instructed the implementer in conversation: "continue. Do not stop
at every ticket until you need a review for a milestone or something from the
owner. Save this as a rule."

This clarifies decision 0002's autonomous change loop:

1. **Continue after each landed ticket.** Once an ordinary change has its
   independent AGREE, green gates, merge and completion bookkeeping, select
   the next open task and continue without waiting for a per-ticket owner
   prompt.
2. **Preserve the per-change controls.** Every non-trivial change still gets
   its required independent review, fixes, green gates and reviewed merge.
   This decision does not weaken evidence discipline or authorize skipping
   pull requests, reviews, CI or capture coordination.
3. **Pause at real decision points.** Stop for a milestone/release review or
   tag, a human-acceptance gate, an owner decision or resource, an escalation
   required by the verdict protocol, or anything unsafe/otherwise blocked.
4. **Reporting is not stopping.** Progress reports after a ticket or review
   are optional; they must not become requests for permission to continue
   ordinary work.
