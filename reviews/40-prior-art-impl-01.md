# Implementation review — PR #53 (round 1)

- **Revision:** `224d6eafcbdee2e4b2a7bfd74cd5cfea1d144863`
- **Pull request:** [#53](https://github.com/diegoami/rt3engine/pull/53), base `main` at `91943582bcaedf6f252726121ee17e0c48cf3957`
- **Files checked (PR file list):**
  - `BACKLOG.md`
  - `docs/environment.md`
  - `docs/prior-art.md`
- **How obtained:** `gh pr view 53 --json headRefOid,baseRefOid,files` reported this head, base and three paths; local `HEAD` and `origin/docs/prior-art-survey` equalled the reported head. `git merge-base main 224d6eafcbdee2e4b2a7bfd74cd5cfea1d144863` equalled the reported base, and `git diff --name-only 91943582bcaedf6f252726121ee17e0c48cf3957..224d6eafcbdee2e4b2a7bfd74cd5cfea1d144863` matched the PR file list above. Read the full merge-base diff.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`; merge delegated on AGREE.

## Findings

1. **blocking — the evidence does not verify the executable's patch level.** `docs/environment.md:29-35` calls the installation patch 1.05 based on `History.rtf` and map marks; `docs/prior-art.md:55` treats the 1.05 map mark as definitive. I reproduced the recorded hashes and sizes, found Version 1.05 as the newest `History.rtf` heading, and counted 40 `0x25E5` current maps plus three older maps. However, the linked Sugus readme labels `0xE5 0x25` as “patch 1.05?” and describes map-version bytes, not the executable build. The PE version resource is acknowledged as non-diagnostic, and the hashes are not matched to an independently identified build. These facts support the history and map format, not a verified RT3.exe patch level. Qualify the patch claim as an inference or supply independent executable-build evidence.

2. **blocking — the RRT3SaveManager license row omits an upstream license statement.** `docs/prior-art.md:46` says `None`. `gh repo view atshaw1994/RRT3SaveManager --json licenseInfo` returns `null` and the repository has no `LICENSE` file, but its `readme.md` has a `License` section stating `MIT`. The row should preserve both facts (and the absence of a license file) rather than report simply `None`; do not imply the GitHub metadata verifies the README's claim.

3. **non-blocking — the OpenRT3 scope summary is stale.** `docs/prior-art.md:45` describes `RT3_mp.exe` and estimates ~5,100 game functions. The repository's current README describes an `RT3.exe` 1.06 target and reports 4,376 of 4,390 game functions rewritten. Update the summary to match the current published description; this does not change the clean-room restriction or adoption verdict.

## Verification

- The seven GitHub `licenseInfo` results matched the recorded metadata except for the README statement noted above: `rt3conf` MIT, OpenRT3 / RT3Multiplayer / RRT3SaveManager `null`, `rt3-modern-1440p` Other, `rrt2-decomp` GPL-3.0, and `bluddy/rails` MIT. The other repository descriptions/READMEs supported their summarized scopes.
- The linked forum pages and Trainmaster readme support the listed extraction, packing, skinning, converter and map-tool scopes. The inspected pages/readme state no license for the Trainmaster kit or Sugus utility; the Sugus readme explicitly marks 1.05 with a question mark. The installed `UserSkinningTools` contains the listed packer, bin-list, UnitC, TrainSkin, logo and portrait tools; `_Extras/MapBuilder` contains `MapBuilder.exe`.
- `RT3.exe` and `d8tx.dll` hashes/sizes match `docs/environment.md`. `History.rtf` contains version headings 1.05 through 1.01, newest first. A recursive map count found 40 `e5 25` current maps, two `89 25` tutorial maps and one `8a 25` legacy map. The version markers match the recorded byte values.
- The clean-room policy is unchanged and no candidate is recorded as adopted. All reviewed links resolved; ModDB rejected a HEAD request but its page was successfully fetched with GET. Issue #3 is referenced as closing in the PR body.
- `$HOME/tools/bin/make fmt`, `make clippy`, and `make test` passed with `RT3_HOME` and the three fixtures under `review-tmp/saves/`; the test run included the install-backed evidence check. CI checks `check` and GitGuardian Security Checks passed for this revision. No emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
