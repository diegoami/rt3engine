# Implementation review: map-record height anchor (round 2)

- **Revision covered:** `8ac611abdd3073f8bfb9c11c18cbb2c1ad55b79a`
- **Files checked (full PR #13 local diff):** `crates/rt3-kit/src/terrain.rs`, `crates/rt3ctl/src/map_cmd.rs`, `docs/evidence/gmp-terrain-v1.md`, `docs/evidence/terrain-scan-v1.json`, `reviews/05-dims-terrain-impl-03.md`, `reviews/06-record-anchor-impl-01.md`.
- **How obtained:** `git merge-base 6b90123 8ac611a` returned `6b9012369090e0008d2c92e40d8c18ff088cade2`; `git diff --name-only <merge-base>..8ac611abdd3073f8bfb9c11c18cbb2c1ad55b79a` returned the six files above. Checked-out `HEAD` matched the covered revision. Inspected `git diff <merge-base>..8ac611abdd3073f8bfb9c11c18cbb2c1ad55b79a -- crates docs` and read the touched files at `HEAD`.
- **Reviewer:** GPT-6 Luna, `opencode-go/gpt-6-luna#high` (effort high)
- **Mode:** OpenCode

## Round-1 findings rechecked

1. **Resolved.** The evidence document now says the record anchor resolves non-square orientation, lists all eight verified maps, and identifies Central Pacific as the sole candidate with the three withdrawn windows noted. The artifact has 8 verified, 1 candidate, 31 no-grid, and 13 dims headers; its verified list includes Alternate USA, Orient Express, and Southern Pacific.
2. **Resolved.** The counts distinguish 32 of 40 without a verified decode from 31 with no grid. The installed 27 headerless current maps have record heights from 256 through 768; the 13 header-bearing maps' record heights match `dims.values[5]`.
3. **Resolved.** The install-backed `scans_reference_maps` test now includes Alternate USA (`1024×640`).

## Findings

1. **non-blocking** — `crates/rt3-kit/src/terrain.rs:17-20`. The module overview says headerless maps are scanned with fixed 256-lattice geometry and their hits become candidates. The CLI now attempts that scan only when the record anchor is absent or gives height 256 (`crates/rt3ctl/src/map_cmd.rs:214-217`). Clarify this gating in the overview.

## Verification

- `~/.cargo/bin/cargo fmt --all -- --check` — passed.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — passed.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' ~/.cargo/bin/cargo test -p rt3-kit terrain -- --nocapture` — 13 passed, 0 failed; includes the Alternate USA reference.
- `RT3_HOME='/mnt/c/Program Files/GOG Galaxy/Games/Railroad Tycoon 3' CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — passed with no drift.
- Independently checked installed current maps: 40 total, 13 with dims headers, and 27 headerless with record heights in 256…768; all 13 header-bearing record heights match their dims height. Parsed the evidence artifact and reconciled its counts and map classifications.
- No game or emulator was booted; no capture was needed.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE
