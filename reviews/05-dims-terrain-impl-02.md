# Implementation review: dims-driven terrain scan (round 2)

- **Revision covered:** `90266b63bc3c76ce4d5a2d7b86c5cdb2cd78d091`
- **Files checked (full PR):** `crates/rt3-kit/src/terrain.rs`, `crates/rt3ctl/src/map_cmd.rs`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/terrain-scan-v1.json`, `reviews/04-terrain-framing-impl-03.md`, `reviews/05-dims-terrain-impl-01.md`.
- **How obtained:** Inspected the round-2 delta with `git diff f954f64..HEAD --stat`, `git diff f954f64..HEAD --name-only`, and `git diff f954f64..HEAD`. Full PR merge base was `9791d1e59122c3e5eab99e800ec7292df8bebbb2` (`git merge-base main HEAD`); inspected its `--stat`, `--name-only`, and full diff through `HEAD`. The checked-out `HEAD` matched the revision covered. Read all files in the full-PR list at `HEAD`.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Round-1 findings rechecked

1. **Resolved.** `scan_with_dims` evaluates both non-square orientations and refuses a result when both have hits; `ambiguous_orientation_is_refused` exercises that case.
2. **Resolved.** Both scan entry points reject unsupported geometries. The 33-cell-per-side floor keeps the probe and prefilter samples within the validator lattice (derivation below).
3. **Addressed, with a wording precision noted in finding 2 below.** The exact-size claim is removed; the evidence now gives the measured span and grid extents and notes the 4 KiB windowing.
4. **Resolved.** The evidence describes the alternate storage form as unresolved.
5. **Resolved.** The JSON grid objects include `rows` and `cols`.

## Findings

1. **blocking** — `docs/evidence/gmp-terrain-v1.md:95-96`. The document says “32 of 40 maps have no verified terrain decode,” but this revision has 5 verified maps, so 35 of 40 lack a verified decode (`40 − 5 = 35`; equivalently, 7 candidate windows plus 28 maps with no validated window). The “32” is stale from the previous count of 8 verified maps. Update the count so the evidence document agrees with the artifact and its own lists.

2. **non-blocking** — `docs/evidence/gmp-terrain-v1.md:75-78`. The measured interval `[0x8000..0x10a000)` ends 452 bytes before the scanned grid’s exclusive end, `0x10a1c4`; literally, it does not “bracket” the grid. The 4 KiB-windowing qualification makes clear the difference is within the stated resolution, and the revision no longer claims equal spans. For precision, describe the intervals as overlapping to within that resolution rather than saying the measured interval brackets the grid.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' ~/.cargo/bin/cargo test -p rt3-kit terrain -- --nocapture` — 11 passed, 0 failed.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed with no output or drift.
- Parsed `terrain-scan-v1.json`: 40 maps; 5 verified, 7 candidates and 28 with no grid. Verified list: Crossing the Alps, France, Germany, State of Germany, Third Republic. Candidate list: Alternate USA, Central Pacific, Chicago to New York, Dutchlantis, Orient Express, Southern Pacific, Tex-Mex. The five header-bearing maps with no grid and both lists match the document. All 12 grid hits contain `rows` and `cols`. The document’s “32 of 40” count is the exception recorded above.
- Re-derived the subset property from `step_of`: probe steps are `4·floor(d/32)` and probe multipliers are 0–7; prefilter steps are `4·floor(d/16)` and multipliers are 0–3. For every supported axis length `d ≥ 33`, these positions are multiples of 4 and at most `d−1`, hence belong to the validator’s 4-step lattice. For rows=129/cols=513, probe steps are 16/64 (max samples 112/448, validator maxima 128/512); prefilter steps are 32/128 (max samples 96/384).
- The release Germany CLI exited successfully and reported `grid at 0x89bc (513x513, ...) [verified]`.
- No game or emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
