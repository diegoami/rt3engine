# PK4 reader implementation re-review

Revision: `14b2319fcb4fd6f521ef4738f5ca4fe9ef88df40`

Reviewer: GPT-6.2 Sol, model `gpt-6.2-sol#low`  
Mode: OpenCode

The local head matched the head reported by `gh pr view 4 --json headRefOid,baseRefOid,files`. The PR file list matched the local merge-base diff from `c5e853156cca1ab588a246a3df342624e5faa1b2` to that head.

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
- `reviews/01-pk4-reader-impl-02.md`
- `tools/evidence.sh`

This continues the fallback reviewer session selected by the coordinating Codex session for the preceding review. The selection is recorded in `reviews/01-pk4-reader-impl-02.md`.

## Resolution of prior findings

1. **Evidence gate.** The Makefile delegates to `tools/evidence.sh`, which retains a failing status when one artifact drifts and the other matches. With fake producers in a temporary checkout, clean output returned 0, drift in either artifact alone returned 1, and failure of either producer alone returned 2.
2. **Writer extraction.** `Pk4Archive::extract` streams to the supplied writer. The installed-archive test calls it for each entry and checks the byte count observed at the writer against the entry size. I exercised that test through all 8,375 entries of 18 synthetic archives, including zero-length entries and entries larger than a 64 KiB chunk. A separate temporary fixture confirmed exact bytes through a writer that accepts only 511 bytes per call and confirmed extraction returns errors for writer failure and post-open file truncation.
3. **Evidence regeneration.** A failed producer writes only to its own temporary file. With either producer failing after emitting partial output, the script returned 2 and both evidence files remained byte-for-byte intact when the other producer emitted the expected output.
4. **Explicit archive JSON.** The CLI emitted parseable JSON for two explicitly selected synthetic archives, preserving entry counts, sizes, and a name containing a quotation mark.
5. **Missing installation.** The previous missing-root reproduction exited 2 with an error on stderr and no empty summary on stdout.

No blocking findings remain.

## Verification

`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` passed from a temporary archive of the reviewed head. The workspace suite ran nine tests, with the installed-archive test self-skipping without `RT3_HOME`. A separate run enabled that test against the synthetic installation described above. This environment lacks `make` and an original installation. I ran the evidence script directly and relied on the implementer's recorded result for extraction against the original archives. GitHub's check workflow and GitGuardian succeeded on the reviewed head.

— GPT-6.2 Sol (gpt-6.2-sol#low), reviewer
AGREE

## Completion

Landed with merge commit `ab827b5` (owner-authorized delegated merge,
2026-11-22). Done-when items and their evidence:

- Issue #1 (format): `docs/evidence/pk4-format-v1.md` documents the PK4
  layout with Ghidra provenance (loader `0x0053b090`, lookup `0x0053b390`,
  seek `0x0053bd40`); independent byte validation of all 18 shipped
  archives; open questions recorded (hash algorithm, legacy versions, flag
  byte).
- Issue #2 (reader): `rt3-kit::pk4` open/parse/find/`extract`-to-writer
  with bounds validation; install-backed test extracts every shipped entry
  (18 archives, 8,375 entries) with byte-count assertions; `rt3ctl pk4
  list` CLI; aggregate evidence `pk4-archives-v1.json` checked by
  `tools/evidence.sh` (drift and producer-failure paths verified).
- Claim level achieved: **parse** (read side). No simulation, authoring or
  fidelity claim.

Post-landing follow-ups stay open: issue #3 (prior-art breadth), PK4
name→hash algorithm (needed only for authoring, milestone 13).

— Implementer (zai-coding-plan/glm-5.3)
