//! Per-cell and per-tile terrain sections `0x2f43`/`0x2f44`.
//!
//! The layout is derived from the original reader at `RT3.exe 0x0042ae50`
//! and validated against all 43 shipped maps. Field semantics remain open.

use crate::terrain::HeightGrid;

pub const PRESENCE_BYTES: usize = 20;
pub const ARRAY_LEN: usize = 53;
const MAX_CELL_ARRAY_CELLS: usize = 1_048_576;
const MAX_TILE_RECORDS: usize = 65_536;
const MAX_DYNAMIC_ENTRIES: usize = 1_000_000;
const MAX_PARSE_RESERVE: usize = 40 * 1024 * 1024;

/// One `0x2f44` tile record. Optional values absent from the presence
/// mask are zero, matching the original reader's initialized record.
#[derive(Debug, Clone, PartialEq)]
pub struct TileRecord {
    pub presence: [u8; PRESENCE_BYTES],
    pub first_u16: [u16; ARRAY_LEN],
    pub second_u16: [u16; ARRAY_LEN],
    pub values_f32: [f32; ARRAY_LEN],
    /// Twelve fixed file-order bytes between the arrays and entry count.
    /// Their individual field semantics are still open.
    pub fixed: [u8; 12],
    /// Raw six-byte entries following the signed 16-bit count.
    pub entries: Vec<[u8; 6]>,
}

/// Parsed `0x2f43` arrays and `0x2f44` records.
#[derive(Debug, Clone, PartialEq)]
pub struct TileSections {
    pub section_offset: usize,
    pub cell_data_offset: usize,
    /// Four byte arrays, each with `(rows+1)*(cols+1)` cells.
    pub cell_arrays: [Vec<u8>; 4],
    pub records_offset: usize,
    pub records: Vec<TileRecord>,
    /// File offset of the terminating `0x2ee5` tag.
    pub end_offset: usize,
}

impl TileSections {
    pub fn dynamic_entries(&self) -> usize {
        self.records.iter().map(|r| r.entries.len()).sum()
    }
}

struct Layout {
    cells: usize,
    tile_count: usize,
}

fn layout(grid: &HeightGrid) -> Option<Layout> {
    if grid.cols < 5
        || grid.rows < 5
        || !(grid.cols - 1).is_multiple_of(4)
        || !(grid.rows - 1).is_multiple_of(4)
    {
        return None;
    }
    let cells = grid
        .rows
        .checked_add(1)?
        .checked_mul(grid.cols.checked_add(1)?)?;
    let tile_count = ((grid.cols - 1) / 4).checked_mul((grid.rows - 1) / 4)?;
    if cells > MAX_CELL_ARRAY_CELLS || tile_count > MAX_TILE_RECORDS {
        return None;
    }
    let reserve = cells
        .checked_mul(4)?
        .checked_add(tile_count.checked_mul(std::mem::size_of::<TileRecord>())?)?;
    if reserve > MAX_PARSE_RESERVE {
        return None;
    }
    Some(Layout { cells, tile_count })
}

fn u32_at(bytes: &[u8], off: usize) -> Option<u32> {
    let end = off.checked_add(4)?;
    Some(u32::from_le_bytes(bytes.get(off..end)?.try_into().ok()?))
}

fn take<'a>(bytes: &'a [u8], off: &mut usize, len: usize) -> Option<&'a [u8]> {
    let end = off.checked_add(len)?;
    let out = bytes.get(*off..end)?;
    *off = end;
    Some(out)
}

fn present(mask: &[u8; PRESENCE_BYTES], bit: usize) -> bool {
    mask[bit / 8] & (1 << (bit % 8)) != 0
}

fn parse_candidate(bytes: &[u8], grid: &HeightGrid, section: usize) -> Option<TileSections> {
    let Layout { cells, tile_count } = layout(grid)?;
    let cell_data_offset = section.checked_add(8)?;
    let arrays_bytes = cells.checked_mul(4)?;
    let end_arrays = cell_data_offset.checked_add(arrays_bytes)?;
    let nested_tag = section.checked_add(4)?;
    let records_tag = end_arrays.checked_add(4)?;
    if u32_at(bytes, section)? != 0x2f43
        || u32_at(bytes, nested_tag)? != 0x9471
        || u32_at(bytes, end_arrays)? != 0x9472
        || u32_at(bytes, records_tag)? != 0x2f44
    {
        return None;
    }
    let mut cell_arrays: [Vec<u8>; 4] = Default::default();
    for (i, array) in cell_arrays.iter_mut().enumerate() {
        let start = i.checked_mul(cells)?.checked_add(cell_data_offset)?;
        let end = start.checked_add(cells)?;
        array.try_reserve_exact(cells).ok()?;
        array.extend_from_slice(bytes.get(start..end)?);
    }
    let records_offset = end_arrays.checked_add(8)?;
    let mut off = records_offset;
    let mut records = Vec::new();
    records.try_reserve_exact(tile_count).ok()?;
    let mut dynamic_entries = 0usize;
    for _ in 0..tile_count {
        let presence: [u8; PRESENCE_BYTES] =
            take(bytes, &mut off, PRESENCE_BYTES)?.try_into().ok()?;
        let mut first_u16 = [0u16; ARRAY_LEN];
        let mut second_u16 = [0u16; ARRAY_LEN];
        let mut values_f32 = [0.0f32; ARRAY_LEN];
        for i in 0..ARRAY_LEN {
            if present(&presence, 3 * i) {
                first_u16[i] = u16::from_le_bytes(take(bytes, &mut off, 2)?.try_into().ok()?);
            }
            if present(&presence, 3 * i + 1) {
                second_u16[i] = u16::from_le_bytes(take(bytes, &mut off, 2)?.try_into().ok()?);
            }
            if present(&presence, 3 * i + 2) {
                values_f32[i] = f32::from_le_bytes(take(bytes, &mut off, 4)?.try_into().ok()?);
            }
        }
        let fixed = take(bytes, &mut off, 12)?.try_into().ok()?;
        let count = i16::from_le_bytes(take(bytes, &mut off, 2)?.try_into().ok()?);
        if count < 0 {
            return None;
        }
        dynamic_entries = dynamic_entries.checked_add(count as usize)?;
        if dynamic_entries > MAX_DYNAMIC_ENTRIES {
            return None;
        }
        let mut entries = Vec::new();
        entries.try_reserve_exact(count as usize).ok()?;
        for _ in 0..count {
            entries.push(take(bytes, &mut off, 6)?.try_into().ok()?);
        }
        records.push(TileRecord {
            presence,
            first_u16,
            second_u16,
            values_f32,
            fixed,
            entries,
        });
    }
    if u32_at(bytes, off)? != 0x2ee5 {
        return None;
    }
    Some(TileSections {
        section_offset: section,
        cell_data_offset,
        cell_arrays,
        records_offset,
        records,
        end_offset: off,
    })
}

/// Finds a structurally complete tile section. Several maps contain
/// incidental `0x2f43` values before the real one, so candidates are
/// accepted only after their full array and record walk reaches `0x2ee5`.
pub fn read(bytes: &[u8], grid: &HeightGrid) -> Option<TileSections> {
    let mut off = grid.offset.saturating_sub(32);
    while off <= bytes.len().saturating_sub(4) {
        if u32_at(bytes, off) == Some(0x2f43) {
            if let Some(sections) = parse_candidate(bytes, grid, off) {
                return Some(sections);
            }
        }
        off += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn push_u32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn reads_optional_values_and_dynamic_entries() {
        let grid = HeightGrid {
            offset: 0,
            rows: 5,
            cols: 5,
            heights: vec![0.0; 25],
            nan_cells: 0,
            zero_cells: 25,
        };
        let mut bytes = vec![0u8; 0x100];
        push_u32(&mut bytes, 0x2f43);
        push_u32(&mut bytes, 0x9471);
        bytes.extend(0..36 * 4);
        push_u32(&mut bytes, 0x9472);
        push_u32(&mut bytes, 0x2f44);
        let mut mask = [0u8; PRESENCE_BYTES];
        mask[0] = 0x8f; // bits 0, 1, 2, 3 and 7
        mask[1] = 0x01; // bit 8
        bytes.extend_from_slice(&mask);
        bytes.extend_from_slice(&11u16.to_le_bytes()); // index 0, first
        bytes.extend_from_slice(&22u16.to_le_bytes()); // index 0, second
        bytes.extend_from_slice(&1.5f32.to_le_bytes()); // index 0, value
        bytes.extend_from_slice(&33u16.to_le_bytes()); // index 1, first only
        bytes.extend_from_slice(&44u16.to_le_bytes()); // index 2, second only
        bytes.extend_from_slice(&2.5f32.to_le_bytes()); // index 2, value
        bytes.extend_from_slice(b"fixed-12byte");
        bytes.extend_from_slice(&1i16.to_le_bytes());
        bytes.extend_from_slice(&[1, 2, 3, 4, 5, 6]);
        push_u32(&mut bytes, 0x2ee5);
        let sections = read(&bytes, &grid).expect("tile sections");
        assert_eq!(sections.cell_arrays[0], (0..36).collect::<Vec<_>>());
        assert_eq!(sections.cell_arrays[3], (108..144).collect::<Vec<_>>());
        assert_eq!(sections.records.len(), 1);
        let record = &sections.records[0];
        assert_eq!(record.first_u16[0], 11);
        assert_eq!(record.second_u16[0], 22);
        assert_eq!(record.values_f32[0], 1.5);
        assert_eq!(record.first_u16[1], 33);
        assert_eq!(record.second_u16[1], 0);
        assert_eq!(record.values_f32[1], 0.0);
        assert_eq!(record.first_u16[2], 0);
        assert_eq!(record.second_u16[2], 44);
        assert_eq!(record.values_f32[2], 2.5);
        assert_eq!(&record.fixed, b"fixed-12byte");
        assert_eq!(record.entries, vec![[1, 2, 3, 4, 5, 6]]);
        assert_eq!(u32_at(&bytes, sections.end_offset), Some(0x2ee5));
        for end in 0..bytes.len() {
            let _ = read(&bytes[..end], &grid);
        }
        bytes[sections.end_offset..sections.end_offset + 4].copy_from_slice(&0u32.to_le_bytes());
        assert!(read(&bytes, &grid).is_none());
    }

    #[test]
    fn rejects_excessive_or_overflowing_dimensions() {
        let oversized = HeightGrid {
            offset: 0,
            rows: 4001,
            cols: 4001,
            heights: Vec::new(),
            nan_cells: 0,
            zero_cells: 0,
        };
        assert!(layout(&oversized).is_none());
        if usize::BITS >= 64 {
            let overflowing = HeightGrid {
                rows: 4_294_967_297,
                cols: 4_294_967_297,
                ..oversized
            };
            assert!(layout(&overflowing).is_none());
            assert!(read(b"\x43\x2f\x00\x00\x71\x94\x00\x00", &overflowing).is_none());
        }
    }

    #[test]
    fn reads_all_shipped_tile_sections() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed tile check");
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
        let mut total_records = 0;
        let mut total_entries = 0;
        let mut max_entries = 0;
        for path in paths {
            let bytes = fs::read(&path).unwrap();
            let grid = crate::terrain::read_framed(&bytes).expect("framed grid");
            let sections = read(&bytes, &grid)
                .unwrap_or_else(|| panic!("no tile sections in {}", path.display()));
            let cells = (grid.rows + 1) * (grid.cols + 1);
            assert!(sections.cell_arrays.iter().all(|a| a.len() == cells));
            assert_eq!(
                sections.records.len(),
                ((grid.cols - 1) / 4) * ((grid.rows - 1) / 4)
            );
            assert!(sections
                .records
                .iter()
                .all(|r| r.presence.iter().any(|v| *v != 0)));
            total_records += sections.records.len();
            total_entries += sections.dynamic_entries();
            max_entries = max_entries.max(
                sections
                    .records
                    .iter()
                    .map(|r| r.entries.len())
                    .max()
                    .unwrap_or(0),
            );
        }
        assert_eq!(total_records, 645_120);
        assert_eq!(total_entries, 4_458);
        assert_eq!(max_entries, 4);
    }
}
