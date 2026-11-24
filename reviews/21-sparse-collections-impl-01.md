# Implementation review — sparse collections

- **Revision:** `792b858616a5dc8245c03aeb5d06b9916bc0dec2`
- **Base:** `46fb4b33bfd68b2736346bd0a77c43adf7a0cecc`
- **Scope proof:** `gh pr view 28 --json headRefOid,baseRefOid,files`; the reported head equals local `HEAD`, and its file list is:
  - `crates/rt3-kit/src/buildings.rs`
  - `crates/rt3-kit/src/cities.rs`
  - `crates/rt3-kit/src/events.rs`
  - `crates/rt3-kit/src/lib.rs`
  - `crates/rt3-kit/src/sparse.rs`
  - `crates/rt3ctl/src/map_cmd.rs`
  - `docs/evidence/README.md`
  - `docs/evidence/gmp-cities-v1.md`
  - `docs/evidence/gmp-collections-v1.md`
  - `docs/evidence/gmp-format-v1.md`
  - `docs/evidence/terrain-scan-v1.json`
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high`
- **Mode:** OpenCode; project slot records `design: none` and delegated merge.

## Findings

1. **non-blocking** — `crates/rt3-kit/src/cities.rs:171-173` and `crates/rt3-kit/src/events.rs:39-67`. The city truncation loop discards every `read` result, so it detects panics but does not assert that a strict truncation is rejected. The new shared sparse reader has positive fixture coverage, but no direct malformed/truncated rejection test. The bounded reads and count/length/terminator checks are present in the implementation, and shipped-map checks pass; a negative test asserting `None` would make the malformed-input guarantee explicit.

## Review

The supplied disassembly supports the building provenance: `0x00443a50` dispatches tag `0x3315` to `0x00413280`; that reader brackets `0x36b1` / `0x36b2`, then performs per-record virtual reads. `0x00518680` confirms four level triples, reads `capacity × 12` bytes for each populated later level, and processes a capacity-sized absence mask. The mask direction and six-byte prefix / 16-bit length / opaque blob sequence were independently checked by walking the original map bytes rather than relying on the implementation.

The loader dump shows the event call at `0x00444959` into `0x00433130`, but an event-reader dump was not available in the supplied disassembly files. The event preamble, layout and exact end were independently checked in the shipped maps. Post-`0x4e9a` virtual-read details and gameplay semantics remain open; this review does not infer them.

An independent read-only byte walk of all 43 maps verified the common headers, level payload skips, absence masks, active slots, prefixes, exact blob sizes and terminators, and matched each per-map summary in the artifact. Totals were buildings **2,450 slots / 2,044 records / 18 nonempty maps** and events **1,810 slots / 1,169 records**. Building labels were nonempty with no leading control byte; event names were all nonempty. All 37 raw José/Martí Latin-1 names occur in Spanish Mainline. Event text begins at offset 224; 262 records have an empty text prefix and all scanned text remainders terminate with NUL. No string conversion or text semantics were assumed. Removing the new building/event fields from each artifact row yields exact equality with the merge-base artifact's prior fields.

Parser review: capacity is capped at 1,024 before allocation; record and blob reservations are fallible; level payload multiplication and cursor advances are checked; mask and record reads are bounds-checked; excess active slots, mismatched counts, unexpected blob lengths and missing terminators return `None`. The city follow-up borrows the mask, guards active-count growth, and retains exact checks for count, length and terminator. No regression was observed in the install-backed city check.

## Verification

- With the documented tools directory on `PATH`, `make fmt` — passed.
- With the documented tools directory on `PATH`, `make clippy` — passed with warnings denied.
- With `RT3_HOME` set to the documented installation root, `make test` — passed: **50 rt3-kit tests**, **5 rt3ctl tests**; the install-backed building, event and city checks ran.
- With the same `RT3_HOME`, `make evidence-check` — passed with no artifact drift.
- Independent read-only all-43-map reconciliation — passed, as detailed above.
- No emulator or original-game runtime was used.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
