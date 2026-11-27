//! Rail infrastructure collections: stations, cars, trains, track and track
//! lines.
//!
//! Tags, record sizes and framings are established in issue #62 from the
//! original reader (`RT3.exe FUN_00443a50`) and a populated save. Stations,
//! trains and track lines use the fixed-record collection; cars and track use
//! the sparse blob collection. Each section may carry an opaque trailer block
//! between its ending tags; its meaning is not inferred.

use crate::fixed::{self, FixedBlobCollection, Spec as FixedSpec};
use crate::sparse::{self, SparseBlobCollection, Spec as SparseSpec};

pub const STATION_RECORD_BYTES: usize = 1549;
pub const TRAIN_RECORD_BYTES: usize = 473;
pub const TRACK_LINE_RECORD_BYTES: usize = 36;

pub fn stations(bytes: &[u8]) -> Option<FixedBlobCollection> {
    fixed::read(
        bytes,
        FixedSpec {
            outer: 0x332a,
            inner: 0x4a9d,
            record_bytes: STATION_RECORD_BYTES,
            header_record_bytes: STATION_RECORD_BYTES as u32,
            nominal_slots: 10,
            header_flag: Some(0),
            max_capacity: 16,
            end_tags: 2,
            trailer: true,
            end: [0x4a3a, 0x4a3b],
        },
    )
}

pub fn cars(bytes: &[u8]) -> Option<SparseBlobCollection> {
    sparse::read(
        bytes,
        SparseSpec {
            outer: 0x332b,
            inner: 0x4a39,
            end: 0x4a3a,
            preamble: None,
            blob_bytes: None,
            trailer_end: Some(0x4a3b),
            header2: Some(10),
            header3: Some(20),
        },
    )
}

pub fn trains(bytes: &[u8]) -> Option<FixedBlobCollection> {
    fixed::read(
        bytes,
        FixedSpec {
            outer: 0x32cd,
            inner: 0x5209,
            record_bytes: TRAIN_RECORD_BYTES,
            header_record_bytes: TRAIN_RECORD_BYTES as u32,
            nominal_slots: 50,
            header_flag: None,
            max_capacity: 64,
            end_tags: 2,
            trailer: true,
            end: [0x520a, 0x520b],
        },
    )
}

pub fn track(bytes: &[u8]) -> Option<SparseBlobCollection> {
    sparse::read(
        bytes,
        SparseSpec {
            outer: 0x32cf,
            inner: 0x38a5,
            end: 0x38a6,
            preamble: None,
            blob_bytes: None,
            trailer_end: Some(0x38a7),
            header2: None,
            header3: None,
        },
    )
}

pub fn track_lines(bytes: &[u8]) -> Option<FixedBlobCollection> {
    fixed::read(
        bytes,
        FixedSpec {
            outer: 0x32d3,
            inner: 0x396d,
            record_bytes: TRACK_LINE_RECORD_BYTES,
            header_record_bytes: 74,
            nominal_slots: 100,
            header_flag: Some(0),
            max_capacity: 128,
            end_tags: 1,
            trailer: false,
            end: [0x396e, 0],
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

    fn header(
        out: &mut Vec<u8>,
        outer: u32,
        inner: u32,
        record: u32,
        nominal: u32,
        capacity: u32,
        active: u32,
    ) {
        for word in [outer, inner, 1, record, nominal, 4, capacity, active, 0] {
            push_u32(out, word);
        }
        for level in 0..4 {
            push_u32(out, u32::from(level == 0));
            push_u32(out, 0);
            push_u32(out, 0);
        }
    }

    #[test]
    fn reads_fixed_framing_with_a_raw_trailer() {
        let mut bytes = vec![0u8; 0x3000];
        header(&mut bytes, 0x332a, 0x4a9d, 4, 10, 3, 2);
        bytes.push(0b101); // slots 0 and 2 active
        bytes.extend_from_slice(&[7; 4]);
        bytes.extend_from_slice(&[9; 4]);
        push_u32(&mut bytes, 0x4a3a);
        bytes.extend_from_slice(b"trailer-bytes");
        push_u32(&mut bytes, 0x4a3b);
        let spec = FixedSpec {
            outer: 0x332a,
            inner: 0x4a9d,
            record_bytes: 4,
            header_record_bytes: 4,
            nominal_slots: 10,
            header_flag: Some(0),
            max_capacity: 16,
            end_tags: 2,
            trailer: true,
            end: [0x4a3a, 0x4a3b],
        };
        let collection = fixed::read(&bytes, spec).expect("fixed collection");
        assert_eq!(collection.records.len(), 2);
        assert_eq!(collection.records[0].data, [7; 4]);
        assert_eq!(collection.records[1].data, [9; 4]);
        assert_eq!(collection.trailer, b"trailer-bytes");
        for end in 0..bytes.len() {
            assert!(
                fixed::read(&bytes[..end], spec).is_none(),
                "accepted prefix {end}"
            );
        }
    }

    #[test]
    fn reads_sparse_framing_with_variable_blobs_and_a_trailer() {
        let mut bytes = vec![0u8; 0x3000];
        for word in [0x332b, 0x4a39, 0, 6, 10, 20, 3, 2, 0] {
            push_u32(&mut bytes, word);
        }
        for level in 0..4 {
            push_u32(&mut bytes, u32::from(level == 0));
            push_u32(&mut bytes, 0);
            push_u32(&mut bytes, 0);
        }
        bytes.push(0b100); // slots 0 and 1 active
        for (prefix, data) in [(1u8, [2u8, 3].as_slice()), (4, [5, 6, 7].as_slice())] {
            bytes.extend_from_slice(&[prefix; 6]);
            bytes.extend_from_slice(&(data.len() as u16).to_le_bytes());
            bytes.extend_from_slice(data);
        }
        push_u32(&mut bytes, 0x38a6);
        bytes.extend_from_slice(b"car-trailer");
        push_u32(&mut bytes, 0x38a7);
        let spec = SparseSpec {
            outer: 0x332b,
            inner: 0x4a39,
            end: 0x38a6,
            preamble: None,
            blob_bytes: None,
            trailer_end: Some(0x38a7),
            header2: Some(10),
            header3: Some(20),
        };
        let collection = sparse::read(&bytes, spec).expect("sparse collection");
        assert_eq!(collection.records.len(), 2);
        assert_eq!(collection.records[0].data, [2, 3]);
        assert_eq!(collection.records[1].data, [5, 6, 7]);
        assert_eq!(collection.trailer, b"car-trailer");
        for end in 0..bytes.len() {
            assert!(
                sparse::read(&bytes[..end], spec).is_none(),
                "accepted prefix {end}"
            );
        }
    }

    #[test]
    fn reads_single_tag_track_line_like_collection() {
        let mut bytes = vec![0u8; 0x3000];
        header(&mut bytes, 0x32d3, 0x396d, 74, 100, 100, 1);
        bytes.resize(bytes.len() + 13, 0); // capacity 100 mask
        bytes.extend_from_slice(&[0x2a; 36]);
        push_u32(&mut bytes, 0x396e);
        let collection = track_lines(&bytes).expect("track-line collection");
        assert_eq!(collection.records.len(), 1);
        assert_eq!(collection.records[0].data.len(), 36);
        assert!(collection.trailer.is_empty());
    }

    #[test]
    fn reads_all_track_sections_from_the_populated_fixture() {
        let Ok(path) = std::env::var("RT3_TRACK_SAVE") else {
            eprintln!("RT3_TRACK_SAVE unset; skipping populated track fixture check");
            return;
        };
        let path = std::path::PathBuf::from(path);
        let path = if path.is_absolute() {
            path
        } else {
            let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
            let candidate = repo.join(&path);
            if candidate.exists() {
                candidate
            } else {
                path
            }
        };
        let bytes = fs::read(&path).expect("track fixture");
        let stations = stations(&bytes).expect("station collection");
        let cars = cars(&bytes).expect("car collection");
        let trains = trains(&bytes).expect("train collection");
        let track = track(&bytes).expect("track collection");
        let track_lines = track_lines(&bytes).expect("track-line collection");
        assert_eq!(stations.records.len(), 3);
        assert_eq!(cars.records.len(), 4);
        assert_eq!(trains.records.len(), 1);
        assert_eq!(track.records.len(), 61);
        assert_eq!(track_lines.records.len(), 5);
        assert_eq!(stations.trailer.len(), 867);
        assert_eq!(cars.trailer.len(), 718);
        assert_eq!(trains.trailer.len(), 110);
        assert_eq!(track.trailer.len(), 339);
        assert!(track_lines.trailer.is_empty());
    }
}
