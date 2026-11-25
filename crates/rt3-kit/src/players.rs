//! Player collections (`0x32d0` / `0x5209`) from shipped maps and saves.
//!
//! Original loader: `RT3.exe 0x004777e0` (`gpdPlayerDB`). Fixed records are
//! 3,243 bytes; the 31-byte raw name field begins at record offset 8.
//! Biographies and other gameplay fields are not interpreted.

use crate::fixed::{FixedBlobCollection, FixedBlobRecord, Spec};

pub type PlayerRecord = FixedBlobRecord;
pub type PlayerCollection = FixedBlobCollection;
pub const PLAYER_BLOB_BYTES: usize = 3_243;
pub const PLAYER_NAME_OFFSET: usize = 8;
pub const PLAYER_NAME_BYTES: usize = 31;

pub fn name_bytes(record: &PlayerRecord) -> Option<&[u8]> {
    record
        .data
        .get(PLAYER_NAME_OFFSET..PLAYER_NAME_OFFSET + PLAYER_NAME_BYTES)
}

pub fn name(record: &PlayerRecord) -> Option<&str> {
    let used = name_bytes(record)?.split(|byte| *byte == 0).next()?;
    if used.is_empty() || !used.iter().all(|byte| (32..127).contains(byte)) {
        return None;
    }
    std::str::from_utf8(used).ok()
}

pub fn read(bytes: &[u8]) -> Option<PlayerCollection> {
    crate::fixed::read(
        bytes,
        Spec {
            outer: 0x32d0,
            inner: 0x5209,
            record_bytes: PLAYER_BLOB_BYTES,
            nominal_slots: 4,
            header_flag: Some(0),
            end: [0x520a, 0x520b],
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reads_all_shipped_player_collections() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed player check");
            return;
        };
        let mut paths = Vec::new();
        for dir in ["maps", "Data/Tutorial"] {
            paths.extend(
                fs::read_dir(std::path::Path::new(&home).join(dir))
                    .unwrap()
                    .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                    .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("gmp")),
            );
        }
        paths.sort();
        assert_eq!(paths.len(), 43);
        let mut current = 0;
        let mut total = 0;
        for path in paths {
            let bytes = fs::read(&path).unwrap();
            current += 1;
            let collection = read(&bytes).expect("player collection");
            total += collection.records.len();
            for record in collection.records {
                assert_eq!(record.data.len(), PLAYER_BLOB_BYTES);
                assert!(name(&record).is_some());
                let name = name_bytes(&record).unwrap();
                let used = name.split(|byte| *byte == 0).next().unwrap();
                assert!(!used.is_empty());
                assert!(used.iter().all(|byte| (32..127).contains(byte)));
            }
        }
        assert_eq!(current, 43);
        assert_eq!(total, 145);
    }
}
