//! Format decoders for original Railroad Tycoon 3 data.
//!
//! This crate owns read-side knowledge of the game's on-disk formats, as
//! measured against the owner's installation behind `RT3_HOME`
//! (see `docs/environment.md`). Planned coverage, one module per family:
//!
//! | Module (planned) | Format | Source |
//! | --- | --- | --- |
//! | `pk4` | `Data/{2D,3D,Sound}/*.PK4` packed asset archives | milestone 1 |
//! | `typed` | `CargoTypes/*.cty`, `BuildingTypes/*.bty`+`.bca`, `EngineTypes/*.car`+`.lco`, `TreeTypes.txt`, `AirplaneTypes.txt` | milestone 2 |//! | `config` | `Data/Configuration/*.cfg` | milestone 2 |
//! | `language` | `Data/Language/RT3.lng` | milestone 2 |
//! | `map` | `maps/*.gmp`, `Data/GrayscaleMaps/*.TGA` | milestone 3 |
//! | `cities` | city collections and raw labels in `maps/*.gmp` | milestone 3b |
//! | `buildings` | placed building/industry collections | milestone 3b |
//! | `companies` | company collections and raw names | milestone 4 |
//! | `players` | player collections and raw names | milestone 4 |
//! | `events` | scenario event records, names and text | milestone 3b |
//! | `terrain` | terrain heightfields inside `maps/*.gmp` | milestone 3b |
//! | `tiles` | per-cell arrays and per-tile terrain records | milestone 3b |
//! | `render` | north-up RGB PNG views of decoded heightfields | milestone 3b |
//! | `save` | `Saved Games/*.gmc` and `*.gms` | milestone 4 |
//!
//! Every format claim in this crate must be backed by an artifact under
//! `docs/evidence/` that re-derives it from `RT3_HOME`. No original bytes are
//! ever committed; tests use fixtures or the installation via `RT3_HOME` and
//! self-skip when it is unset.
//!
//! Status: milestone 1 — `pk4` (PK4 asset archives); milestone 2 — `typed`
//! (typed game-data records and text/config); milestone 3 — `map`
//! (`.gmp` prelude); milestone 3b — `terrain` (coarse heightfield),
//! `tiles` (per-tile records), `cities`, `buildings`, `events` and
//! `render` (north-up PNG); milestone 4 — `save` (original GMC/GMS structural
//! inventory), with `companies` and `players` fixed collections. The remaining
//! families land with their milestones.

pub mod buildings;
pub mod cities;
pub mod companies;
pub mod events;
pub mod fixed;
pub mod map;
pub mod pk4;
pub mod players;
pub mod rail;
pub mod render;
pub mod save;
pub mod sparse;
pub mod terrain;
pub mod tiles;
pub mod typed;
