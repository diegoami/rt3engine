# rt3engine

Scaffolded from the harness at `r5.1` (1dd8b54e29fb), preset `standard`.

Read [`AGENTS.md`](AGENTS.md) if you work with OpenCode, or
[`CLAUDE.md`](CLAUDE.md) if you work with Claude Code. Both read
[`PRINCIPLES.md`](PRINCIPLES.md).

## Policy

- **mode:** OpenCode mode — the implementer's tool is OpenCode (the project slot in `CLAUDE.md`).
- **merge:** `owner` — the owner merges.
- **design:** `none` — no design stage; the implementation review alone decides.

## First session

1. **Owner action — the remote.** Create it before the first iteration:

   ```sh
   gh repo create <owner>/rt3engine --private --source . --push   # or --public
   ```

   A done-when that names CI is not met until the workflow has run green.
