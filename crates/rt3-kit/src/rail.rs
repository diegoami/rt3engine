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

/// Byte offset of the piece origin (three f32) in a track record blob.
pub const TRACK_ORIGIN_OFFSET: usize = 12;
/// An origin step above this many units starts a new run.
pub const RUN_BREAK_UNITS: f32 = 100.0;

#[derive(Debug, Clone, PartialEq)]
pub struct TrackPiece {
    pub origin: [f32; 3],
}

/// A declared connectivity graph over decoded track piece origins.
///
/// Runs split where consecutive origins are more than [`RUN_BREAK_UNITS`]
/// apart; run ends within one median piece step are joined. The rule is a
/// declared model of connectivity, not a recovered game semantic.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackGraph {
    pub pieces: Vec<TrackPiece>,
    pub runs: Vec<std::ops::Range<usize>>,
    pub joins: Vec<(usize, usize)>,
    pub piece_length: f32,
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

impl TrackGraph {
    /// Builds the graph from a parsed track collection.
    pub fn from_collection(track: &SparseBlobCollection) -> Option<Self> {
        let mut pieces = Vec::new();
        pieces.try_reserve_exact(track.records.len()).ok()?;
        for record in &track.records {
            let raw = record
                .data
                .get(TRACK_ORIGIN_OFFSET..TRACK_ORIGIN_OFFSET + 12)?;
            let origin = [
                f32::from_le_bytes(raw[0..4].try_into().ok()?),
                f32::from_le_bytes(raw[4..8].try_into().ok()?),
                f32::from_le_bytes(raw[8..12].try_into().ok()?),
            ];
            if !origin.iter().all(|value| value.is_finite()) {
                return None;
            }
            pieces.push(TrackPiece { origin });
        }
        Some(Self::from_pieces(pieces))
    }

    /// Splits runs and joins run ends under the declared rule.
    pub fn from_pieces(pieces: Vec<TrackPiece>) -> Self {
        let mut runs: Vec<std::ops::Range<usize>> = Vec::new();
        if !pieces.is_empty() {
            let mut start = 0;
            for index in 1..pieces.len() {
                if distance(pieces[index - 1].origin, pieces[index].origin) > RUN_BREAK_UNITS {
                    runs.push(start..index);
                    start = index;
                }
            }
            runs.push(start..pieces.len());
        }

        let mut steps: Vec<f32> = Vec::new();
        for run in &runs {
            for index in run.start + 1..run.end {
                steps.push(distance(pieces[index - 1].origin, pieces[index].origin));
            }
        }
        steps.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let piece_length = if steps.is_empty() {
            0.0
        } else {
            steps[steps.len() / 2]
        };

        let mut joins = Vec::new();
        if piece_length > 0.0 {
            for a in 0..runs.len() {
                for b in a + 1..runs.len() {
                    for (end, start) in [
                        (runs[a].end - 1, runs[b].start),
                        (runs[b].end - 1, runs[a].start),
                    ] {
                        if distance(pieces[end].origin, pieces[start].origin) <= piece_length {
                            joins.push((end, start));
                        }
                    }
                }
            }
        }

        Self {
            pieces,
            runs,
            joins,
            piece_length,
        }
    }

    /// Edges: consecutive pieces within a run, plus the declared joins.
    pub fn edges(&self) -> Vec<(usize, usize)> {
        let mut edges = Vec::new();
        for run in &self.runs {
            for index in run.start + 1..run.end {
                edges.push((index - 1, index));
            }
        }
        edges.extend(self.joins.iter().copied());
        edges
    }
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

    #[test]
    fn splits_runs_and_joins_run_ends() {
        let pieces = vec![
            TrackPiece {
                origin: [0.0, 0.0, 0.0],
            },
            TrackPiece {
                origin: [33.0, 0.0, 0.0],
            },
            TrackPiece {
                origin: [1000.0, 0.0, 0.0],
            },
            TrackPiece {
                origin: [33.0, 33.0, 0.0],
            },
            TrackPiece {
                origin: [66.0, 33.0, 0.0],
            },
        ];
        let graph = TrackGraph::from_pieces(pieces);
        assert_eq!(graph.runs, vec![0..2, 2..3, 3..5]);
        assert!((graph.piece_length - 33.0).abs() < 0.01);
        assert_eq!(graph.joins, vec![(1, 3)]);
        assert_eq!(graph.edges().len(), 3);
    }

    #[test]
    fn builds_the_declared_graph_from_the_populated_fixture() {
        let Ok(path) = std::env::var("RT3_TRACK_SAVE") else {
            eprintln!("RT3_TRACK_SAVE unset; skipping populated track graph check");
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
        let track = track(&bytes).expect("track collection");
        let graph = TrackGraph::from_collection(&track).expect("track graph");
        assert_eq!(graph.pieces.len(), 61);
        assert_eq!(graph.runs, vec![0..5, 5..10, 10..61]);
        assert!((30.0..36.0).contains(&graph.piece_length));
        assert_eq!(graph.joins.len(), 2);
        assert!(graph.joins.contains(&(4, 10)));
        assert!(graph.joins.contains(&(60, 5)));
        assert_eq!(graph.edges().len(), 60);
    }
}
