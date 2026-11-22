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
//! | `save` | `Saved Games/*.sav` | milestone 4 |
//!
//! Every format claim in this crate must be backed by an artifact under
//! `docs/evidence/` that re-derives it from `RT3_HOME`. No original bytes are
//! ever committed; tests use fixtures or the installation via `RT3_HOME` and
//! self-skip when it is unset.
//!
//! Status: milestone 1 — `pk4` (PK4 asset archives); milestone 2 — `typed`
//! (typed game-data records and text/config). The remaining families land
//! with their milestones.

pub mod map;
pub mod pk4;
pub mod typed;
