> Guidance for Claude Code. OpenCode uses [`AGENTS.md`](AGENTS.md); the shared
> principles and the verdict protocol are in [`PRINCIPLES.md`](PRINCIPLES.md).
> **Read it before implementing.**

# The Claude Code mode

This file records the Claude-specific process and the project slot.

## The process

- Claude **implements** the change on a branch and opens a pull request; each
  review round is posted on it as `PRINCIPLES.md` says (*Posting*, *Pull
  requests*), which also covers a project with no remote yet.
- The review is a **fresh-context session** — a new session that has not seen
  the implementation. **The reviewer is the same model family by default; no
  cross-family reviewer is required.** The mechanism may instead be an
  **external process** from another family (for example `codex exec`, or
  `opencode run -m <provider>/<model>`); when it is, record the tool and the
  model id in the review.
- There is **no design stage**, and a change's review carries **no AGREE/BLOCK
  marker**. The review is recorded per [`reviews/README.md`](reviews/README.md).
- **Milestones** follow `PRINCIPLES.md` (*Milestones*); their verdict is not a
  change's review and does carry the marker.
- **Fallback:** a new session, or the external process, recorded. The rules are
  in the protocol.
- The builder fixes findings in the same change; a finding the builder disagrees
  with goes to the owner, not around the reviewer.
- The **owner may review** as an independent option, but an owner is not
  automatically a fresh context — and is not one if they directed or wrote the
  change.
- The **owner merges** (`PRINCIPLES.md`), unless the project slot records
  `merge: auto`. The owner may also ask for a review by OpenCode's process
  instead, when a cross-family check is wanted.

Materiality, fallback, waiver and the defect path are in `PRINCIPLES.md`; the
bootstrap applies as written there — one review, not two stages.

## Project slot

<!-- SLOT:BEGIN -->

- **product:** rt3engine — An open, native stand-in engine for Railroad Tycoon 3 that plays the original game from the user's own installation assets, with headless simulation, observers, save/map compatibility, and an MCP interface for AI-assisted play.
- **premise:** product — the project's content is the deliverable.
- **roles:** recorded before the mode; the mode is the implementer's tool's.
  - **implementer:** OpenCode, model `opencode/deepseek-v4.1-flash`. The
    project works in OpenCode mode (`AGENTS.md`).
  - **reviewer of each change:** opencode/gpt-5.6-luna#high, as a subagent.
  - **reviewer of releases:** a model family that implemented none of the
    candidate; the owner picks one at each milestone (`PRINCIPLES.md`,
    *Milestones*).
- **paths to inspect:** the project's source roots and documents worth reading
  by default.
- **the canonical source:** the one place to read and edit; name any mirror,
  copy or generated artifact that must never be edited or cited.
- **paths to normally ignore:** generated, vendored or binary paths, each with
  its reason; read the lockfile only when dependencies are the task. Ignoring a
  path never means deleting or gitignoring it.
- **never read or echo:** secrets, signing material, one machine's paths. None
  are known; keep it that way and list them here when that changes. A path
  outside the repository is described relative to it, never absolutely.
- **merge:** owner
- **design:** none — OpenCode mode only (`AGENTS.md`, *The two
  stages*).
- **milestones:** annotated tags `vX.Y.Z` on `main`, as
  `PRINCIPLES.md` (*Milestones*) says; the plan that holds each release's
  claims: TBD — name the one file that will hold each release's claims.
- **the gates table:**

  | gate | command | covers | when | repeats | failure model |
  |---|---|---|---|---|---|
  | unit | `make test` | the project's tests | every change | 1 | deterministic tests; repeats when a flaky gate appears |
  | CI | `.github/workflows/check.yml` | the unit gate | pull requests and `main` pushes | as CI runs | a red CI blocks the merge |

- **conventions:** the player-facing language and the language of comments and
  commits; promises about build steps and dependencies; decided, and not to be
  re-opened; open work lives in `ROADMAP.md` when it exists.

<!-- SLOT:END -->
