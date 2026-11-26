//! Scenario event collections (`0x32dc` / `0x4e99`).

use crate::sparse::{SparseBlobCollection, SparseBlobRecord, Spec};

pub type EventRecord = SparseBlobRecord;
pub type EventCollection = SparseBlobCollection;
pub const EVENT_BLOB_BYTES: usize = 2191;
pub const EVENT_NAME_OFFSET: usize = 24;
pub const EVENT_NAME_BYTES: usize = 200;

pub fn name_bytes(record: &EventRecord) -> Option<&[u8]> {
    record
        .data
        .get(EVENT_NAME_OFFSET..EVENT_NAME_OFFSET + EVENT_NAME_BYTES)
}

pub fn read(bytes: &[u8]) -> Option<EventCollection> {
    crate::sparse::read(
        bytes,
        Spec {
            outer: 0x32dc,
            inner: 0x4e99,
            end: 0x4e9a,
            preamble: Some(1001),
            blob_bytes: EVENT_BLOB_BYTES,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn push_u32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn reads_level_payload_and_event_name() {
        let mut bytes = vec![0u8; 0x3000];
        push_u32(&mut bytes, 0x32dc);
        push_u32(&mut bytes, 0x4e99);
        push_u32(&mut bytes, 1001);
        for word in [0, 6, 10, 20, 3, 1, 0] {
            push_u32(&mut bytes, word);
        }
        let levels = [(1, 0, 0), (0, 0, 0), (1, 4, 10), (0, 0, 0)];
        for (a, b, c) in levels {
            push_u32(&mut bytes, a);
            push_u32(&mut bytes, b);
            push_u32(&mut bytes, c);
        }
        bytes.extend(0..36); // level 2: capacity × 12 bytes
        bytes.push(0b110); // only slot 0 active
        bytes.extend_from_slice(&[9; 6]);
        bytes.extend_from_slice(&(EVENT_BLOB_BYTES as u16).to_le_bytes());
        let mut blob = vec![0u8; EVENT_BLOB_BYTES];
        blob[EVENT_NAME_OFFSET..EVENT_NAME_OFFSET + 9].copy_from_slice(b"SP - TEST");
        bytes.extend_from_slice(&blob);
        push_u32(&mut bytes, 0x4e9a);
        let collection = read(&bytes).expect("event collection");
        assert_eq!(collection.records.len(), 1);
        assert!(name_bytes(&collection.records[0])
            .unwrap()
            .starts_with(b"SP - TEST"));
        for end in 0..bytes.len() {
            assert!(read(&bytes[..end]).is_none(), "accepted event prefix {end}");
        }
    }

    #[test]
    fn reads_all_shipped_event_collections() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed event check");
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
        for path in paths {
            let bytes = fs::read(&path).unwrap();
            let collection = read(&bytes).expect("event collection");
            total += collection.records.len();
            for record in collection.records {
                assert_eq!(record.data.len(), EVENT_BLOB_BYTES);
                let name = name_bytes(&record).unwrap();
                let used = name.split(|b| *b == 0).next().unwrap();
                assert!(!used.is_empty());
            }
        }
        assert_eq!(total, 1_169);
    }
}
