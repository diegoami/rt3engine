//! Reader for Railroad Tycoon 3 `.gmp` scenario maps (prelude, milestone 3).
//!
//! Proven so far (see `docs/evidence/gmp-format-v1.md`; claim level
//! **parse — prelude only**):
//!
//! ```text
//! u32 version = 9701
//! map record, 20,268 bytes:
//!   extent/config words at +8..0x27 (semantics open; 320/384-class values)
//!   scenario intro text at +0x78 (NUL-terminated prose — never committed)
//! u32 skipped, then two 3,001-byte auxiliary records
//!   (0xCC debug fill tolerated — largely unused in shipped files)
//! map-specific tail follows (terrain + objects; grammar open)
//! ```
//!
//! The save-side framing is proven (`RT3.exe` `0x00444dd0` writes this
//! record via `0x00531340` tag writes and a `0x00531030` raw write of
//! 20,268 bytes; the reader mirrors it). The tail — including the
//! terrain heights (`rt3-kit::terrain`, coarse grid proven) and objects —
//! is only partially decoded; the reader exposes its offset and length
//! only.

use std::fs;
use std::path::{Path, PathBuf};

/// Total size of the fixed map record after the version word.
pub const MAP_RECORD_LEN: usize = 20_268;
/// Offset of the scenario text inside the map record.
pub const TEXT_OFFSET: usize = 0x78;

/// An error raised while reading a `.gmp` map.
#[derive(Debug)]
pub struct MapError {
    path: PathBuf,
    detail: String,
}

impl MapError {
    /// The failure detail, without the machine-specific path.
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl std::fmt::Display for MapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.detail)
    }
}

impl std::error::Error for MapError {}

/// A parsed `.gmp` map prelude.
#[derive(Debug, Clone, PartialEq)]
pub struct GmpMap {
    /// Format version (9701 in every shipped map).
    pub version: u32,
    /// The eight extent/config words from the map record (semantics open).
    pub extents: [u32; 8],
    /// Scenario intro text length in bytes (the prose itself is original
    /// content and is not retained by tools).
    pub text_len: usize,
    /// File offset where the map-specific tail begins (after the version,
    /// map record, one skipped word and the two auxiliary records).
    pub tail_offset: u64,
    /// Tail length in bytes.
    pub tail_len: u64,
    /// Total file size.
    pub file_len: u64,
}

impl GmpMap {
    /// Opens and parses the prelude of a `.gmp` map.
    pub fn open(path: &Path) -> Result<GmpMap, MapError> {
        let bytes = fs::read(path).map_err(|e| MapError {
            path: path.to_path_buf(),
            detail: format!("{e}"),
        })?;
        Self::from_bytes(path, &bytes)
    }

    /// Peeks at the version word without parsing the prelude. Older
    /// shipped maps use 9609/9610 with a different layout (unsupported
    /// here; the ladder is recorded in the evidence doc).
    pub fn peek_version(bytes: &[u8]) -> Option<u32> {
        (bytes.len() >= 4).then(|| u32::from_le_bytes(bytes[0..4].try_into().unwrap()))
    }

    /// Parses a map image held in memory.
    pub fn from_bytes(path: &Path, bytes: &[u8]) -> Result<GmpMap, MapError> {
        let err = |detail: String| MapError {
            path: path.to_path_buf(),
            detail,
        };
        if bytes.len() < 4 + MAP_RECORD_LEN {
            return Err(err(format!(
                "truncated: {} bytes, need at least {}",
                bytes.len(),
                4 + MAP_RECORD_LEN
            )));
        }
        let version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        if version != 9701 {
            return Err(err(format!("unsupported .gmp version {version}")));
        }
        let record = &bytes[4..4 + MAP_RECORD_LEN];
        let mut extents = [0u32; 8];
        for (i, slot) in extents.iter_mut().enumerate() {
            let off = 8 + 4 * i;
            *slot = u32::from_le_bytes(record[off..off + 4].try_into().unwrap());
        }
        let text_field = &record[TEXT_OFFSET..];
        let Some(nul) = text_field.iter().position(|b| *b == 0) else {
            return Err(err("scenario text is not NUL-terminated".to_string()));
        };
        // The text is 8-bit (Windows codepage) prose; only its length is
        // retained, never the content.
        let text_len = nul;
        // version (4) + record (20,268) + skip (4) + 2 × (3,001)
        let tail_offset: u64 = 4 + MAP_RECORD_LEN as u64 + 4 + 2 * 3001;
        if (bytes.len() as u64) < tail_offset {
            return Err(err(format!(
                "truncated before the tail: {} bytes, need {tail_offset}",
                bytes.len()
            )));
        }
        Ok(GmpMap {
            version,
            extents,
            text_len,
            tail_offset,
            tail_len: bytes.len() as u64 - tail_offset,
            file_len: bytes.len() as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(text: &str) -> Vec<u8> {
        let mut d = vec![0u8; 4 + MAP_RECORD_LEN + 4 + 2 * 3001 + 128];
        d[0..4].copy_from_slice(&9701u32.to_le_bytes());
        // extents at record+8
        d[12..16].copy_from_slice(&320u32.to_le_bytes());
        d[16..20].copy_from_slice(&448u32.to_le_bytes());
        // text at record+0x78 (file offset 4+0x78)
        let t = text.as_bytes();
        d[4 + TEXT_OFFSET..4 + TEXT_OFFSET + t.len()].copy_from_slice(t);
        d
    }

    #[test]
    fn parses_fixture() {
        let d = fixture("A test brief.");
        let map = GmpMap::from_bytes(Path::new("test.gmp"), &d).unwrap();
        assert_eq!(map.version, 9701);
        assert_eq!(map.extents[0], 320);
        assert_eq!(map.extents[1], 448);
        assert_eq!(map.text_len, "A test brief.".len());
        assert_eq!(map.tail_offset, 4 + 20_268 + 4 + 6_002);
        assert_eq!(map.tail_len, 128);
    }

    #[test]
    fn rejects_bad_version_and_truncation() {
        let mut d = fixture("x");
        d[0..4].copy_from_slice(&9700u32.to_le_bytes());
        assert!(GmpMap::from_bytes(Path::new("v.gmp"), &d).is_err());
        assert!(GmpMap::from_bytes(Path::new("t.gmp"), &d[..100]).is_err());
    }

    /// Install-backed check: parses every shipped map. Self-skips without
    /// `RT3_HOME`.
    #[test]
    fn parses_every_shipped_map() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed map check");
            return;
        };
        let mut maps = Vec::new();
        for dir in ["maps", "Data/Tutorial"] {
            let dir = Path::new(&home).join(dir);
            if let Ok(rd) = fs::read_dir(&dir) {
                let mut paths: Vec<_> = rd
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| {
                        p.extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e.eq_ignore_ascii_case("gmp"))
                            .unwrap_or(false)
                    })
                    .collect();
                paths.sort();
                maps.extend(paths);
            }
        }
        assert_eq!(maps.len(), 43, "maps/ + tutorial maps");
        let mut parsed = 0;
        let mut legacy = 0;
        for path in &maps {
            let bytes = fs::read(path).unwrap();
            match GmpMap::peek_version(&bytes) {
                Some(v @ 9609..=9610) => {
                    assert!((9609..=9610).contains(&v), "{}", path.display());
                    legacy += 1;
                }
                Some(9701) => {
                    let map = GmpMap::from_bytes(path, &bytes).unwrap_or_else(|e| panic!("{e}"));
                    assert!(map.tail_len > 0, "{}", path.display());
                    parsed += 1;
                }
                other => panic!("{}: unexpected version {other:?}", path.display()),
            }
        }
        assert_eq!((parsed, legacy), (40, 3), "current + legacy maps");
    }
}
