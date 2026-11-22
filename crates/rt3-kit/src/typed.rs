//! Readers for Railroad Tycoon 3's typed game-data records.
//!
//! Formats and provenance (full detail: `docs/evidence/typed-formats-v1.md`;
//! claim level **parse**, milestone 2):
//!
//! | Family | Files | Layout |
//! | --- | --- | --- |
//! | `CargoTypes/*.cty` | 41 | 62-byte records: version 1002, name[20], price f32, factor f32, category u8, icon u32, car-type name[21], index i32 |
//! | `BuildingTypes/*.bty` | 190 | version 1004: 6 × name[30], dims u8×3, u32×4, u16×2, `w·h·t·t/2` blob, version-gated extras |
//! | `BuildingTypes/*.bca` | 67 | version 1010: name[30], fields, count u32 @0x33, `count × 188`-byte subrecords (cargo rules with `{...}` scripts), 169-byte tail |
//! | `EngineTypes/*.lco` | 66 | version 2005: name[30], 4 × f32 (speed/power/price/reliability), u8, u32×7, u8×4, 2 × clamped i32 |
//! | `EngineTypes/*.car` | 145 | version 1002: flag u32, id u32, name[60], lco-name[60], name[30]×2, name[60], 20-byte tail |
//! | `TreeTypes.txt`, `AirplaneTypes.txt` | 2 | plain text tables (`;` comments, whitespace columns) |
//! | `Language/RT3.lng` | 1 | `;`-commented `id text` lines; `\n` escapes a newline |
//! | `Configuration/engine.cfg` | 1 | header 0x41B, width, height, then the field map cross-checked against MIT rt3conf |
//! | `Configuration/game.cfg` | 1 | first u32 is the payload length, followed by the payload |
//!
//! Loaders were located in `RT3.exe` via Ghidra (see the evidence doc for
//! addresses). Fields whose semantics are not yet established are parsed
//! and validated structurally but carried as raw numbers; unknown bytes are
//! tolerated (the original authoring tools left 0xCC/0xCD debug fill).

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// Error raised while parsing a typed record.
#[derive(Debug)]
pub struct TypedError {
    path: PathBuf,
    detail: String,
}

impl TypedError {
    fn new(path: &Path, detail: impl fmt::Display) -> TypedError {
        TypedError {
            path: path.to_path_buf(),
            detail: detail.to_string(),
        }
    }
}

impl fmt::Display for TypedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.detail)
    }
}

impl std::error::Error for TypedError {}

/// Reads a NUL-padded fixed-width name field. Spaces are allowed inside
/// names; other control or non-ASCII bytes before the first NUL are not
/// (0xCC/0xCD debug fill may follow the terminator).
fn fixed_name(bytes: &[u8]) -> Result<String, String> {
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    let name = &bytes[..end];
    if !name.iter().all(|b| *b == b' ' || b.is_ascii_graphic()) {
        return Err(format!("name field has non-graphical bytes: {name:?}"));
    }
    Ok(String::from_utf8_lossy(name).into_owned())
}

fn u32_at(d: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(d[off..off + 4].try_into().unwrap())
}

fn i32_at(d: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(d[off..off + 4].try_into().unwrap())
}

fn f32_at(d: &[u8], off: usize) -> f32 {
    f32::from_le_bytes(d[off..off + 4].try_into().unwrap())
}

fn u16_at(d: &[u8], off: usize) -> u16 {
    u16::from_le_bytes(d[off..off + 2].try_into().unwrap())
}

/// A cargo type record (`CargoTypes/*.cty`, version 1002, 62 bytes).
#[derive(Debug, Clone, PartialEq)]
pub struct CargoType {
    pub name: String,
    /// Base price (per-unit revenue scale); observed 1e4..2.35e5.
    pub price: f32,
    /// Price/supply factor; observed 0.1..0.3.
    pub factor: f32,
    /// Category byte; observed 0..11 (semantics open).
    pub category: u8,
    /// Icon-related id; observed 0x708..0x776 (semantics open).
    pub icon: u32,
    /// Required car type name, e.g. `Reefer` (uppercased by the loader).
    pub car_type: String,
    /// Cargo index assigned by the game.
    pub index: i32,
}

impl CargoType {
    pub fn parse(path: &Path, d: &[u8]) -> Result<CargoType, TypedError> {
        if d.len() != 62 {
            return Err(TypedError::new(
                path,
                format!("expected 62 bytes, found {}", d.len()),
            ));
        }
        let version = i32_at(d, 0);
        if version != 1002 {
            return Err(TypedError::new(
                path,
                format!("unsupported .cty version {version}"),
            ));
        }
        Ok(CargoType {
            name: fixed_name(&d[4..24]).map_err(|e| TypedError::new(path, e))?,
            price: f32_at(d, 24),
            factor: f32_at(d, 28),
            category: d[32],
            icon: u32_at(d, 33),
            car_type: fixed_name(&d[37..58])
                .map_err(|e| TypedError::new(path, e))?
                .to_uppercase(),
            index: i32_at(d, 58),
        })
    }
}

/// A building type record (`BuildingTypes/*.bty`, version 1004).
#[derive(Debug, Clone, PartialEq)]
pub struct BuildingType {
    pub names: [String; 6],
    /// Grid dimensions; observed (6, 6, 48).
    pub dims: (u8, u8, u8),
    pub u32s: [u32; 4],
    pub u16s: [u16; 2],
    /// Length of the cell blob that follows the fixed header.
    pub blob_len: usize,
    /// Bytes consumed by the whole record (equals the file size).
    pub record_len: usize,
}

impl BuildingType {
    const HEADER: usize = 0xCF;

    pub fn parse(path: &Path, d: &[u8]) -> Result<BuildingType, TypedError> {
        if d.len() < 4 {
            return Err(TypedError::new(
                path,
                format!("truncated: {} bytes", d.len()),
            ));
        }
        let version = i32_at(d, 0);
        if version != 1004 {
            return Err(TypedError::new(
                path,
                format!("unsupported .bty version {version}"),
            ));
        }
        if d.len() < Self::HEADER {
            return Err(TypedError::new(
                path,
                format!("truncated: {} bytes", d.len()),
            ));
        }
        let mut names = Vec::with_capacity(6);
        for i in 0..6 {
            names.push(
                fixed_name(&d[4 + 30 * i..4 + 30 * (i + 1)])
                    .map_err(|e| TypedError::new(path, e))?,
            );
        }
        let dims = (d[0xB8], d[0xB9], d[0xBA]);
        let blob_len = dims.0 as usize * dims.1 as usize * dims.2 as usize * dims.2 as usize / 2;
        let mut record_len = Self::HEADER + blob_len;
        if version > 1000 {
            record_len += 8;
        }
        if version > 1002 {
            record_len += 12 * 5;
        }
        if version > 1003 {
            record_len += 30;
        }
        record_len += 4 + 46;
        if record_len != d.len() {
            return Err(TypedError::new(
                path,
                format!("record length {record_len} != file size {}", d.len()),
            ));
        }
        let names: [String; 6] = names.try_into().unwrap();
        Ok(BuildingType {
            names,
            dims,
            u32s: [
                u32_at(d, 0xBB),
                u32_at(d, 0xBF),
                u32_at(d, 0xC3),
                u32_at(d, 0xC7),
            ],
            u16s: [u16_at(d, 0xCB), u16_at(d, 0xCD)],
            blob_len,
            record_len,
        })
    }
}

/// A building behavior record (`BuildingTypes/*.bca`, version 1010).
#[derive(Debug, Clone, PartialEq)]
pub struct BuildingBehavior {
    pub name: String,
    /// Build cost (observed 100_000..200_000 for industries).
    pub cost: u32,
    /// Number of 188-byte behavior subrecords.
    pub count: u32,
    /// `{...}` script strings found inside the subrecords.
    pub scripts: Vec<String>,
}

impl BuildingBehavior {
    const SUBRECORD: usize = 188;
    const HEADER: usize = 55;
    const TAIL: usize = 169;

    pub fn parse(path: &Path, d: &[u8]) -> Result<BuildingBehavior, TypedError> {
        if d.len() < 4 {
            return Err(TypedError::new(
                path,
                format!("truncated: {} bytes", d.len()),
            ));
        }
        let version = i32_at(d, 0);
        if version != 1010 {
            return Err(TypedError::new(
                path,
                format!("unsupported .bca version {version}"),
            ));
        }
        if d.len() < Self::HEADER + Self::TAIL {
            return Err(TypedError::new(
                path,
                format!("truncated: {} bytes", d.len()),
            ));
        }
        let name = fixed_name(&d[4..34]).map_err(|e| TypedError::new(path, e))?;
        let count = u32_at(d, 0x33);
        let expected = Self::HEADER + count as usize * Self::SUBRECORD + Self::TAIL;
        if expected != d.len() {
            return Err(TypedError::new(
                path,
                format!(
                    "count {count} implies {expected} bytes, file has {}",
                    d.len()
                ),
            ));
        }
        let mut scripts = Vec::new();
        for i in 0..count as usize {
            let sub =
                &d[Self::HEADER + i * Self::SUBRECORD..Self::HEADER + (i + 1) * Self::SUBRECORD];
            if let Some(start) = sub.iter().position(|b| *b == b'{') {
                if let Some(len) = sub[start..].iter().position(|b| *b == b'}') {
                    let script = &sub[start..=start + len];
                    if script.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
                        scripts.push(String::from_utf8_lossy(script).into_owned());
                    }
                }
            }
        }
        Ok(BuildingBehavior {
            name,
            cost: u32_at(d, 0x22),
            count,
            scripts,
        })
    }
}

/// A locomotive descriptor (`EngineTypes/*.lco`, version 2005, 91 bytes).
#[derive(Debug, Clone, PartialEq)]
pub struct Locomotive {
    pub name: String,
    /// Speed, power, price and reliability factors (era-ordered values).
    pub floats: [f32; 4],
    pub u8_field: u8,
    pub u32s: [u32; 7],
    pub u8s: [u8; 4],
    /// Two loader-clamped ints (0..8, 0..9); semantics open.
    pub clamped: [i32; 2],
}

impl Locomotive {
    pub fn parse(path: &Path, d: &[u8]) -> Result<Locomotive, TypedError> {
        if d.len() != 91 {
            return Err(TypedError::new(
                path,
                format!("expected 91 bytes, found {}", d.len()),
            ));
        }
        let version = i32_at(d, 0);
        if version != 2005 {
            return Err(TypedError::new(
                path,
                format!("unsupported .lco version {version}"),
            ));
        }
        Ok(Locomotive {
            name: fixed_name(&d[4..34]).map_err(|e| TypedError::new(path, e))?,
            floats: [f32_at(d, 34), f32_at(d, 38), f32_at(d, 42), f32_at(d, 46)],
            u8_field: d[50],
            u32s: [
                u32_at(d, 51),
                u32_at(d, 55),
                u32_at(d, 59),
                u32_at(d, 63),
                u32_at(d, 67),
                u32_at(d, 71),
                u32_at(d, 75),
            ],
            u8s: [d[79], d[80], d[81], d[82]],
            clamped: [i32_at(d, 83).clamp(0, 8), i32_at(d, 87).clamp(0, 9)],
        })
    }
}

/// A car/engine type record (`EngineTypes/*.car`, version 1002, 272 bytes).
#[derive(Debug, Clone, PartialEq)]
pub struct CarType {
    /// Flag u32; 1 means the record references a `.lco` by name.
    pub flag: u32,
    pub string_id: u32,
    /// Display name (`242 A1`).
    pub display_name: String,
    /// `.lco` reference name (`242_A1_L`).
    pub lco_name: String,
    /// Id name (`242_A1L`).
    pub id_name: String,
    /// Short name (`242_A1`).
    pub short_name: String,
    pub tail_name: String,
    pub tail_u32s: [u32; 4],
}

impl CarType {
    pub fn parse(path: &Path, d: &[u8]) -> Result<CarType, TypedError> {
        if d.len() != 272 {
            return Err(TypedError::new(
                path,
                format!("expected 272 bytes, found {}", d.len()),
            ));
        }
        let version = i32_at(d, 0);
        if version != 1002 {
            return Err(TypedError::new(
                path,
                format!("unsupported .car version {version}"),
            ));
        }
        Ok(CarType {
            flag: u32_at(d, 4),
            string_id: u32_at(d, 8),
            display_name: fixed_name(&d[12..72]).map_err(|e| TypedError::new(path, e))?,
            lco_name: fixed_name(&d[72..132]).map_err(|e| TypedError::new(path, e))?,
            id_name: fixed_name(&d[132..162]).map_err(|e| TypedError::new(path, e))?,
            short_name: fixed_name(&d[162..192]).map_err(|e| TypedError::new(path, e))?,
            tail_name: fixed_name(&d[192..252]).map_err(|e| TypedError::new(path, e))?,
            tail_u32s: [
                u32_at(d, 252),
                u32_at(d, 258),
                u32_at(d, 264),
                u32_at(d, 268),
            ],
        })
    }
}

/// A whitespace-separated text table with `;` comment lines
/// (`TreeTypes.txt`, `AirplaneTypes.txt`).
#[derive(Debug, Clone, PartialEq)]
pub struct TextTable {
    pub rows: Vec<Vec<String>>,
}

impl TextTable {
    pub fn parse(path: &Path, text: &str) -> Result<TextTable, TypedError> {
        let mut rows = Vec::new();
        for line in text.lines() {
            let line = line.trim_end_matches(['\r', ';', ' ', '\t']);
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            let row: Vec<String> = line.split_whitespace().map(str::to_owned).collect();
            if row.is_empty() {
                continue;
            }
            if !row.iter().all(|c| c.chars().all(|c| c.is_ascii_graphic())) {
                return Err(TypedError::new(path, format!("odd row: {line:?}")));
            }
            rows.push(row);
        }
        if rows.is_empty() {
            return Err(TypedError::new(path, "no rows found"));
        }
        Ok(TextTable { rows })
    }
}

/// A language file (`Language/RT3.lng`): `id text` lines with `;` comments.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LanguageFile {
    pub strings: BTreeMap<i64, String>,
}

impl LanguageFile {
    pub fn parse(path: &Path, text: &str) -> Result<LanguageFile, TypedError> {
        let mut strings = BTreeMap::new();
        for line in text.lines() {
            let line = line.trim_end_matches('\r').trim_start_matches([' ', '\t']);
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            // ids are leading digits; text may follow without a separator
            // (one shipped line is `1"Warning"`).
            let digits = line.len() - line.trim_start_matches(|c: char| c.is_ascii_digit()).len();
            if digits == 0 {
                return Err(TypedError::new(path, format!("bad id line: {line:?}")));
            }
            let id: i64 = line[..digits].parse().unwrap();
            let text = line[digits..].trim_start().trim_end().replace("\\n", "\n");
            strings.insert(id, text);
        }
        if strings.is_empty() {
            return Err(TypedError::new(path, "no strings found"));
        }
        Ok(LanguageFile { strings })
    }
}

/// `Configuration/engine.cfg`: header 0x41B, width, height and a fixed
/// field map cross-checked against rt3conf (MIT). Only the semantic head
/// fields are exposed; the rest is validated by length.
#[derive(Debug, Clone, PartialEq)]
pub struct EngineConfig {
    pub width: u32,
    pub height: u32,
}

impl EngineConfig {
    pub const LEN: usize = 980;

    pub fn parse(path: &Path, d: &[u8]) -> Result<EngineConfig, TypedError> {
        if d.len() != Self::LEN {
            return Err(TypedError::new(
                path,
                format!("expected {} bytes, found {}", Self::LEN, d.len()),
            ));
        }
        let header = u32_at(d, 0);
        if header != 0x41B {
            return Err(TypedError::new(
                path,
                format!("bad engine.cfg header {header:#x}"),
            ));
        }
        Ok(EngineConfig {
            width: u32_at(d, 4),
            height: u32_at(d, 8),
        })
    }
}

/// `Configuration/game.cfg`: leading u32 plus payload (the field's exact
/// semantics are open; it does not encode the file length).
#[derive(Debug, Clone, PartialEq)]
pub struct GameConfig {
    pub leading: u32,
}

impl GameConfig {
    pub fn parse(path: &Path, d: &[u8]) -> Result<GameConfig, TypedError> {
        if d.len() < 4 {
            return Err(TypedError::new(path, "truncated"));
        }
        Ok(GameConfig {
            leading: u32_at(d, 0),
        })
    }
}

/// A directory scan result for one family.
pub struct FamilyScan<T> {
    pub records: Vec<(String, T)>,
    pub failures: Vec<TypedError>,
}

/// Parses every file matching `suffix` in `dir` with `parse`.
pub fn scan_family<T>(
    dir: &Path,
    suffix: &str,
    mut parse: impl FnMut(&Path, &[u8]) -> Result<T, TypedError>,
) -> FamilyScan<T> {
    let mut records = Vec::new();
    let mut failures = Vec::new();
    let mut paths: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case(suffix))
                    .unwrap_or(false)
            })
            .collect(),
        Err(e) => {
            failures.push(TypedError::new(dir, e));
            Vec::new()
        }
    };
    paths.sort();
    for path in paths {
        match std::fs::read(&path) {
            Ok(bytes) => match parse(&path, &bytes) {
                Ok(record) => records.push((
                    path.file_name().unwrap().to_string_lossy().into_owned(),
                    record,
                )),
                Err(e) => failures.push(e),
            },
            Err(e) => failures.push(TypedError::new(&path, e)),
        }
    }
    FamilyScan { records, failures }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err(_path: &Path, r: Result<(), TypedError>) {
        if let Err(e) = r {
            panic!("{e}");
        }
    }

    #[test]
    fn cty_roundtrip() {
        let mut d = vec![0u8; 62];
        d[0..4].copy_from_slice(&1002i32.to_le_bytes());
        d[4..9].copy_from_slice(b"Grain");
        d[24..28].copy_from_slice(&25000.0f32.to_le_bytes());
        d[28..32].copy_from_slice(&0.2f32.to_le_bytes());
        d[32] = 3;
        d[33..37].copy_from_slice(&0x708u32.to_le_bytes());
        d[37..40].copy_from_slice(b"Box");
        d[58..62].copy_from_slice(&14i32.to_le_bytes());
        let cty = CargoType::parse(Path::new("Grain.cty"), &d).unwrap();
        assert_eq!(cty.name, "Grain");
        assert_eq!(cty.price, 25000.0);
        assert_eq!(cty.factor, 0.2);
        assert_eq!(cty.car_type, "BOX");
        assert_eq!(cty.index, 14);
    }

    #[test]
    fn lco_roundtrip() {
        let mut d = vec![0xCCu8; 91];
        d[0..4].copy_from_slice(&2005i32.to_le_bytes());
        d[4..12].copy_from_slice(b"TEST_L\0\0");
        d[34..38].copy_from_slice(&95.0f32.to_le_bytes());
        d[83..87].copy_from_slice(&2i32.to_le_bytes());
        d[87..91].copy_from_slice(&9i32.to_le_bytes());
        let lco = Locomotive::parse(Path::new("TEST_L.lco"), &d).unwrap();
        assert_eq!(lco.name, "TEST_L");
        assert_eq!(lco.floats[0], 95.0);
        assert_eq!(lco.clamped, [2, 9]);
    }

    #[test]
    fn car_and_bca_shape() {
        let mut d = vec![0xCDu8; 272];
        d[0..4].copy_from_slice(&1002i32.to_le_bytes());
        d[4..8].copy_from_slice(&1u32.to_le_bytes());
        d[12..21].copy_from_slice(b"Test Car\0");
        d[72..79].copy_from_slice(b"Test_L\0");
        d[132..138].copy_from_slice(b"TESTL\0");
        d[162..167].copy_from_slice(b"Test\0");
        d[192..197].copy_from_slice(b"Tail\0");
        // Tail layout: u32@252, u8, u8, u32@258 (intro year), u8, u8,
        // u32@264 (end year), u32@268.
        d[252..256].copy_from_slice(&6u32.to_le_bytes());
        d[256] = 1;
        d[257] = 1;
        d[258..262].copy_from_slice(&1946u32.to_le_bytes());
        d[262] = 1;
        d[263] = 1;
        d[264..268].copy_from_slice(&1960u32.to_le_bytes());
        d[268..272].copy_from_slice(&200.0f32.to_le_bytes());
        let car = CarType::parse(Path::new("Test.car"), &d).unwrap();
        assert_eq!(
            (car.flag, car.display_name.as_str(), car.lco_name.as_str()),
            (1, "Test Car", "Test_L")
        );
        assert_eq!(car.tail_u32s, [6, 1946, 1960, 200.0f32.to_bits()]);

        // bca: header 55 + 1×188 subrecord with a script + 169 tail = 412.
        let mut b = vec![0xCCu8; 55 + 188 + 169];
        b[0..4].copy_from_slice(&1010i32.to_le_bytes());
        b[4..10].copy_from_slice(b"Mill\0\0");
        b[0x22..0x26].copy_from_slice(&100000u32.to_le_bytes());
        b[0x33..0x37].copy_from_slice(&1u32.to_le_bytes());
        b[55 + 10..55 + 23].copy_from_slice(b"{Demand Only}");
        let bca = BuildingBehavior::parse(Path::new("Mill.bca"), &b).unwrap();
        assert_eq!(bca.count, 1);
        assert_eq!(bca.cost, 100000);
        assert_eq!(bca.scripts, vec!["{Demand Only}".to_string()]);
    }

    #[test]
    fn lng_and_tables() {
        let lng = LanguageFile::parse(
            Path::new("RT3.lng"),
            "; comment\n1000 Hello\n1001 World\\nAgain\n",
        )
        .unwrap();
        assert_eq!(lng.strings[&1000], "Hello");
        assert_eq!(lng.strings[&1001], "World\nAgain");
        let trees = TextTable::parse(
            Path::new("TreeTypes.txt"),
            "; h\nbush1_1A B .5\r\n\r\noak T .9\r\n",
        )
        .unwrap();
        assert_eq!(trees.rows.len(), 2);
        assert_eq!(trees.rows[0], ["bush1_1A", "B", ".5"]);
    }

    #[test]
    fn cfg_parsers() {
        let mut e = vec![0u8; 980];
        e[0..4].copy_from_slice(&0x41Bu32.to_le_bytes());
        e[4..8].copy_from_slice(&1920u32.to_le_bytes());
        e[8..12].copy_from_slice(&1080u32.to_le_bytes());
        let cfg = EngineConfig::parse(Path::new("engine.cfg"), &e).unwrap();
        assert_eq!((cfg.width, cfg.height), (1920, 1080));

        let mut g = Vec::new();
        g.extend_from_slice(&1065u32.to_le_bytes());
        g.extend_from_slice(&[0u8; 1065]);
        assert_eq!(
            GameConfig::parse(Path::new("game.cfg"), &g)
                .unwrap()
                .leading,
            1065
        );
    }

    #[test]
    fn short_inputs_error_instead_of_panicking() {
        for len in [0, 1, 3] {
            let d = vec![0xCCu8; len];
            assert!(
                BuildingType::parse(Path::new("x.bty"), &d).is_err(),
                "bty len {len}"
            );
            assert!(
                BuildingBehavior::parse(Path::new("x.bca"), &d).is_err(),
                "bca len {len}"
            );
        }
    }

    /// Install-backed check: parses every typed file when `RT3_HOME` is set.
    /// Self-skips otherwise.
    #[test]
    fn parses_every_shipped_typed_file() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed typed-data check");
            return;
        };
        let root = Path::new(&home);
        let data = root.join("Data");

        let cargo = scan_family(&data.join("CargoTypes"), "cty", CargoType::parse);
        assert_eq!(cargo.records.len(), 41, "cargo types");
        assert!(
            cargo.failures.is_empty(),
            "{:?} of {} failed",
            cargo.failures.first(),
            cargo.failures.len()
        );

        let buildings = scan_family(&data.join("BuildingTypes"), "bty", BuildingType::parse);
        assert_eq!(buildings.records.len(), 190, "building types");
        assert!(buildings.failures.is_empty());

        let behaviors = scan_family(&data.join("BuildingTypes"), "bca", BuildingBehavior::parse);
        assert_eq!(behaviors.records.len(), 67, "building behaviors");
        assert!(behaviors.failures.is_empty());

        let locos = scan_family(&data.join("EngineTypes"), "lco", Locomotive::parse);
        assert_eq!(locos.records.len(), 66, "locomotives");
        assert!(locos.failures.is_empty());

        let cars = scan_family(&data.join("EngineTypes"), "car", CarType::parse);
        assert_eq!(cars.records.len(), 145, "car types");
        assert!(cars.failures.is_empty());

        // RT3.lng is 8-bit text (Windows codepage); decode lossily.
        let lng_bytes = std::fs::read(data.join("Language/RT3.lng")).unwrap();
        let lng_text = String::from_utf8_lossy(&lng_bytes);
        let lng = LanguageFile::parse(&data.join("Language/RT3.lng"), &lng_text).unwrap();
        assert!(!lng.strings.is_empty());

        let trees = std::fs::read_to_string(data.join("TreeTypes/TreeTypes.txt")).unwrap();
        err(&data, TextTable::parse(&data, &trees).map(|_| ()));
        let planes = std::fs::read_to_string(data.join("Other/AirplaneTypes.txt")).unwrap();
        err(&data, TextTable::parse(&data, &planes).map(|_| ()));

        let engine_bytes = std::fs::read(data.join("Configuration/engine.cfg")).unwrap();
        err(&data, EngineConfig::parse(&data, &engine_bytes).map(|_| ()));
        let game_bytes = std::fs::read(data.join("Configuration/game.cfg")).unwrap();
        err(&data, GameConfig::parse(&data, &game_bytes).map(|_| ()));
    }
}
