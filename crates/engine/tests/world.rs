//! Install-backed check for claim C1: a map-grounded world loads the
//! structural records and the cargo catalogue. Terrain and cargo figures are
//! read from the committed evidence artifacts; city and building counts are
//! compared with `rt3-kit`'s own collections on the same bytes.

use std::fs;
use std::path::PathBuf;

use engine::world;

struct Case {
    dir: &'static str,
    file: &'static str,
}

const CASES: [Case; 3] = [
    Case {
        dir: "maps",
        file: "British Isles.gmp",
    },
    Case {
        dir: "maps",
        file: "California Island.gmp",
    },
    // A current map with placed building records, so the building path is
    // exercised with a non-zero collection too.
    Case {
        dir: "maps",
        file: "Greenland Growing.gmp",
    },
];

fn home() -> Option<PathBuf> {
    std::env::var_os("RT3_HOME").map(PathBuf::from)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn number_after(text: &str, key: &str) -> f64 {
    let start = text
        .find(key)
        .unwrap_or_else(|| panic!("missing {key} in evidence"))
        + key.len();
    let rest = text[start..].trim_start();
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '-' || c == '.'))
        .unwrap_or(rest.len());
    rest[..end]
        .parse()
        .unwrap_or_else(|_| panic!("bad number after {key}"))
}

/// The `terrain-scan-v1.json` row for one map, as
/// (rows, cols, min height, max height).
fn terrain_artifact(name: &str) -> (usize, usize, f64, f64) {
    let text = fs::read_to_string(repo_root().join("docs/evidence/terrain-scan-v1.json"))
        .expect("terrain artifact");
    let marker = format!("\"map\": \"{name}\"");
    let start = text
        .find(&marker)
        .unwrap_or_else(|| panic!("no {name} row"));
    let rest = &text[start..];
    let end = rest[1..]
        .find("\"map\": ")
        .map(|offset| offset + 1)
        .unwrap_or(rest.len());
    let row = &rest[..end];
    (
        number_after(row, "\"rows\":") as usize,
        number_after(row, "\"cols\":") as usize,
        number_after(row, "\"min_height_m\":"),
        number_after(row, "\"max_height_m\":"),
    )
}

fn cargo_artifact_count() -> usize {
    let text = fs::read_to_string(repo_root().join("docs/evidence/typed-tables-v1.json"))
        .expect("typed artifact");
    let start = text.find("\"cargo_types\"").expect("cargo_types");
    number_after(&text[start..], "\"files\":") as usize
}

#[test]
fn loads_map_grounded_world() {
    let Some(home) = home() else {
        eprintln!("RT3_HOME unset; skipping install-backed world check");
        return;
    };
    let cargo_types = cargo_artifact_count();
    for case in CASES {
        let path = home.join(case.dir).join(case.file);
        let bytes = fs::read(&path).unwrap();
        let name = case.file.trim_end_matches(".gmp");
        let (rows, cols, min_expected, max_expected) = terrain_artifact(name);
        let cities = rt3_kit::cities::read(&bytes).expect("city collection");
        let buildings = rt3_kit::buildings::read(&bytes).expect("building collection");

        let world = world::load(&path, &home).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(world.map_name, name, "map name for {}", case.file);
        assert_eq!(
            (world.terrain.rows, world.terrain.cols),
            (rows, cols),
            "terrain dimensions for {} against terrain-scan-v1.json",
            case.file
        );
        let (min, max) = world.terrain.height_range().expect("height range");
        assert!(
            ((min as f64) - min_expected).abs() < 0.06,
            "{} min height {min} != {min_expected}",
            case.file
        );
        assert!(
            ((max as f64) - max_expected).abs() < 0.06,
            "{} max height {max} != {max_expected}",
            case.file
        );
        assert_eq!(
            world.city_records,
            cities.records.len(),
            "cities for {} against rt3-kit",
            case.file
        );
        assert_eq!(
            world.building_records,
            buildings.records.len(),
            "buildings for {} against rt3-kit",
            case.file
        );
        assert_eq!(
            world.cargo.len(),
            cargo_types,
            "cargo types for {} against typed-tables-v1.json",
            case.file
        );
        assert!(
            world.cargo.iter().any(|t| t.name == "Alcohol"),
            "named cargo type for {}",
            case.file
        );
        assert!(
            world
                .cargo
                .windows(2)
                .all(|pair| (pair[0].index, &pair[0].name) <= (pair[1].index, &pair[1].name)),
            "cargo catalogue order for {}",
            case.file
        );
    }
}

#[test]
fn rejects_legacy_map_versions() {
    let Some(home) = home() else {
        eprintln!("RT3_HOME unset; skipping install-backed version check");
        return;
    };
    let path = home.join("Data").join("Tutorial").join("Tutorial_1.gmp");
    let error = world::load_map(&path).expect_err("legacy map must be rejected");
    assert!(
        error.to_string().contains("version 9609"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_malformed_prelude_despite_intact_sections() {
    let Some(home) = home() else {
        eprintln!("RT3_HOME unset; skipping malformed-prelude check");
        return;
    };
    let mut bytes = fs::read(home.join("maps").join("British Isles.gmp")).unwrap();
    // Keep version 9701 and every later section, but remove the prelude's NUL
    // terminator: the map parser must reject this before a World is built.
    for byte in &mut bytes[4..4 + 30_000] {
        *byte = 0x41;
    }
    let path = std::env::temp_dir().join(format!(
        "rt3engine-malformed-prelude-{}.gmp",
        std::process::id()
    ));
    fs::write(&path, &bytes).unwrap();
    let error = world::load_map(&path).expect_err("malformed prelude must be rejected");
    fs::remove_file(&path).ok();
    assert!(
        error
            .to_string()
            .contains("not a supported version-9701 map"),
        "unexpected error: {error}"
    );
}

#[test]
fn rejects_non_map_files_with_a_named_error() {
    let path = std::env::temp_dir().join(format!("rt3engine-not-a-map-{}.bin", std::process::id()));
    fs::write(&path, b"not a map at all").unwrap();
    let error = world::load_map(&path).expect_err("non-map file must be rejected");
    fs::remove_file(&path).ok();
    let message = error.to_string();
    assert!(
        message.contains("not a supported version-9701 map"),
        "unexpected error: {message}"
    );
}
