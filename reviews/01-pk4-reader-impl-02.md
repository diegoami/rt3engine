# PK4 reader implementation review

Revision: `507074017e35e9b4deec926d22ab1fc53d2265b0`

Reviewer: GPT-6.2 Sol; model `gpt-6.2-sol#low`  
Mode: OpenCode

The head held locally matches GitHub PR #4. The file list from `gh pr view 4 --json files` matches the local merge-base diff from `c5e853156cca1ab588a246a3df342624e5faa1b2` to that head:

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
- `docs/prior-art.md`
- `reviews/01-pk4-reader-impl-01.md`

The coordinating Codex session selected a fresh GPT-6.2 Sol subagent as the fallback. The assigned `opencode-go/gpt-6-luna#high` was unavailable in the session's reviewer catalog, and the quota tracker refused the connection. The fallback is from a different model family than GLM-5.3. The existing review file ends in AGREE for `e23ac9049b7084adc3291a213c3bd3ce5c96b3c4`; this review covers the current PR head.

## Findings

1. **blocking** — `Makefile:28`–`31` masks an inventory evidence failure when the PK4 diff passes. The first `exit 1` exits only its subshell and the second diff supplies the recipe's final status. I ran the base and head recipes verbatim through `/bin/sh` in a temporary directory with a fake Cargo that emitted a changed inventory and the expected PK4 JSON. Both printed the inventory drift message. The base returned 1 and the PR head returned 0. Propagate failures from either producer or comparison so `make test` fails when either artifact drifts.
2. **blocking** — `crates/rt3-kit/src/pk4.rs:259`–`274` and `:383`–`:415` leave extraction acceptance in [issue #2](https://github.com/diegoami/rt3engine/issues/2) incomplete. The API reads into a newly allocated vector and has no extraction path to a caller-supplied writer. The install-backed test only opens the archives and counts entries, so it passes even if reading payloads at their offsets fails. The synthetic disk test exercises two entries. Provide extraction to a caller-supplied writer and make the install-backed check exercise that path for every shipped entry and compare the extracted size with its table size. A counting writer can keep the game files read-only and avoid writing copies of the payloads.
3. **non-blocking** — `Makefile:20`–`23` reports successful regeneration after a failed producer, after shell redirection has already truncated the output. In a temporary copy, substituting `false` for Cargo returned status 0, printed the success message and reduced both artifacts to zero bytes. The existing inventory recipe has the same failure mode, and the new recipe extends it to the PK4 artifact. Generate into temporary files and replace each artifact only after its producer succeeds.
4. **non-blocking** — `crates/rt3ctl/src/pk4_cmd.rs:167`–`178` ignores the requested output format for the combination shown in `crates/rt3ctl/src/main.rs:27`. Running `rt3ctl pk4 list --archive fixture.pk4 --json` on a one-entry synthetic v1003 archive printed the human listing and exited successfully. Support JSON in that mode or reject the combination and state the supported forms in usage.
5. **non-blocking** — `crates/rt3ctl/src/pk4_cmd.rs:90`–`109` silently drops directory discovery errors. Running `rt3ctl pk4 list --path missing --json` printed a successful zero-archive summary for a nonexistent root. Return an error for an invalid installation root so callers can distinguish it from a successful scan.

## Verification

- Executed the Cargo equivalents of the Rust test, formatting and Clippy gates against a temporary directory populated by `git archive` of the reviewed head. Formatting and Clippy passed. All nine workspace tests reported passing, with the install-backed PK4 test self-skipping because `RT3_HOME` was unset. The synthetic fixtures exercise parsing, version and bounds rejection, case-insensitive lookup, payload offsets and bytes. The full installed payload set and Ghidra findings could not be independently checked in this session.
- `make` was unavailable. The two evidence recipe reproductions used the actual recipe bodies with substituted variables and ran only against temporary files.
- Parsed the committed PK4 aggregate evidence and recomputed 18 archives, 8,375 entries and 387,347,776 bytes from its per-archive rows. This checks internal consistency. It does not regenerate evidence from an installation.
- GitHub reported its check workflow and GitGuardian checks successful on the reviewed head. Its workflow also skips installation-backed evidence without `RT3_HOME`.

— GPT-6.2 Sol (gpt-6.2-sol#low), reviewer
BLOCK
