//! Map-grounded world for the headless kernel (milestone 5, claim C1).
//!
//! The world holds only parser-proven structural data: the coarse terrain
//! heightfield, city and placed-building record counts, and the typed cargo
//! catalogue. Building and city *semantics* (industry types, production,
//! demand links) are not established by the format evidence, so they are not
//! represented here; the declared economy model lands with claim C2.
//!
//! Loading is read-only. The original installation is never written, and the
//! caller supplies its root (`RT3_HOME` or `--path`); no path is baked in.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use rt3_kit::buildings;
use rt3_kit::cities;
use rt3_kit::map::GmpMap;
use rt3_kit::terrain::{self, HeightGrid};
use rt3_kit::typed::CargoType;

/// Error raised while loading a map or the cargo catalogue.
#[derive(Debug)]
pub struct WorldError {
    path: PathBuf,
    detail: String,
}

impl WorldError {
    fn new(path: &Path, detail: impl fmt::Display) -> WorldError {
        WorldError {
            path: path.to_path_buf(),
            detail: detail.to_string(),
        }
    }
}

impl fmt::Display for WorldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.detail)
    }
}

impl std::error::Error for WorldError {}

/// A loaded map plus the cargo catalogue the economy runs on.
#[derive(Debug, Clone)]
pub struct World {
    /// Map file stem, e.g. `British Isles`.
    pub map_name: String,
    /// Coarse terrain heightfield from the map's explicit frame.
    pub terrain: HeightGrid,
    /// City records in the map's city collection.
    pub city_records: usize,
    /// Placed building records in the map's building collection.
    pub building_records: usize,
    /// Typed cargo catalogue, ordered by cargo index then name.
    pub cargo: Vec<CargoType>,
}

/// Loads one map's structural world. The map prelude is validated by the
/// map parser before any section is read, so a malformed header cannot be
/// reported as a world; only version-9701 maps are accepted.
pub fn load_map(map: &Path) -> Result<World, WorldError> {
    let bytes = fs::read(map).map_err(|e| WorldError::new(map, e))?;
    GmpMap::from_bytes(map, &bytes)
        .map_err(|e| WorldError::new(map, format!("not a supported version-9701 map: {e}")))?;
    let terrain = terrain::read_framed(&bytes)
        .ok_or_else(|| WorldError::new(map, "no framed terrain grid"))?;
    let city = cities::read(&bytes).ok_or_else(|| WorldError::new(map, "no city collection"))?;
    let building =
        buildings::read(&bytes).ok_or_else(|| WorldError::new(map, "no building collection"))?;
    let map_name = map
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    Ok(World {
        map_name,
        terrain,
        city_records: city.records.len(),
        building_records: building.records.len(),
        cargo: Vec::new(),
    })
}

/// Loads the typed cargo catalogue from `Data/CargoTypes/*.cty`.
pub fn load_cargo(home: &Path) -> Result<Vec<CargoType>, WorldError> {
    let dir = home.join("Data").join("CargoTypes");
    let entries = fs::read_dir(&dir).map_err(|e| WorldError::new(&dir, e))?;
    let mut cargo = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| WorldError::new(&dir, e))?;
        let path = entry.path();
        let is_cargo = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("cty"))
            .unwrap_or(false);
        if !is_cargo {
            continue;
        }
        let bytes = fs::read(&path).map_err(|e| WorldError::new(&path, e))?;
        cargo.push(CargoType::parse(&path, &bytes).map_err(|e| WorldError::new(&path, e))?);
    }
    if cargo.is_empty() {
        return Err(WorldError::new(&dir, "no cargo types found"));
    }
    cargo.sort_by(|a, b| a.index.cmp(&b.index).then_with(|| a.name.cmp(&b.name)));
    Ok(cargo)
}

/// Loads a map and the installation's cargo catalogue into one world.
pub fn load(map: &Path, home: &Path) -> Result<World, WorldError> {
    let mut world = load_map(map)?;
    world.cargo = load_cargo(home)?;
    Ok(world)
}
