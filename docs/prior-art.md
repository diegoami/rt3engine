# Prior art and reuse policy

Survey updated 2026-11-25 (issue
[#3](https://github.com/diegoami/rt3engine/issues/3); GitHub repository search
plus a broader web and forum survey, see *Method*). The 2026-11-21
GitHub-only table is superseded by this one.

## Policy

- This project's code is MIT, written here. **No code is copied from GPL or
  proprietary sources**; such projects are reference-only at most.
- Reverse-engineering findings (file formats, constants, algorithms) are
  facts and are documented freely with their method (Ghidra project, oracle
  session). Third-party code remains off-limits regardless.
- **Clean-room discipline for unlicensed reimplementations:** where a prior
  project reverse-engineers the same game (e.g. OpenRT3), implementers do
  not read its source or generated decompilations. Our findings are derived
  independently with our own Ghidra project on the GOG `RT3.exe`; where
  topics overlap, our evidence records the independent provenance (the PK4
  format record predates any contact with that project's material).
- Before adopting any third-party tool, library or dataset: license audit,
  provenance record, and an explicit decision under `docs/decisions/`.
- The original `RT3.exe` and `d8tx.dll` are analyzed, never executed at
  runtime by our product and never distributed.

## Method

- GitHub repository search for RT3/RRT3 candidates, then license metadata
  from each repository; licenses below were re-read on 2026-11-25.
- Broader web and forum survey on 2026-11-25: DuckDuckGo HTML results and the
  Hawk & Badger Railroad archive and forum pages (links in the table). The
  `websearch` integration was unavailable; the searches were run by fetching
  the search page and the linked pages directly.
- Local inspection of the tools shipped with the GOG installation
  (`UserSkinningTools`, `_Extras/MapBuilder`) and of the executable identity
  recorded in [environment.md](environment.md).
- A forum-hosted binary with no stated license is recorded as "none stated"
  and is reference-only: no binary, asset or code from it is used.

## Survey

| Candidate | Scope | License | Verdict |
| --- | --- | --- | --- |
| [MichaelMcDonnell/rt3conf](https://github.com/MichaelMcDonnell/rt3conf) | Rust writer for `Data/Configuration/engine.cfg` with RE'd field offsets (modern resolutions); documents a diff-based field method | MIT | Reference. Format facts usable; milestone 2 input. Code reusable with attribution if ever adopted. |
| [FlorianFFF/OpenRT3](https://github.com/FlorianFFF/OpenRT3) | Function-at-a-time reimplementation of `RT3_mp.exe` (2004-09-08 build) in C++ (re3/OpenRCT2 method: DLL hooks, differential tests); includes PK4 readers and format docs; ~5,100 game functions estimated | **None** | Reference-only. Existence and method noted; **no source reading by implementers** (clean-room, see policy). |
| [atshaw1994/RRT3SaveManager](https://github.com/atshaw1994/RRT3SaveManager) | Cross-platform save-game manager (C#) | None | Reference-only; potentially relevant to milestone 4 (save format). No source reading beyond scope facts. |
| [FlorianFFF/RT3Multiplayer](https://github.com/FlorianFFF/RT3Multiplayer) | Community multiplayer lobby for RT3 | None | Behavior notes only. |
| [PariPariKai/rt3-modern-1440p](https://github.com/PariPariKai/rt3-modern-1440p) | Unofficial Steam 1.05 compatibility package: DXVK, 2560×1440 menu/scene fixes, reversible installer | Other (NOASSERTION) | Behavior notes only. |
| [pedryvo/rrt2-decomp](https://github.com/pedryvo/rrt2-decomp) | Railroad Tycoon **II** reverse engineering and Rust reimplementation | GPL-3.0 | Different game; context only, no source reading. |
| [bluddy/rails](https://github.com/bluddy/rails) | Original Railroad Tycoon (1990) remake in OCaml | MIT | Different game; context only. |
| `UserSkinningTools` (shipped in the installation) | `pack4.exe`, `binlist2.exe`, `UnitC.exe`, `TrainSkin`/logo/portrait batch tools: PK4 packing, DDS and logo/portrait packaging | Proprietary (part of the GOG installation) | Behavior notes only; never copied, distributed or invoked from our product. Our PK4 format facts remain independently derived. |
| `_Extras/MapBuilder/MapBuilder.exe` (shipped in the installation) | Official scenario-map editor | Proprietary | Behavior notes only. |
| Watto Studios Game Extractor (named in the [Hawk & Badger mod-tools thread](https://hawkdawg.com/forums/viewtopic.php?t=1107)) | PK4 extraction; closed source; community notes say it cannot create usable PK4 | Freeware; no source or license stated | Reference-only; extraction behavior only. |
| [Trainmaster Rolling Stock Developer Kit](https://hawkdawg.com/rrt/rrt3_extras/rrt3_utilities.php) (nedfumpkin) | `pack4` batch packing, Java `PK4Unpack`, DDS and hex utilities, plus the game's own user-skinning tools | None stated | Reference-only; behavior notes only. |
| [RT3 Map Overview & 1.06 Downgrade Utility](https://hawkdawg.com/txt/rt3/xtras/sugus/RT3p6dgUtility.txt) (Sugus) | Map listing and map-version relabel utility | None stated | Reference-only. Its readme documents the map version marks `0x25E5` (1.05) and `0x26AD` (1.06); recorded as a format fact with that method. |
| [PJay BMP2GMP & GMP2BMP](https://hawkdawg.com/rrt/rrt3_extras/rrt3_utilities.php) | Map bitmap converters | None stated | Reference-only. |
| [PK4Scape](https://www.moddb.com/downloads/pk4scape), [id-pk4-extractor](https://github.com/klaussilveira/id-pk4-extractor) | id Tech 4 PK4 tools (Doom 3, Quake 4) | n/a (separate projects) | Not applicable: RT3's PK4 is not the id Tech 4 container; the shared extension is a false friend. |
| Generic editors named by the community (XVI32, Hexplorer, DDSview, DDS Converter 2, NVidia DDS utilities, GIMP, MicroDEM) | General hex, image or GIS tools | Various | No RT3-specific format knowledge; no adoption and no impact. |

## Open items

- A deeper archive sweep (map sites, Trainmaster sub-sites, legacy tools) may
  add candidates; add rows with the same method and license check.
- **No third-party code, binary or asset is adopted.** Any future adoption
  needs a license audit and an owner decision under `docs/decisions/`.
- Version identity is settled for this installation: 1.05, with the executable
  and DLL hashes recorded in [environment.md](environment.md).
