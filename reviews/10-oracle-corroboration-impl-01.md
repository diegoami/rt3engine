# Implementation review — oracle corroboration

- **Revision reviewed:** `d91f2fee10544d2206c2d27cda877d37c4d8edab`
- **Files checked:** `docs/evidence/gmp-terrain-v1.md` — obtained from
  `gh pr view 17 --json headRefOid,baseRefOid,files`; the PR reported this
  single modified file (48 additions, 0 deletions). Confirmed the local
  `HEAD` matched the reported head before reviewing the full diff.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode implementation review (`design: none`)

## Findings

None.

## Verification

- Inspected the complete changed document and diff. The six retained picker
  captures show the listed map names and exact sizes: British Isles 320×512,
  Chicago to New York 640×512, East Coast, USA 320×256, Argentina Opens Up
  640×384, Central Pacific 512×256, and Ireland 256×320. The Ireland picker
  still displays the default Argentina title, as documented; its loaded
  capture shows Ireland and Dublin.
- Parsed `docs/evidence/terrain-scan-v1.json`: 40 maps total (13 verified,
  26 pitch-anchored, one candidate). Of the six picker confirmations, five
  are pitch-anchored and East Coast is verified, leaving exactly 21
  pitch-anchored maps without picker confirmation.
- Read the offline comparison script and reran it for British Isles, Italy,
  and Ireland. It maps grid cells to minimap pixels by nearest-neighbor
  integer indexing, classifies grid water as zero or NaN and minimap water
  with `B > G > R`, then tests no flip, horizontal, vertical, and both-axis
  reflections. The reproduced vertical-flip match scores and next-best
  reflection scores round to 75.7% / 61.9%, 76.0% / 65.4%, and 77.2% /
  73.5%, respectively. These comparisons support the stated row-order
  corroboration for these three maps only; the document does not turn them
  into a general threshold or claim for untested maps.
- The retained substitution log records the same SHA-256,
  `b7aaed58f6e54b73f40f0e75d4ce2d866cbede252329c0623d7eb4f6f87d5214`,
  for Ireland and the substituted Argentina filename; it records game exit
  and restoration. The byte comparison of restored Argentina against its
  backup passed. No RT3 game process was running at review completion.
- The PR file list contains no screenshots or original game bytes; the only
  implementation change is the evidence document.
- Gates passed (with `RT3_HOME` set to the installation documented in
  `docs/evidence/README.md`):
  - `~/.cargo/bin/cargo fmt --all -- --check`
  - `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings`
  - `~/.cargo/bin/cargo test --workspace`
    (34 `rt3-kit` tests and 3 `rt3ctl` tests passed)
  - `CARGO=$HOME/.cargo/bin/cargo sh tools/evidence.sh check`

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
AGREE

## Completion

Landed with merge commit `97f5615` (delegated merge per decision 0002,
2026-11-23). Done-when items and evidence:

- Original-picker lattice checks are recorded for six maps: five
  pitch-anchored plus East Coast; the other 21 pitch-anchored maps remain
  explicitly open.
- Vertical-flip minimap comparisons establish south-to-north stored rows
  on British Isles, Italy and Ireland, with scores, method and limitations
  recorded; no unchecked-map generalization was made.
- Ireland loaded from unmodified bytes under the documented oracle-only
  filename substitution; Argentina was restored by an exit trap and
  verified byte-for-byte with `cmp`.
- Decoder and artifact remained unchanged. Formatting, clippy, 34
  `rt3-kit` tests, three `rt3ctl` tests and the evidence check passed;
  CI and GitGuardian succeeded on the reviewed head.
- Final AGREE covers implementation revision
  `d91f2fee10544d2206c2d27cda877d37c4d8edab` and is posted on PR #17.

— Implementer (zai-coding-plan/glm-5.3)
