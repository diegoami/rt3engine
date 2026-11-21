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

### Round 3 — owner-launched fallback review (posted verbatim on PR #4)

The coordinating owner session posted this review as a pull-request comment;
copied here unchanged because the file stays canonical. The fallback applies
because the assigned `opencode-go/gpt-6-luna#high` was unavailable in that
session's reviewer catalog (quota tracker refused the connection); the
fallback family differs from the implementer's.

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


### Response to round 3

— Implementer (zai-coding-plan/glm-5.3)

1. **blocking (Makefile masking): fixed.** Evidence recipes now delegate to
   `tools/evidence.sh`, which checks every artifact and accumulates
   failures: any drift or producer failure exits non-zero after reporting
   all of them. Verified by simulation: injected drift → exit 1 with the
   drift message; clean run → exit 0; failing producer (`CARGO=/bin/false`)
   → exit 2 with committed artifacts untouched (byte sizes unchanged).
2. **blocking (extraction acceptance): fixed.** `Pk4Archive::extract` now
   streams payloads to a caller-supplied writer (chunked reads, returns the
   byte count); `read` is a thin wrapper. The install-backed test extracts
   **every** shipped entry (all 18 archives, 8,375 entries, ~387 MB) through
   a counting writer and asserts written == table size (runtime ≈40 s with
   `RT3_HOME` set; still self-skipping without it).
3. **non-blocking (regen truncation): fixed** by the same script — producers
   write to a temporary file and only replace the artifact after success
   (verified with `CARGO=/bin/false`).
4. **non-blocking (`--archive --json` ignored): fixed.** Listing mode now
   emits JSON (`{"archives":[{"archive","version","entries":[{"name","size"}]}]}`
   in table order) and the human listing otherwise.
5. **non-blocking (invalid root silently succeeds): fixed.** A resolved root
   that is not a directory now reports an error and exits 2 (verified with
   `--path /nonexistent`, with and without `RT3_HOME`).

Gates after the fixes: `cargo test --workspace` (9 tests; install-backed
extraction included), `cargo fmt --all --check`, `cargo clippy
--workspace --all-targets -- -D warnings` all pass.
