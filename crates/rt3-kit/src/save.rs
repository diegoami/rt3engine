//! Original `.gmc`/`.gms` save-game structural inventory (milestone 4).

use crate::buildings::BuildingCollection;
use crate::cities::CityCollection;
use crate::companies::CompanyCollection;
use crate::events::EventCollection;
use crate::map::GmpMap;
use crate::players::PlayerCollection;
use crate::terrain::{read_framed, HeightGrid};
use crate::tiles::TileSections;

const SOURCE_MAP_OFFSET: usize = 0x73d0;
const SOURCE_MAP_REGION_LEN: usize = 0x7403 - SOURCE_MAP_OFFSET;

pub struct SaveGame {
    pub file_bytes: usize,
    pub sha256: [u8; 32],
    pub source_map: String,
    pub terrain: HeightGrid,
    pub tiles: TileSections,
    pub cities: CityCollection,
    pub buildings: BuildingCollection,
    pub companies: CompanyCollection,
    pub players: PlayerCollection,
    pub events: EventCollection,
}

impl SaveGame {
    pub fn sha256_hex(&self) -> String {
        self.sha256.iter().map(|b| format!("{b:02x}")).collect()
    }
}

/// Reads the save-populated source-map basename at the fixed offset observed
/// in both independent version-9701 fixtures. The 51-byte region ends where
/// the following display-label field begins in the controlled save.
fn source_map_name(bytes: &[u8]) -> Option<String> {
    let field = bytes.get(SOURCE_MAP_OFFSET..SOURCE_MAP_OFFSET + SOURCE_MAP_REGION_LEN)?;
    let end = field.iter().position(|&byte| byte == 0)?;
    let name = std::str::from_utf8(&field[..end]).ok()?;
    if name.is_empty()
        || !name.to_ascii_lowercase().ends_with(".gmp")
        || !name.bytes().all(|byte| (0x20..=0x7e).contains(&byte))
        || name.contains('/')
        || name.contains('\\')
    {
        return None;
    }
    Some(name.to_owned())
}

/// Parses the version-9701 save container and all currently proven
/// structural collections. Any missing proven section rejects the save.
pub fn from_bytes(bytes: &[u8]) -> Option<SaveGame> {
    if GmpMap::peek_version(bytes) != Some(9701) {
        return None;
    }
    let source_map = source_map_name(bytes)?;
    let terrain = read_framed(bytes)?;
    let tiles = crate::tiles::read(bytes, &terrain)?;
    let cities = crate::cities::read(bytes)?;
    let buildings = crate::buildings::read(bytes)?;
    let companies = crate::companies::read(bytes)?;
    let players = crate::players::read(bytes)?;
    let events = crate::events::read(bytes)?;
    Some(SaveGame {
        file_bytes: bytes.len(),
        sha256: sha256(bytes),
        source_map,
        terrain,
        tiles,
        cities,
        buildings,
        companies,
        players,
        events,
    })
}

/// Dependency-free SHA-256 used only to identify a supplied save fixture.
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut state = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut data = bytes.to_vec();
    data.push(0x80);
    while data.len() & 63 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&(bytes.len() as u64).wrapping_mul(8).to_be_bytes());
    for block in data.as_chunks::<64>().0 {
        let mut w = [0u32; 64];
        for (i, word) in block.as_chunks::<4>().0.iter().enumerate() {
            w[i] = u32::from_be_bytes(*word);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut out = [0u8; 32];
    for (i, word) in state.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn sha256_known_vector() {
        let hash = sha256(b"abc");
        assert_eq!(
            hex(&hash),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    fn hex(hash: &[u8; 32]) -> String {
        hash.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn validates_source_map_field() {
        let mut bytes = vec![0u8; SOURCE_MAP_OFFSET + SOURCE_MAP_REGION_LEN];
        bytes[SOURCE_MAP_OFFSET..SOURCE_MAP_OFFSET + 17].copy_from_slice(b"British Isles.gmp");
        assert_eq!(
            source_map_name(&bytes).as_deref(),
            Some("British Isles.gmp")
        );

        bytes[SOURCE_MAP_OFFSET..SOURCE_MAP_OFFSET + 17].copy_from_slice(b"not-a-map.txt\0\0\0\0");
        assert_eq!(source_map_name(&bytes), None);

        bytes[SOURCE_MAP_OFFSET..SOURCE_MAP_OFFSET + 17].copy_from_slice(b"../ Isles.gmp\0\0\0\0");
        assert_eq!(source_map_name(&bytes), None);

        bytes.fill(0x20);
        assert_eq!(source_map_name(&bytes), None);
    }

    #[test]
    fn rejects_missing_sections() {
        let mut bytes = vec![0u8; 16];
        bytes[..4].copy_from_slice(&9701u32.to_le_bytes());
        assert!(from_bytes(&bytes).is_none());
    }

    /// Resolves a fixture named by an environment variable. The evidence docs
    /// give repository-relative values; those resolve from the repository
    /// root, not from the crate directory the tests run in.
    fn fixture(name: &str) -> Option<std::path::PathBuf> {
        let path = std::path::PathBuf::from(std::env::var_os(name)?);
        if path.is_absolute() {
            return Some(path);
        }
        let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let candidate = repo.join(&path);
        Some(if candidate.exists() { candidate } else { path })
    }

    #[test]
    fn reads_original_save() {
        let Some(path) = fixture("RT3_SAVE") else {
            eprintln!("RT3_SAVE unset; skipping original-save check");
            return;
        };
        let bytes = fs::read(path).unwrap();
        let save = from_bytes(&bytes).expect("original save");
        assert_eq!(save.file_bytes, 8_490_851);
        assert_eq!(save.source_map, "Go West!.gmp");
        assert_eq!((save.terrain.rows, save.terrain.cols), (257, 385));
        assert_eq!(save.tiles.records.len(), 6_144);
        assert_eq!(save.cities.records.len(), 59);
        assert_eq!(save.buildings.records.len(), 410);
        assert_eq!(save.companies.records.len(), 0);
        assert_eq!(save.players.records.len(), 4);
        assert_eq!(
            crate::players::name(&save.players.records[3]),
            Some("Unassigned")
        );
        assert_eq!(save.events.records.len(), 17);
        assert_eq!(
            save.sha256_hex(),
            "3c11cbbdde844273c499086a8a4c23df9e1cfe9837910696fa13bf32d121d99b"
        );
    }

    #[test]
    fn reads_independent_control_save() {
        let Some(path) = fixture("RT3_CONTROL_SAVE") else {
            eprintln!("RT3_CONTROL_SAVE unset; skipping controlled-save check");
            return;
        };
        let bytes = fs::read(path).unwrap();
        let save = from_bytes(&bytes).expect("independent controlled save");
        assert_eq!(save.file_bytes, 10_285_120);
        assert_eq!(save.source_map, "British Isles.gmp");
        assert_eq!((save.terrain.rows, save.terrain.cols), (513, 321));
        assert_eq!(save.tiles.records.len(), 10_240);
        assert_eq!(save.cities.records.len(), 68);
        assert_eq!(save.buildings.records.len(), 520);
        assert_eq!(save.companies.records.len(), 3);
        let founded_years = save
            .companies
            .records
            .iter()
            .map(crate::companies::founded_year)
            .collect::<Option<Vec<_>>>();
        assert_eq!(founded_years, Some(vec![1830, 1830, 1829]));
        assert_eq!(
            crate::companies::name(&save.companies.records[2]),
            Some("Great Republic")
        );
        assert_eq!(save.players.records.len(), 3);
        assert_eq!(
            crate::players::name(&save.players.records[0]),
            Some("George Hudson")
        );
        assert_eq!(save.events.records.len(), 39);
        assert_eq!(
            save.sha256_hex(),
            "f892bc28ea1fb3bb1ff60f3ed443e82de7818d3fd7a7c26a93e974cfb69ac6bc"
        );
    }
}
