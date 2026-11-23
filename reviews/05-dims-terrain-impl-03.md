# Implementation review: dims-driven terrain scan (round 3)

- **Revision covered:** `a03f731fc12af78296e53de65023e96e3a971a2b`
- **Files checked (full PR #12):** `crates/rt3-kit/src/terrain.rs`, `crates/rt3ctl/src/map_cmd.rs`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/terrain-scan-v1.json`, `reviews/04-terrain-framing-impl-03.md`, `reviews/05-dims-terrain-impl-01.md`, `reviews/05-dims-terrain-impl-02.md`.
- **How obtained:** `gh pr view 12 --json headRefOid,files` reported this revision and the seven files listed above. `git merge-base main HEAD` was `9791d1e59122c3e5eab99e800ec7292df8bebbb2`; `git diff --name-only <merge-base>..HEAD` matched the PR file list. The checked-out `HEAD` matched the revision covered. Inspected `git diff 90266b6..HEAD --stat`, `--name-only`, and the full delta; it changes only `docs/evidence/gmp-terrain-v1.md` and adds the round-2 record. Read the changed document and record, the artifact, prior review records, and spot-checked the unchanged scanner and CLI fixes at `HEAD`.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Round-2 findings rechecked

1. **Resolved.** The artifact has 40 maps: 5 verified, 7 candidates, and 28 with no grid. The verified list is Crossing the Alps, France, Germany, State of Germany, and Third Republic. All 12 grid hits have `rows` and `cols`; 13 maps have dims headers. Thus 35 of 40 lack a verified decode, matching the document.
2. **Resolved.** The Germany span sentence now describes the grid's final 452 bytes as outside the measured span. Independently, `0x10a000 - 0x89bc = 0x101644 = 1,054,276`, while `513 × 2,056 = 1,054,728`, a 452-byte difference; `0x10a1c4 - 0x10a000 = 0x1c4 = 452`. The final bytes fall in the first 4 KiB window past the span.

## Round-1 fixes spot-checked

1. **Resolved.** `scan_with_dims` still evaluates both orientations for non-square lattices and returns no result if both validate; `ambiguous_orientation_is_refused` remains present.
2. **Resolved.** The scanner still rejects geometries below the 33-cell minimum; `rejects_unsupported_small_geometry` remains present.
3. **Resolved.** The documentation uses the measured Germany span and grid bounds rather than claiming equal extents.
4. **Resolved.** The alternate storage form remains explicitly unresolved.
5. **Resolved.** The JSON artifact retains `rows` and `cols` for every grid hit.

## Findings

No findings.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME`-backed `~/.cargo/bin/cargo test -p rt3-kit terrain` — 11 passed, 0 failed.
- `RT3_HOME`-backed `CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed with no drift or producer errors.
- No game or emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
