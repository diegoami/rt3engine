# Implementation review — PR #53 (round 2)

- **Revision:** `00e5c4005593de80ba490c4cfcf8500b4a8d1e47`
- **Pull request:** [#53](https://github.com/diegoami/rt3engine/pull/53), base `main` at `91943582bcaedf6f252726121ee17e0c48cf3957`
- **Files checked (PR file list):**
  - `BACKLOG.md`
  - `docs/environment.md`
  - `docs/prior-art.md`
  - `reviews/40-prior-art-impl-01.md`
- **How obtained:** `gh pr view 53 --json headRefOid,baseRefOid,files` reported this head, base and four paths; local `HEAD` and `origin/docs/prior-art-survey` equalled the reported head. `git merge-base main 00e5c4005593de80ba490c4cfcf8500b4a8d1e47` equalled the reported base, and `git diff --name-only 91943582bcaedf6f252726121ee17e0c48cf3957..00e5c4005593de80ba490c4cfcf8500b4a8d1e47` matched the PR file list above. The three implementation paths are unchanged from round 1; the fourth path is the round-1 review record. Read the full merge-base diff and the round-2 diff.
- **Reviewer:** GPT-6 Luna (`opencode-go/gpt-6-luna#high`)
- **Mode:** OpenCode; project slot records `design: none`; merge delegated on AGREE.

## Round 1 follow-up

1. **Partially resolved — round-1 finding 1.** `docs/environment.md` now calls 1.05 an inference, and `docs/prior-art.md` preserves the question mark in the Sugus map-version observation. The categorical statement that a 1.06 installation is not the reference machine remains; see finding 1 below.
2. **Resolved — round-1 finding 2.** `docs/prior-art.md:46` now records the README's MIT statement separately from the absent `LICENSE` file and null GitHub license metadata, and labels the README as an upstream claim.
3. **Resolved — round-1 finding 3.** `docs/prior-art.md:45` now matches the current OpenRT3 README's target, function counts and differential-test count.

## Findings

1. **blocking — the page still rules out a 1.06 executable installation without evidence.** `docs/environment.md:35-37` says that 9901 originals are rejected by the parser and then states “a 1.06 installation is not the reference machine.” The revised paragraph correctly qualifies 1.05 as an inference and says no independent build identifier has been matched, but the categorical 1.06 exclusion does not follow: the parser accepts/rejects map-format versions, while the observed map headers identify maps, not the executable build. The matching hashes pin these files but are not compared to an independently identified build. Reword the final clause to limit it to the parser's map-version support, or qualify the 1.06 exclusion as unverified.

## Verification

- The RRT3SaveManager README's License section states MIT; GitHub `licenseInfo` is `null`, and the repository tree contains no `LICENSE` file. The revised row accurately distinguishes the upstream statement from verified metadata.
- The current OpenRT3 README identifies `RT3.exe` 1.06 (2004-09-08), 4,376 of 4,390 functions rewritten, and 829 differential tests. The revised row matches those scope facts. I read the README only, not source or decompilations; the clean-room restriction remains intact.
- The Sugus readme labels `E5 25` as “patch 1.05?” and `AD 26` as 1.06 for map-version bytes. `History.rtf` has headings 1.05 through 1.01, newest first. The recorded hashes and sizes match the installation; 40 current maps carry `e5 25` (9701), while the three legacy maps carry `89 25` twice and `8a 25` once. These support recording 1.05 as an inference, but do not independently identify the executable patch.
- `$HOME/tools/bin/make fmt` and `make clippy` passed. `make test` with the requested installation and fixtures under `review-tmp/saves/` completed successfully, including the install-backed evidence check. CI `check` and GitGuardian Security Checks passed for this revision. No emulator was booted.

— GPT-6 Luna (opencode-go/gpt-6-luna#high), reviewer
BLOCK
