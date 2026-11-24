Revision reviewed: `4a8e0947c3e8209e02fcef3d810783062208a36f`

PR #24 head and base were obtained with
`gh pr view 24 --json headRefOid,baseRefOid,files`; the head matches this
revision, the base is
`ef0837fa00d0149d74703f23a6246aad847cd7f1`, and the file list is exactly:

- `crates/rt3-kit/src/terrain.rs`
- `docs/evidence/gmp-terrain-v1.md`

Reviewer: GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
Mode: OpenCode; `design: none`; implementation review.

## Findings

None, blocking or non-blocking.

## Verification

- Independently inspected the retained reader, writer and caller
  disassemblies. `0x0044cfb0` calls `0x00553240` at `0x0044cffb`.
  `0x00553240` seeks `0x2af9`, reads the `0xbba` constant and dimensions,
  seeks `0x2afa`, and reads the layout tag. Its `0x2afb` branch multiplies
  unincremented dimensions; for the observed `0x2afc` layout the other
  branch increments both dimensions before multiplying and passes
  product × 4 to the `0x00531150` payload read. `0x00553760` writes the
  same tags and dimensions, increments both before multiplying, and writes
  product × 4. The caller and these reader paths support the documented
  `0x00531360` tag-seek and `0x00531150` payload-read interpretations.
  The `0x2afb` layout is correctly distinguished from the observed
  `0x2afc` layout; it is not claimed observed in shipped maps. The document
  also leaves the remaining section handlers open rather than claiming a
  complete handler map.
- Parser dimensions are capped at 8,193 before incrementing or multiplying
  (`terrain.rs:414`, `420–422`); the full extent check precedes validation
  and range reads (`422–428`).
- Independently enumerated all 43 installation maps: every complete stored
  cell is NaN or within the accepted height range, every final stored row
  is exactly zero, and the total final-row extent is 21,398 cells across
  19,755 corner-grid rows. Returned offsets, dimensions and corner-grid
  statistics match `terrain-scan-v1.json`.
- Synthetic framed tests cover a valid complete frame, both `0x2ee2` and
  `0x2ee1`, truncation by one final stored cell, an out-of-range final
  stored cell, and a bad frame signature (`terrain.rs:945–982`). The
  truncation and out-of-range cases would be invisible to the former
  corner-grid-only extent and range checks. The install-backed
  trailing/final-row regression checks
  the extra row from column 0 through `cols` and asserts all 21,398 values
  are zero (`terrain.rs:1179–1220`).
- `~/.cargo/bin/cargo fmt --all -- --check` — pass.
- `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` — pass.
- With `RT3_HOME` set to the installation specified in the review prompt,
  `~/.cargo/bin/cargo test --workspace` — pass: 42 `rt3-kit`, 5 `rt3ctl`;
  no failures.
- With the same `RT3_HOME`,
  `CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check` — pass; no drift.
- No emulator boot.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

Landed with merge commit `e8a1dc4` (delegated merge per decisions 0002
and 0003, 2026-11-24). Done-when items and evidence:

- Static RE identifies `0x00553240` as the coarse-grid reader called by
  `0x0044cfb0`, with `0x00553760` as writer. Both establish the observed
  `0x2afc` payload as `(cols+1)*(rows+1)` f32 cells.
- `read_framed` now requires and range-checks the complete stored array
  while returning only the corner grid. All 43 original maps pass with
  no artifact drift.
- Every additional final-row cell is zero: 21,398 cells across all 43
  maps, checked by the extended install-backed regression.
- Synthetic truncation and invalid-final-row assertions failed against
  old/bypassed behavior and pass with the fix. Tags/layout and remaining
  open semantics are recorded without broadening the unobserved branch.
- Formatting, clippy, 42 `rt3-kit` tests, five `rt3ctl` tests and the
  evidence check passed; CI and GitGuardian succeeded on the reviewed
  head. Final AGREE covers implementation revision
  `4a8e0947c3e8209e02fcef3d810783062208a36f` and is posted on PR #24.

— Implementer (zai-coding-plan/glm-5.3)
