# Implementation review: dims-driven terrain scan

- **Revision covered:** `f954f64f22fa09bd694dd64c4abbf596bfc4958b`
- **Files checked:** `crates/rt3-kit/src/terrain.rs`, `crates/rt3ctl/src/map_cmd.rs`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/terrain-scan-v1.json`, `reviews/04-terrain-framing-impl-03.md`.
- **How obtained:** `git diff main...HEAD --stat`, `git diff main...HEAD --name-only`, and `git diff main...HEAD`; merge base was `9791d1e59122c3e5eab99e800ec7292df8bebbb2`. Checked-out `HEAD` matched the revision covered. Read each changed file at `HEAD`.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Findings

1. **blocking** — `crates/rt3-kit/src/terrain.rs:419-428`; `crates/rt3ctl/src/map_cmd.rs:207-214`. `scan_with_dims` returns as soon as the first orientation produces any heuristic hit; it tries the transposed geometry only if the first returns `None`. I reproduced a wrong-orientation hit using a smooth 128×512 fixture stored transposed for dims `(512,128)`: direct `scan_geo` with the actual `(rows, cols) = (513,129)` geometry found the fixture at its start, but `scan_with_dims` returned that same offset as `(129,513)` because the wrong row pitch also passed validation. `scan_map` would mark that result `verified`. Evaluate both orientations and disambiguate competing hits (or report ambiguity), with a regression test.

2. **blocking** — `crates/rt3-kit/src/terrain.rs:187-201,217-229,237-249`. The step rounding does not keep probe positions in the validator lattice for every accepted geometry: for lattice 27, `rows = cols = 28`, `step_of(28, 8)` rounds to zero and `.max(4)` makes the probe sample coordinate 28, while validation samples only through 24. A temporary test against this source put an out-of-range value in the trailing cell at `(0,28)`; `validate` accepted the window, but the probe rejected it and `scan_geo` missed it. `read_dims` only requires the lattice fields to be nonzero (and at most 8192), so this geometry is not excluded by the header parser. Keep every probe/prefilter coordinate within the same-origin validator lattice, or reject unsupported dimensions explicitly, and retain a regression.

3. **blocking** — `docs/evidence/gmp-terrain-v1.md:65-71`; `crates/rt3-kit/src/terrain.rs:54-78`; `docs/evidence/terrain-scan-v1.json:19`. The Germany span is not the exact geometry claimed: `0x10a000 - 0x8000 = 1,056,768`, whereas `513 × 2,056 = 1,054,728` (the `Geometry` formula for a 512 lattice), a difference of 2,040 bytes. From the artifact's grid offset `0x89bc`, that geometry ends at `0x10a1c4`, 452 bytes after the cited span end. Correct the span/bounds and its method, or narrow the claim; as written it does not provide the stated exact second-map span corroboration.

4. **non-blocking** — `docs/evidence/gmp-terrain-v1.md:86-93`. Five header-bearing maps with no validated window establish only that no window passed these scanner gates; they do not prove the maps use another storage form. The categorical “which means the raw-row layout is not the only storage form” conflicts with the following qualification that the checks establish only what was not found. Rephrase the inference as unresolved.

5. **non-blocking** — `crates/rt3ctl/src/map_cmd.rs:251-255,284-300`; `docs/evidence/gmp-terrain-v1.md:60-61`. The library `HeightGrid` retains `rows` and `cols`, but `run_terrain_all` drops them when building the summary tuple, so the JSON artifact does not record the geometry/orientation used. The prose says the hit records that geometry; either include the dimensions in the machine-readable result or scope that statement to the library result / single-map CLI.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME`-backed `~/.cargo/bin/cargo test -p rt3-kit terrain -- --nocapture` — 9 passed, 0 failed.
- `RT3_HOME`-backed `CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed with no drift or producer errors.
- `terrain-scan-v1.json` parses: 40 maps, 8 verified, 4 candidates, 28 with no grid; the five header-bearing misses and verified/candidate lists match the artifact.
- The release Germany CLI output reports `513x513` and `[verified]`. The requested `| head -5` pipeline printed the expected lines, then the CLI reported a broken pipe when `head` closed stdout; the same release command without the pipe exited successfully.
- The PR #11 malformed cases (`--all --bogus`, `--all <name>`, missing `--path` value, and two positional names) all returned exit code 2.
- No game or emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
