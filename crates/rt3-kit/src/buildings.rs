//! Placed building/industry collections (`0x3315` / `0x36b1`).

use crate::sparse::{SparseBlobCollection, SparseBlobRecord, Spec};

pub type BuildingRecord = SparseBlobRecord;
pub type BuildingCollection = SparseBlobCollection;
pub const BUILDING_BLOB_BYTES: usize = 993;

pub fn read(bytes: &[u8]) -> Option<BuildingCollection> {
    crate::sparse::read(
        bytes,
        Spec {
            outer: 0x3315,
            inner: 0x36b1,
            end: 0x36b2,
            preamble: None,
            blob_bytes: BUILDING_BLOB_BYTES,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reads_all_shipped_building_collections() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed building check");
            return;
        };
        let mut paths = Vec::new();
        for dir in ["maps", "Data/Tutorial"] {
            paths.extend(
                fs::read_dir(std::path::Path::new(&home).join(dir))
                    .unwrap()
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("gmp")),
            );
        }
        paths.sort();
        assert_eq!(paths.len(), 43);
        let mut total = 0;
        let mut nonempty = 0;
        for path in paths {
            let bytes = fs::read(&path).unwrap();
            let collection = read(&bytes).expect("building collection");
            nonempty += (!collection.records.is_empty()) as usize;
            total += collection.records.len();
            for record in collection.records {
                assert_eq!(record.data.len(), BUILDING_BLOB_BYTES);
                let label = record.label_bytes().unwrap();
                assert!(!label.is_empty());
                assert!(label[0] >= 32);
            }
        }
        assert_eq!(total, 2_044);
        assert_eq!(nonempty, 18);
    }
}
