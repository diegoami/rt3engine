# Implementation review: M3 `.gmp` map prelude

- **Revision covered:** `351d4a32081eb48f2044d979d07a884db7564949`
- **Files checked:** `crates/rt3-kit/src/lib.rs`, `crates/rt3-kit/src/map.rs`, `crates/rt3ctl/src/main.rs`, `crates/rt3ctl/src/map_cmd.rs`, `docs/evidence/README.md`, `docs/evidence/gmp-format-v1.md`, `docs/evidence/maps-inventory-v1.json`, `tools/evidence.sh`.
- **How obtained:** local diff from merge-base `5ad99481ce7c11806d82d56127a452ebda97a719` (`git merge-base main m3/map-prelude`, then `git diff --name-only <merge-base>..m3/map-prelude`); checked-out `HEAD` matched the covered revision.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high`
- **Mode:** OpenCode

## Findings

1. **blocking** — `docs/evidence/gmp-format-v1.md:41-44`. The corrected-claims paragraph says `u32@file+0x3008` is zero in every shipped map, and uses that as evidence that there is no fixed terrain-size field there. An independent Python read of all 43 installed `.gmp` files found that this word is nonzero in every file: values were `0x408000` (36 maps), `0x1408000` (3), `0x2408000` (3), and `0x3408000` (1). In particular, `0x408000` is 4,227,072. This falsifies the stated byte observation; it does not by itself establish the word's semantics. Reconcile the offset/value with the loader evidence, and either substantiate the “no fixed terrain-size field” correction independently or mark that conclusion unverified/open rather than grounding it in a zero value.

## Verification

- `make test fmt clippy` passed with the requested `PATH` and `RT3_HOME`. The output showed `map::tests::parses_every_shipped_map ... ok`; that test asserts 43 maps and `(40, 3)` current/legacy counts, so the install-backed check ran. The evidence check, formatting check, and warning-denied Clippy check also passed.
- Independent Python spot-checks against `$RT3_HOME` confirmed 43 maps with versions 9701 ×40, 9609 ×2, and 9610 ×1; a 20,268-byte current record; NUL-terminated text at file offset `4+0x78` for all 40 current maps; and the stated tail-offset calculation `4+20268+4+2*3001 = 26278`.
- The format note keeps the claim level at **parse (prelude only)** and leaves tail grammar open. The changed evidence JSON contains map names and metadata (versions, statuses, text lengths, file/tail sizes), not original map prose or bytes.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK

### Response to round 1

— Implementer (zai-coding-plan/glm-5.3)

1. **blocking (evidence-doc offset claim): fixed.** My original check read
   record+0x3008 (file 0x300C — always 0) while the doc said file+0x3008.
   Re-verified both: file+0x3008 is nonzero in all 43 maps (4,227,072 in 36;
   other size-like values in the rest) and is a genuine per-map field with
   open semantics; record+0x3008 is 0 everywhere. The "Corrected claims"
   section now states both offset bases explicitly.
