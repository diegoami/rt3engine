//! Coarse terrain heightfield extraction from `.gmp` maps (milestone 3b,
//! step 1 — see `docs/evidence/gmp-terrain-v1.md`).
//!
//! Proven so far (claim level **decode — one verified map**):
//!
//! ```text
//! on Crossing the Alps (dims header lattice 256×256), the coarse
//! heights are stored uncompressed as f32 in 1032-byte rows: 257
//! height cells + 1 trailing cell (semantics open), forming the
//! 257×257 corner grid over the 256×256 terrain lattice.
//! ```
//!
//! The dims header varies per map (13 of 40 shipped maps carry a
//! detectable `0x2ee2` header — see `read_dims`); on lattices wider
//! than 256 this module's fixed window geometry is not a rectilinear
//! subwindow of the parent grid, so scan hits there are candidates,
//! not verified decodes (the CLI and evidence artifact carry the
//! status). Water is `0.0` in some maps and `NaN` in others; both are
//! accepted. The scan validates candidates by value range, land
//! fraction and row-to-row smoothness, so all-`0x00`/all-`0xFF` filler
//! (which reads as `0.0`/NaN floats) is rejected.

/// Height rows in the coarse grid.
pub const ROWS: usize = 257;
/// Height columns in the coarse grid.
pub const COLS: usize = 257;
/// Bytes per stored row: 257 height cells plus one trailing cell.
pub const ROW_BYTES: usize = (COLS + 1) * 4;
/// Total stored size of the coarse grid.
pub const GRID_BYTES: usize = ROWS * ROW_BYTES;

/// Lowest accepted height (metres).
const MIN_HEIGHT: f32 = -1000.0;
/// Highest accepted height (metres).
const MAX_HEIGHT: f32 = 9000.0;
/// Smallest magnitude that counts as land. Below ~5 cm, sane-looking
/// nonzero floats are almost certainly denormals from integer data.
const MIN_LAND: f32 = 0.05;
/// Validation samples every `VALIDATION_STEP`-th row and column.
const VALIDATION_STEP: usize = 4;
const SR: usize = ROWS.div_ceil(VALIDATION_STEP);
const SC: usize = COLS.div_ceil(VALIDATION_STEP);

/// Terrain section dims header: nine u32s after tag `0x2EE2`, present
/// in a subset of shipped maps (the others frame terrain differently;
/// see the evidence doc). `tiles_x/y`, `w2/h2`, `lattice_w/h`,
/// `fine_w/h` are the observed groupings (e.g. Alps: 64,64,64,64,
/// 256,256,1024,1024,0 — coarse corner grid 257×257 over the 256
/// lattice, 1024×1024 fine grid).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerrainDims {
    pub header_offset: usize,
    pub values: [u32; 9],
}

/// The tag that opens the dims header.
const DIMS_TAG: u32 = 0x2EE2;

/// Searches the whole file for the dims header: tag `0x2EE2`, nine
/// small words, then another terrain-family tag. Returns the first
/// structurally valid hit.
pub fn read_dims(bytes: &[u8]) -> Option<TerrainDims> {
    let mut off = 0usize;
    while off + 44 <= bytes.len() {
        if u32::from_le_bytes(bytes[off..off + 4].try_into().ok()?) == DIMS_TAG {
            let mut values = [0u32; 9];
            let mut ok = true;
            for (i, v) in values.iter_mut().enumerate() {
                *v = u32::from_le_bytes(bytes[off + 4 + 4 * i..off + 8 + 4 * i].try_into().ok()?);
                if *v > 8192 {
                    ok = false;
                    break;
                }
            }
            let next = u32::from_le_bytes(bytes[off + 40..off + 44].try_into().ok()?);
            let distinct = {
                let mut s = values.to_vec();
                s.sort_unstable();
                s.dedup();
                s.len()
            };
            if ok && distinct >= 3 && (0x2EE0..=0x2F50).contains(&next) {
                return Some(TerrainDims {
                    header_offset: off,
                    values,
                });
            }
        }
        off += 4;
    }
    None
}

/// An extracted coarse height grid.
#[derive(Debug, Clone, PartialEq)]
pub struct HeightGrid {
    /// File offset of the grid's first row.
    pub offset: usize,
    /// `ROWS × COLS` heights, row-major; NaN marks water sentinels.
    pub heights: Vec<f32>,
    /// Number of NaN cells.
    pub nan_cells: usize,
    /// Number of exactly-zero cells.
    pub zero_cells: usize,
}

impl HeightGrid {
    /// Height at `(row, col)`.
    pub fn get(&self, r: usize, c: usize) -> f32 {
        self.heights[r * COLS + c]
    }

    /// Lowest and highest non-NaN height, if any cell is populated.
    pub fn height_range(&self) -> Option<(f32, f32)> {
        self.heights
            .iter()
            .filter(|v| !v.is_nan())
            .copied()
            .fold(None::<(f32, f32)>, |acc, v| match acc {
                None => Some((v, v)),
                Some((lo, hi)) => Some((lo.min(v), hi.max(v))),
            })
    }
}

fn cell(bytes: &[u8], off: usize) -> Option<f32> {
    let b = bytes.get(off..off + 4)?;
    Some(f32::from_le_bytes(b.try_into().unwrap()))
}

/// Cheap pre-filter: 16 spread cells must all be `NaN`, zero or in range.
fn prefilter(bytes: &[u8], off: usize) -> bool {
    for i in 0..16 {
        let r = (i / 4) * 64;
        let c = (i % 4) * 64;
        match cell(bytes, off + (r * (COLS + 1) + c) * 4) {
            Some(v) if v.is_nan() || v == 0.0 || (MIN_HEIGHT..=MAX_HEIGHT).contains(&v) => {}
            _ => return false,
        }
    }
    true
}

/// Full validation: samples a 65×65 sub-grid (step 4) and gates on land
/// fraction, NaN fraction, value range and smoothness.
fn validate(bytes: &[u8], off: usize) -> bool {
    let mut vals = [[0.0f32; SC]; SR];
    let mut nan = 0usize;
    let mut land = 0usize;
    let sampled_rows: Vec<usize> = (0..ROWS).step_by(VALIDATION_STEP).collect();
    for &r in &sampled_rows {
        let ri = r / VALIDATION_STEP;
        for (ci, c) in (0..COLS).step_by(VALIDATION_STEP).enumerate() {
            let Some(v) = cell(bytes, off + (r * (COLS + 1) + c) * 4) else {
                return false;
            };
            if v.is_nan() {
                nan += 1;
                vals[ri][ci] = f32::NAN;
            } else if !(MIN_HEIGHT..=MAX_HEIGHT).contains(&v) {
                return false;
            } else {
                if v.abs() >= MIN_LAND {
                    land += 1;
                }
                vals[ri][ci] = v;
            }
        }
        // Early smoothness after the first quarter: float-dense garbage
        // (coordinate tables and the like) rejects here without paying
        // for the full pass. The threshold uses the quarter's own
        // spread, so flat sea openings pass trivially.
        if sampled_rows.len() >= 4 && ri == sampled_rows.len() / 4 {
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for row in vals.iter().take(ri + 1) {
                for v in row {
                    if !v.is_nan() {
                        lo = lo.min(*v);
                        hi = hi.max(*v);
                    }
                }
            }
            if lo <= hi && !band_smooth(&vals, 0..ri + 1, (hi - lo).max(1.0) * 0.08) {
                return false;
            }
        }
    }
    let total = SR * SC;
    // Filler blocks (all 0x00/0xFF) have no land at all; sea maps still
    // keep at least ~5% land among sampled cells.
    if land * 20 < total || nan * 5 > total * 4 {
        return false;
    }
    let mut lo = f32::MAX;
    let mut hi = f32::MIN;
    for row in &vals {
        for v in row {
            if !v.is_nan() {
                lo = lo.min(*v);
                hi = hi.max(*v);
            }
        }
    }
    let threshold = (hi - lo).max(1.0) * 0.08;
    // Relief gate: a window whose sampled spread is under ~10 m is a
    // table of small values, not a 256 km terrain grid.
    if hi - lo < 10.0 {
        return false;
    }
    let mut diff = 0.0f32;
    let mut pairs = 0usize;
    for ri in 0..SR {
        for ci in 0..SC {
            let v = vals[ri][ci];
            if v.is_nan() {
                continue;
            }
            if ci + 1 < SC && !vals[ri][ci + 1].is_nan() {
                diff += (v - vals[ri][ci + 1]).abs();
                pairs += 1;
            }
            if ri + 1 < SR && !vals[ri + 1][ci].is_nan() {
                diff += (v - vals[ri + 1][ci]).abs();
                pairs += 1;
            }
        }
    }
    pairs > 0 && diff / pairs as f32 <= threshold
}

/// Mean adjacent-|Δ| over a row band, for the early smoothness check.
fn band_smooth(vals: &[[f32; SC]; SR], rows: std::ops::Range<usize>, threshold: f32) -> bool {
    let mut diff = 0.0f32;
    let mut pairs = 0usize;
    for ri in rows {
        for ci in 0..vals[ri].len() {
            let v = vals[ri][ci];
            if v.is_nan() || ci + 1 >= vals[ri].len() {
                continue;
            }
            let w = vals[ri][ci + 1];
            if !w.is_nan() {
                diff += (v - w).abs();
                pairs += 1;
            }
        }
    }
    pairs > 0 && diff / pairs as f32 <= threshold
}

/// Extracts the full grid stored at `off` (already validated). Every
/// cell must be NaN or inside the height range — sampled validation can
/// otherwise pass windows with garbage between the samples.
fn extract(bytes: &[u8], off: usize) -> Option<HeightGrid> {
    if off + GRID_BYTES > bytes.len() {
        return None;
    }
    let mut heights = Vec::with_capacity(ROWS * COLS);
    let mut nan_cells = 0;
    let mut zero_cells = 0;
    for r in 0..ROWS {
        for c in 0..COLS {
            let v = cell(bytes, off + (r * (COLS + 1) + c) * 4)?;
            if v.is_nan() {
                nan_cells += 1;
            } else if v == 0.0 {
                zero_cells += 1;
            } else if !(MIN_HEIGHT..=MAX_HEIGHT).contains(&v) {
                return None;
            }
            heights.push(v);
        }
    }
    Some(HeightGrid {
        offset: off,
        heights,
        nan_cells,
        zero_cells,
    })
}

/// Scans for validated coarse grids, 4-byte aligned, from `from`.
///
/// The probe only ever rejects: its sample positions are a subset of
/// the validator's (both start at the window origin; the probe steps
/// 32, the validator 4), so a probe range violation guarantees
/// validation failure. No accept-side shortcut is taken — a window the
/// validator would accept is always validated (see the regression test
/// for a land layout the probe's lattice cannot see).
pub fn scan_all(bytes: &[u8], from: usize) -> Vec<HeightGrid> {
    let mut out = Vec::new();
    let mut off = from & !3;
    while off + GRID_BYTES <= bytes.len() {
        if !probe_rejects(bytes, off) && prefilter(bytes, off) && validate(bytes, off) {
            if let Some(g) = extract(bytes, refine_start(bytes, off)) {
                let next = g.offset + GRID_BYTES;
                out.push(g);
                off = next;
                continue;
            }
        }
        off += 4;
    }
    out
}

/// Cheap rejection from 64 cells spread over the whole grid (every
/// 32nd row and column). Sound as a rejector only: every probe
/// position is also a validator position, so a range violation here
/// guarantees `validate` fails. The probe deliberately draws no
/// accept-side conclusion.
fn probe_rejects(bytes: &[u8], off: usize) -> bool {
    for i in 0..64 {
        let r = (i / 8) * 32;
        let c = (i % 8) * 32;
        let Some(v) = cell(bytes, off + (r * (COLS + 1) + c) * 4) else {
            return true;
        };
        if v.is_nan() {
            continue;
        }
        if !(MIN_HEIGHT..=MAX_HEIGHT).contains(&v) {
            return true;
        }
    }
    false
}

/// After a hit, walks back one row at a time while the window still
/// validates, so a grid entered mid-way (after water-row skipping) is
/// reported from its first row. Zero padding that blends into leading
/// water rows is indistinguishable from them; the walk stops at the
/// first non-conforming window.
fn refine_start(bytes: &[u8], hit: usize) -> usize {
    let mut off = hit;
    while off >= ROW_BYTES && validate(bytes, off - ROW_BYTES) {
        off -= ROW_BYTES;
    }
    off
}

/// Scans for the first validated coarse grid from `from`, stopping at
/// the earliest hit.
pub fn scan(bytes: &[u8], from: usize) -> Option<HeightGrid> {
    let mut off = from & !3;
    while off + GRID_BYTES <= bytes.len() {
        if !probe_rejects(bytes, off) && prefilter(bytes, off) && validate(bytes, off) {
            if let Some(g) = extract(bytes, refine_start(bytes, off)) {
                return Some(g);
            }
        }
        off += 4;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Builds a synthetic terrain image: prelude-sized filler, then a
    /// smooth 257×257 grid with a hill, a sea of NaN and a flat plain.
    fn fixture() -> Vec<u8> {
        // 0x7F filler reads as 3.4e38 floats — far outside the height
        // range — so the scanner cannot validate inside the padding.
        let mut d = vec![0x7Fu8; 0x4000 + GRID_BYTES + 16];
        let base = 0x4000;
        for r in 0..ROWS {
            for c in 0..COLS + 1 {
                let v = if r < 32 {
                    f32::NAN // open sea
                } else {
                    // gentle rolling hills + plain, all in range and
                    // smooth at the validation sampling step
                    ((r as f32 - 32.0) * 0.3 + (c as f32 * 0.04).sin() * 20.0).max(0.0)
                };
                let o = base + (r * (COLS + 1) + c) * 4;
                d[o..o + 4].copy_from_slice(&v.to_le_bytes());
            }
        }
        d
    }

    #[test]
    fn finds_synthetic_grid() {
        let d = fixture();
        let g = scan(&d, 0x3000).expect("grid found");
        assert_eq!(g.offset, 0x4000);
        assert_eq!(g.heights.len(), ROWS * COLS);
        assert!(g.nan_cells > 0);
        let (lo, hi) = g.height_range().unwrap();
        assert!(lo >= 0.0 && hi <= 1000.0, "range {lo}..{hi}");
    }

    #[test]
    fn rejects_zero_filler() {
        let d = vec![0u8; 0x4000 + GRID_BYTES + 16];
        assert!(scan(&d, 0x3000).is_none());
    }

    #[test]
    fn reads_dims_header() {
        let mut d = vec![0u8; 0x2000];
        // A structurally valid header: tag, nine small words with
        // variety, then a terrain-family tag.
        d[0x1000..0x1004].copy_from_slice(&0x2EE2u32.to_le_bytes());
        for (i, v) in [64u32, 64, 64, 64, 256, 256, 1024, 1024, 0]
            .into_iter()
            .enumerate()
        {
            d[0x1004 + 4 * i..0x1008 + 4 * i].copy_from_slice(&v.to_le_bytes());
        }
        d[0x1028..0x102C].copy_from_slice(&0x2EE3u32.to_le_bytes());
        let dims = read_dims(&d).expect("dims header found");
        assert_eq!(dims.header_offset, 0x1000);
        assert_eq!(dims.values, [64, 64, 64, 64, 256, 256, 1024, 1024, 0]);
    }

    #[test]
    fn ignores_constant_dims() {
        let mut d = vec![0u8; 0x1000];
        d[0x100..0x104].copy_from_slice(&0x2EE2u32.to_le_bytes());
        // all-identical words: not a real header
        for i in 0..9 {
            d[0x104 + 4 * i..0x108 + 4 * i].copy_from_slice(&2u32.to_le_bytes());
        }
        d[0x128..0x12C].copy_from_slice(&0x2EE3u32.to_le_bytes());
        assert!(read_dims(&d).is_none());
    }

    /// Regression (review round 1, finding 1): a window whose land the
    /// probe's 32-cell lattice cannot see (blocks offset between probe
    /// rows/columns) still passes full validation — the probe must never
    /// be an accept-side shortcut.
    #[test]
    fn finds_probe_invisible_land() {
        let mut d = vec![0u8; 0x4000 + GRID_BYTES + 16];
        let base = 0x4000;
        for r in 0..ROWS {
            for c in 0..COLS + 1 {
                let land = r % 32 >= 16 && r % 32 < 24 && c % 32 >= 16 && c % 32 < 24;
                let v = if land { 30.0f32 } else { 0.0 };
                let o = base + (r * (COLS + 1) + c) * 4;
                d[o..o + 4].copy_from_slice(&v.to_le_bytes());
            }
        }
        let g = scan(&d, 0x3000).expect("probe-invisible window must still be found");
        // The all-zero padding blends into the grid's water rows, so the
        // earliest validating window may start before the fixture's grid;
        // the regression point is that a window is found at all and it
        // carries the probe-invisible land.
        let (lo, hi) = g.height_range().unwrap();
        assert_eq!((lo, hi), (0.0, 30.0));
        assert!(g.offset <= 0x4000);
    }

    #[test]
    fn rejects_pure_ff_filler() {
        // 0xFF bytes read as NaN everywhere: sane for the range gates,
        // but the land gate rejects it.
        let d = vec![0xFFu8; 0x4000 + GRID_BYTES + 16];
        assert!(scan(&d, 0x3000).is_none());
    }

    #[test]
    fn rejects_mixed_ff_zero_filler() {
        // NaN/zero mix with no land: rejects on the land gate.
        let mut d = vec![0xFFu8; 0x4000 + GRID_BYTES + 16];
        for (i, b) in d.iter_mut().enumerate() {
            if i % 7 == 0 {
                *b = 0;
            }
        }
        assert!(scan(&d, 0x3000).is_none());
    }

    /// Install-backed smoke check: the reference calibration map scans
    /// to a sane grid (full per-map coverage lives in the
    /// `terrain-scan-v1.json` evidence artifact via `rt3ctl map terrain
    /// --all --json`). Self-skips without `RT3_HOME`.
    #[test]
    fn scans_reference_map() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed terrain check");
            return;
        };
        let path = std::path::Path::new(&home).join("maps/Crossing the Alps.gmp");
        let Ok(bytes) = fs::read(&path) else {
            eprintln!("{} missing; skipping", path.display());
            return;
        };
        let g = scan(&bytes, 0x3000).unwrap_or_else(|| panic!("no grid in {}", path.display()));
        let (lo, hi) = g.height_range().unwrap();
        // Verified decode: the map's own dims header must say the
        // lattice is 256×256 (the only lattice the fixed window
        // geometry is known to read rectilinearly).
        let dims = read_dims(&bytes).expect("dims header present");
        assert_eq!((&dims.values[4], &dims.values[5]), (&256, &256));
        assert!(g.offset < 0x20_0000, "offset {:#x}", g.offset);
        assert!(lo >= -1000.0 && hi <= 4000.0, "range {lo}..{hi}");
        assert!(hi > 1500.0, "expected alpine relief, max {hi}");
    }
}
