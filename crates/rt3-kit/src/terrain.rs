//! Terrain heightfield extraction from `.gmp` maps (milestone 3b — see
//! `docs/evidence/gmp-terrain-v1.md`).
//!
//! Proven so far (claim level **decode — dims-driven, render-corroborated
//! on two maps**):
//!
//! ```text
//! the coarse heights are stored uncompressed as f32 in
//! (lattice+2)-cell rows: (lattice+1) height cells + 1 trailing cell
//! (semantics open), forming the (lattice+1)² corner grid over the
//! terrain lattice. Verified on Crossing the Alps (lattice 256×256,
//! matches the oracle's in-game render) and Germany (512×512, renders
//! the expected relief); the lattice comes from each map's own
//! `0x2ee2` dims header when present.
//! ```
//!
//! Maps without a dims header get their lattice height from the record
//! extent word and their width from a measured row pitch (see
//! `measure_pitch`); hits at that geometry are *pitch-anchored*. Only
//! when no pitch can be measured does the fixed 256-lattice geometry
//! run as a plain candidate, and only when the record anchors the
//! height at 256. Water is `0.0` in some maps and `NaN` in others;
//! both are accepted. The scan validates windows by
//! value range, land fraction and row-to-row smoothness, so
//! all-`0x00`/all-`0xFF` filler (which reads as `0.0`/NaN floats) is
//! rejected.

/// Height rows of the default (256-lattice) coarse grid.
pub const ROWS: usize = 257;
/// Height columns of the default (256-lattice) coarse grid.
pub const COLS: usize = 257;
/// Default stored row: 257 height cells plus one trailing cell.
pub const ROW_BYTES: usize = (COLS + 1) * 4;
/// Default stored grid size.
pub const GRID_BYTES: usize = ROWS * ROW_BYTES;

/// Lowest accepted height (metres).
const MIN_HEIGHT: f32 = -1000.0;
/// Highest accepted height (metres).
const MAX_HEIGHT: f32 = 9000.0;
/// Smallest magnitude that counts as land. Below ~5 cm, sane-looking
/// nonzero floats are almost certainly denormals from integer data.
const MIN_LAND: f32 = 0.05;
/// Validation samples every 4th row and column.
const VALIDATION_STEP: usize = 4;

/// Smallest grid the sampler supports: the probe needs 8 distinct rows
/// and columns on the step-4 validation lattice (7·4+1 = 33 cells).
/// Geometries below this floor are rejected by the scan rather than
/// sampled out of bounds (review round 1, finding 2).
const MIN_GRID_CELLS: usize = 8 * VALIDATION_STEP + 1;

/// Stored-grid geometry for one lattice: `rows = h + 1` height rows of
/// `cols = w + 1` cells, stored in `row_cells = w + 2`-cell rows (the
/// extra cell trails each row; semantics open).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    pub rows: usize,
    pub cols: usize,
    pub row_cells: usize,
}

impl Geometry {
    /// Geometry for a `w × h` terrain lattice.
    pub fn for_lattice(w: usize, h: usize) -> Geometry {
        Geometry {
            rows: h + 1,
            cols: w + 1,
            row_cells: w + 2,
        }
    }

    /// The default geometry (256×256 lattice) used when a map carries
    /// no dims header.
    pub fn default_256() -> Geometry {
        Geometry::for_lattice(256, 256)
    }

    /// Whether the sampler's probe/validate lattices fit inside this
    /// geometry without reading outside the window.
    pub fn supported(&self) -> bool {
        self.rows >= MIN_GRID_CELLS && self.cols >= MIN_GRID_CELLS
    }

    /// Total stored size of the grid in bytes.
    pub fn grid_bytes(&self) -> usize {
        self.rows * self.row_cells * 4
    }

    /// File offset of height `(r, c)` relative to the grid start.
    fn cell_off(&self, r: usize, c: usize) -> usize {
        (r * self.row_cells + c) * 4
    }
}

/// Terrain section dims header: nine u32s after tag `0x2EE2`, present
/// in a subset of shipped maps (the others frame terrain differently;
/// see the evidence doc). The observed groupings are tiles, tiles,
/// lattice, lattice×4 (e.g. Alps: 64,64,64,64,256,256,1024,1024,0 —
/// coarse corner grid 257×257 over the 256 lattice, 1024×1024 fine
/// grid).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerrainDims {
    pub header_offset: usize,
    pub values: [u32; 9],
}

impl TerrainDims {
    /// The lattice (lattice_w, lattice_h) = values[4], values[5].
    pub fn lattice(&self) -> (usize, usize) {
        (self.values[4] as usize, self.values[5] as usize)
    }
}

/// The map record's first extent word (file+12) equals `lattice_h × 256`
/// on every header-bearing map checked (13 of 13; see the evidence
/// doc). This anchors the lattice **height** independently of the
/// dims header, which resolves the orientation of non-square lattices.
pub fn lattice_h_from_record(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 16 {
        return None;
    }
    let word = u32::from_le_bytes(bytes[12..16].try_into().ok()?);
    if word % 256 != 0 {
        return None;
    }
    let h = (word / 256) as usize;
    (128..=4096).contains(&h).then_some(h)
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
            if ok
                && distinct >= 3
                && (0x2EE0..=0x2F50).contains(&next)
                && values[4] > 0
                && values[5] > 0
            {
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
    /// Height rows in this grid.
    pub rows: usize,
    /// Height columns in this grid.
    pub cols: usize,
    /// `rows × cols` heights, row-major; NaN marks water sentinels.
    pub heights: Vec<f32>,
    /// Number of NaN cells.
    pub nan_cells: usize,
    /// Number of exactly-zero cells.
    pub zero_cells: usize,
}

impl HeightGrid {
    /// Height at `(row, col)`.
    pub fn get(&self, r: usize, c: usize) -> f32 {
        self.heights[r * self.cols + c]
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

/// Probe/validate sample strides, rounded down to multiples of the
/// validation step so every probe/prefilter position is also a
/// validator position (the rejection shortcuts rely on that subset).
fn step_of(dim: usize, divisor: usize) -> usize {
    let raw = dim / divisor;
    (raw / VALIDATION_STEP) * VALIDATION_STEP
}

/// Cheap rejection from 64 cells spread over the whole grid. Sound as
/// a rejector only: every probe position is also a validator position,
/// so a range violation here guarantees `validate` fails. The probe
/// deliberately draws no accept-side conclusion.
fn probe_rejects(bytes: &[u8], off: usize, geo: &Geometry) -> bool {
    let step_r = step_of(geo.rows, 8).max(VALIDATION_STEP);
    let step_c = step_of(geo.cols, 8).max(VALIDATION_STEP);
    for i in 0..64 {
        let r = (i / 8) * step_r;
        let c = (i % 8) * step_c;
        let Some(v) = cell(bytes, off + geo.cell_off(r, c)) else {
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

/// Cheap rejection from 16 spread cells (positions on the validator
/// lattice, as above).
fn prefilter(bytes: &[u8], off: usize, geo: &Geometry) -> bool {
    let step_r = step_of(geo.rows, 4).max(VALIDATION_STEP);
    let step_c = step_of(geo.cols, 4).max(VALIDATION_STEP);
    for i in 0..16 {
        let r = (i / 4) * step_r;
        let c = (i % 4) * step_c;
        let Some(v) = cell(bytes, off + geo.cell_off(r, c)) else {
            return false;
        };
        if v.is_nan() || (MIN_HEIGHT..=MAX_HEIGHT).contains(&v) {
            continue;
        }
        return false;
    }
    true
}

/// Full validation: samples every 4th row/column and gates on land
/// fraction, NaN fraction, value range, relief and smoothness (with an
/// early smoothness check after the first quarter of rows).
fn validate(bytes: &[u8], off: usize, geo: &Geometry) -> bool {
    let sr = geo.rows.div_ceil(VALIDATION_STEP);
    let sc = geo.cols.div_ceil(VALIDATION_STEP);
    let mut vals = vec![0.0f32; sr * sc];
    let mut nan = 0usize;
    let mut land = 0usize;
    let mut lo = f32::MAX;
    let mut hi = f32::MIN;
    let quarter = sr / 4;
    for ri in 0..sr {
        for ci in 0..sc {
            let (r, c) = (ri * VALIDATION_STEP, ci * VALIDATION_STEP);
            let Some(v) = cell(bytes, off + geo.cell_off(r, c)) else {
                return false;
            };
            if v.is_nan() {
                nan += 1;
                vals[ri * sc + ci] = f32::NAN;
                continue;
            }
            if !(MIN_HEIGHT..=MAX_HEIGHT).contains(&v) {
                return false;
            }
            if v.abs() >= MIN_LAND {
                land += 1;
            }
            lo = lo.min(v);
            hi = hi.max(v);
            vals[ri * sc + ci] = v;
        }
        // Early smoothness after the first quarter: float-dense garbage
        // (coordinate tables and the like) rejects here without paying
        // for the full pass. The threshold uses the quarter's own
        // spread, so flat sea openings pass trivially.
        if ri == quarter && lo <= hi {
            let thr = (hi - lo).max(1.0) * 0.08;
            if !band_smooth(&vals, sc, 0..ri + 1, thr) {
                return false;
            }
        }
    }
    let total = sr * sc;
    // Filler blocks (all 0x00/0xFF) have no land at all; sea maps still
    // keep at least ~5% land among sampled cells.
    if land * 20 < total || nan * 5 > total * 4 {
        return false;
    }
    // Relief gate: a window whose sampled spread is under ~10 m is a
    // table of small values, not a terrain grid.
    if hi - lo < 10.0 {
        return false;
    }
    let threshold = (hi - lo).max(1.0) * 0.08;
    let mut diff = 0.0f32;
    let mut pairs = 0usize;
    for ri in 0..sr {
        for ci in 0..sc {
            let v = vals[ri * sc + ci];
            if v.is_nan() {
                continue;
            }
            if ci + 1 < sc && !vals[ri * sc + ci + 1].is_nan() {
                diff += (v - vals[ri * sc + ci + 1]).abs();
                pairs += 1;
            }
            if ri + 1 < sr && !vals[(ri + 1) * sc + ci].is_nan() {
                diff += (v - vals[(ri + 1) * sc + ci]).abs();
                pairs += 1;
            }
        }
    }
    pairs > 0 && diff / pairs as f32 <= threshold
}

/// Mean adjacent-|Δ| over a row band of the sampled grid, for the
/// early smoothness check.
fn band_smooth(vals: &[f32], sc: usize, rows: std::ops::Range<usize>, threshold: f32) -> bool {
    let mut diff = 0.0f32;
    let mut pairs = 0usize;
    for ri in rows {
        for ci in 0..sc.saturating_sub(1) {
            let v = vals[ri * sc + ci];
            let w = vals[ri * sc + ci + 1];
            if v.is_nan() || w.is_nan() {
                continue;
            }
            diff += (v - w).abs();
            pairs += 1;
        }
    }
    pairs > 0 && diff / pairs as f32 <= threshold
}

/// Extracts the full grid stored at `off` (already validated). Every
/// cell must be NaN or inside the height range — sampled validation can
/// otherwise pass windows with garbage between the samples.
fn extract(bytes: &[u8], off: usize, geo: &Geometry) -> Option<HeightGrid> {
    if off + geo.grid_bytes() > bytes.len() {
        return None;
    }
    let mut heights = Vec::with_capacity(geo.rows * geo.cols);
    let mut nan_cells = 0;
    let mut zero_cells = 0;
    for r in 0..geo.rows {
        for c in 0..geo.cols {
            let v = cell(bytes, off + geo.cell_off(r, c))?;
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
        rows: geo.rows,
        cols: geo.cols,
        heights,
        nan_cells,
        zero_cells,
    })
}

/// After a hit, walks back one row at a time while the window still
/// validates, so a grid entered mid-way is reported from its first
/// row. Zero padding that blends into leading water rows is
/// indistinguishable from them; the walk stops at the first
/// non-conforming window.
fn refine_start(bytes: &[u8], hit: usize, geo: &Geometry) -> usize {
    let row = geo.row_cells * 4;
    let mut off = hit;
    while off >= row && validate(bytes, off - row, geo) {
        off -= row;
    }
    off
}

/// Candidate row pitches: `w + 2` cells for every lattice width
/// observed in shipped maps plus small reserves.
const PITCH_CANDIDATES: [usize; 12] = [130, 178, 258, 322, 386, 450, 514, 578, 642, 770, 834, 1026];

/// Largest region searched for the height-dense span.
const PITCH_SEARCH_LIMIT: usize = 0x30_0000;
/// Land-magnitude density a 4 KiB window needs to join the span.
const PITCH_SPAN_DENSITY: f32 = 0.12;

fn is_land_like(v: f32) -> bool {
    v.is_finite() && (MIN_LAND..=MAX_HEIGHT).contains(&v) || (-MAX_HEIGHT..=-MIN_LAND).contains(&v)
}

/// Finds the largest height-dense span (4 KiB windows at ≥
/// `PITCH_SPAN_DENSITY` land-like values) within the first
/// `PITCH_SEARCH_LIMIT` bytes. Returns `(lo, hi)`.
fn densest_span(bytes: &[u8]) -> Option<(usize, usize)> {
    let end = bytes.len().min(PITCH_SEARCH_LIMIT);
    let mut best: Option<(usize, usize)> = None;
    let mut cur: Option<(usize, usize)> = None;
    let mut off = 0x3000;
    while off + 4096 <= end {
        let mut land = 0usize;
        for i in 0..256 {
            let o = off + i * 16; // every 4th float
            if let Some(v) = cell(bytes, o) {
                if is_land_like(v) {
                    land += 1;
                }
            }
        }
        let hot = land as f32 / 256.0 >= PITCH_SPAN_DENSITY;
        if hot {
            cur = Some(match cur {
                Some((lo, hi)) => (lo, hi + 4096),
                None => (off, off + 4096),
            });
        } else if let Some(span) = cur.take() {
            let better = best
                .map(|(lo, hi)| span.1 - span.0 > hi - lo)
                .unwrap_or(true);
            if better {
                best = Some(span);
            }
        }
        off += 4096;
    }
    if let Some(span) = cur.take() {
        let better = best
            .map(|(lo, hi)| span.1 - span.0 > hi - lo)
            .unwrap_or(true);
        if better {
            best = Some(span);
        }
    }
    best
}

/// Measures the dominant row pitch inside the map's densest span by
/// lag autocorrelation over the observed pitch candidates. Returns
/// `(row_cells, winner_score, runner_up_score)` when at least two
/// candidates scored and the winner is clearly separated (score ≤ 60%
/// of the runner-up — the separation observed for real grids, e.g.
/// Tex-Mex 10 vs 29 and Chicago to New York 10 vs 179).
pub fn measure_pitch(bytes: &[u8]) -> Option<(usize, f32, f32)> {
    let (lo, hi) = densest_span(bytes)?;
    let n = (hi - lo) / 4;
    let mut scores: Vec<(f32, usize)> = Vec::new();
    for lag in PITCH_CANDIDATES {
        if lag + 4 >= n {
            continue;
        }
        let mut diff = 0.0f32;
        let mut pairs = 0usize;
        let mut i = 0usize;
        while i + lag + 4 < n {
            let a = cell(bytes, lo + i * 4);
            let b = cell(bytes, lo + (i + lag) * 4);
            if let (Some(a), Some(b)) = (a, b) {
                if is_land_like(a) && is_land_like(b) {
                    diff += (a - b).abs();
                    pairs += 1;
                }
            }
            i += 83; // prime stride over the span
        }
        if pairs > 40 {
            scores.push((diff / pairs as f32, lag));
        }
    }
    if scores.len() < 2 {
        return None;
    }
    scores.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
    let (win, lag) = scores[0];
    let (run, _) = scores[1];
    if run > 0.0 && win <= run * 0.6 {
        Some((lag, win, run))
    } else {
        None
    }
}

/// How a headerless map's hit geometry was established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderlessHit {
    /// Height from the record word, width from a clearly separated
    /// measured pitch, window validated at exactly that geometry.
    PitchAnchored,
    /// No pitch measurable and the record anchors the height at 256:
    /// the fixed 256-lattice guess.
    Candidate,
}

/// Scans a headerless map through the refusal ladder:
///
/// 1. record height (no usable record word — no result);
/// 2. measured pitch — the pitched geometry is scanned and, if it does
///    not validate, the ladder **stops**: no fallback to a guessed
///    geometry after the map has spoken;
/// 3. only with no measurable pitch and a record height of exactly 256
///    does the fixed-geometry candidate run.
pub fn scan_headerless(bytes: &[u8], from: usize) -> Option<(HeightGrid, HeaderlessHit)> {
    let h = lattice_h_from_record(bytes)?;
    if let Some((row_cells, _win, _run)) = measure_pitch(bytes) {
        let w = row_cells.checked_sub(2)?;
        return scan_geo(bytes, from, &Geometry::for_lattice(w, h))
            .map(|g| (g, HeaderlessHit::PitchAnchored));
    }
    if h == 256 {
        return scan(bytes, from).map(|g| (g, HeaderlessHit::Candidate));
    }
    None
}

/// Scans for validated grids of the given geometry, 4-byte aligned,
/// from `from`. The probe only ever rejects (see `probe_rejects`); no
/// accept-side shortcut is taken — a window the validator would accept
/// is always validated (see the regression test for a land layout the
/// probe's lattice cannot see).
pub fn scan_all_geo(bytes: &[u8], from: usize, geo: &Geometry) -> Vec<HeightGrid> {
    if !geo.supported() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut off = from & !3;
    while off + geo.grid_bytes() <= bytes.len() {
        if !probe_rejects(bytes, off, geo)
            && prefilter(bytes, off, geo)
            && validate(bytes, off, geo)
        {
            if let Some(g) = extract(bytes, refine_start(bytes, off, geo), geo) {
                let next = g.offset + geo.grid_bytes();
                out.push(g);
                off = next;
                continue;
            }
        }
        off += 4;
    }
    out
}

/// Scans for the first validated grid of the given geometry.
pub fn scan_geo(bytes: &[u8], from: usize, geo: &Geometry) -> Option<HeightGrid> {
    if !geo.supported() {
        return None;
    }
    let mut off = from & !3;
    while off + geo.grid_bytes() <= bytes.len() {
        if !probe_rejects(bytes, off, geo)
            && prefilter(bytes, off, geo)
            && validate(bytes, off, geo)
        {
            if let Some(g) = extract(bytes, refine_start(bytes, off, geo), geo) {
                return Some(g);
            }
        }
        off += 4;
    }
    None
}

/// Scans with the map's own dims-header geometry. The orientation is
/// anchored by the map record's extent word (`lattice_h_from_record`,
/// exact on every header-bearing map checked): the geometry whose
/// height matches the record is the one scanned. Without a usable
/// anchor both orientations are evaluated, and when both validate the
/// correct one cannot be distinguished — the result is reported as
/// ambiguous (`None`) rather than guessed.
pub fn scan_with_dims(bytes: &[u8], from: usize, dims: &TerrainDims) -> Option<HeightGrid> {
    let (w, h) = dims.lattice();
    if let Some(h_rec) = lattice_h_from_record(bytes) {
        if h == h_rec {
            return scan_geo(bytes, from, &Geometry::for_lattice(w, h));
        }
        if w == h_rec {
            return scan_geo(bytes, from, &Geometry::for_lattice(h, w));
        }
        // The record anchor matches neither orientation: the header and
        // the record disagree; no verified claim.
        return None;
    }
    let first = scan_geo(bytes, from, &Geometry::for_lattice(w, h));
    if w == h {
        return first;
    }
    let second = scan_geo(bytes, from, &Geometry::for_lattice(h, w));
    match (first, second) {
        (Some(g), None) | (None, Some(g)) => Some(g),
        (None, None) => None,
        (Some(_), Some(_)) => None, // ambiguous orientation
    }
}

/// Scans with the default (256-lattice) geometry: the scan used for
/// maps without a dims header, whose hits are candidates.
pub fn scan(bytes: &[u8], from: usize) -> Option<HeightGrid> {
    scan_geo(bytes, from, &Geometry::default_256())
}

/// Scans for every validated default-geometry grid (calibration aid).
pub fn scan_all(bytes: &[u8], from: usize) -> Vec<HeightGrid> {
    scan_all_geo(bytes, from, &Geometry::default_256())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Builds a synthetic terrain image of the given lattice: filler,
    /// then a (rows×cols) grid with a NaN sea, a plain and rolling
    /// hills.
    fn fixture_lattice(lattice_w: usize, lattice_h: usize) -> (Vec<u8>, usize) {
        let geo = Geometry::for_lattice(lattice_w, lattice_h);
        let base = 0x4000;
        // 0x7F filler reads as 3.4e38 floats — far outside the height
        // range — so the scanner cannot validate inside the padding.
        let mut d = vec![0x7Fu8; base + geo.grid_bytes() + 16];
        let sea_rows = geo.rows / 8;
        for r in 0..geo.rows {
            for c in 0..geo.row_cells {
                let v = if r < sea_rows {
                    f32::NAN // open sea
                } else {
                    // gentle rolling hills + plain, all in range and
                    // smooth at the validation sampling step
                    ((r - sea_rows) as f32 * 0.3 + (c as f32 * 0.04).sin() * 20.0).max(0.0)
                };
                let o = base + geo.cell_off(r, c);
                d[o..o + 4].copy_from_slice(&v.to_le_bytes());
            }
        }
        (d, base)
    }

    #[test]
    fn finds_synthetic_grid() {
        let (d, base) = fixture_lattice(256, 256);
        let g = scan(&d, 0x3000).expect("grid found");
        assert_eq!(g.offset, base);
        assert_eq!((g.rows, g.cols), (257, 257));
        assert_eq!(g.heights.len(), ROWS * COLS);
        assert!(g.nan_cells > 0);
        let (lo, hi) = g.height_range().unwrap();
        assert!(lo >= 0.0 && hi <= 1000.0, "range {lo}..{hi}");
    }

    #[test]
    fn finds_wider_lattice_with_dims_geometry() {
        // A 512-lattice grid: the dims-driven geometry reads it
        // rectilinearly at the fixture offset. (The fixed 256-lattice
        // geometry happens to validate a mangled window inside this
        // very smooth fixture too — that is exactly why headerless
        // hits are only candidates.)
        let (d, base) = fixture_lattice(512, 512);
        let geo = Geometry::for_lattice(512, 512);
        let g = scan_geo(&d, 0x3000, &geo).expect("512-lattice grid found");
        assert_eq!(g.offset, base);
        assert_eq!((g.rows, g.cols), (513, 513));
    }

    /// Regression (review round 1, finding 1): when a dims header names
    /// a non-square lattice and BOTH orientations validate, the correct
    /// one cannot be distinguished — scan_with_dims must refuse rather
    /// than return a wrong-orientation hit as verified.
    #[test]
    fn ambiguous_orientation_is_refused() {
        // A linear height ramp validates under any row pitch, so a
        // (512,128) header over ramp data yields hits under BOTH
        // orientations — scan_with_dims must refuse the ambiguity
        // rather than return one as verified.
        let ramp_cells = 80_000usize; // > max window of either geometry
        let mut file = vec![0x7Fu8; 0x4000 + ramp_cells * 4 + 0x200];
        for i in 0..ramp_cells {
            let v = i as f32 * 0.001; // smooth under any pitch, relief ~80 m
            let o = 0x4000 + i * 4;
            file[o..o + 4].copy_from_slice(&v.to_le_bytes());
        }
        let hdr_at = 0x4000 + ramp_cells * 4 + 0x40;
        file[hdr_at..hdr_at + 4].copy_from_slice(&0x2EE2u32.to_le_bytes());
        for (i, v) in [32u32, 32, 32, 32, 512, 128, 2048, 512, 0]
            .into_iter()
            .enumerate()
        {
            let o = hdr_at + 4 + 4 * i;
            file[o..o + 4].copy_from_slice(&v.to_le_bytes());
        }
        let o = hdr_at + 40;
        file[o..o + 4].copy_from_slice(&0x2EE3u32.to_le_bytes());
        let dims = read_dims(&file).expect("dims header parses");
        assert_eq!(dims.lattice(), (512, 128));
        assert!(
            scan_with_dims(&file, 0x3000, &dims).is_none(),
            "ambiguity must be refused"
        );
        // Each orientation alone validates on the ramp.
        assert!(scan_geo(&file, 0x3000, &Geometry::for_lattice(512, 128)).is_some());
        assert!(scan_geo(&file, 0x3000, &Geometry::for_lattice(128, 512)).is_some());
    }

    /// Regression (review round 1, finding 2): geometries too small for
    /// the probe lattice are rejected outright, so the probe can never
    /// sample outside the window the validator checks.
    #[test]
    fn rejects_unsupported_small_geometry() {
        let tiny = Geometry::for_lattice(27, 27);
        assert!(!tiny.supported());
        // A window full of valid heights under a tiny geometry: refused
        // (no verified claim), not mis-sampled.
        let (d, _) = fixture_lattice(256, 256);
        assert!(scan_geo(&d, 0x3000, &tiny).is_none());
        assert!(scan_all_geo(&d, 0x3000, &tiny).is_empty());
    }

    /// The map record's extent word anchors the lattice height
    /// (word0 = lattice_h × 256; exact on all 13 header-bearing maps).
    #[test]
    fn record_word_anchors_height() {
        let mut d = vec![0u8; 64];
        d[12..16].copy_from_slice(&0x28000u32.to_le_bytes());
        assert_eq!(lattice_h_from_record(&d), Some(640));
        d[12..16].copy_from_slice(&0x10001u32.to_le_bytes());
        assert_eq!(lattice_h_from_record(&d), None); // not a multiple of 256
        d[12..16].copy_from_slice(&0x10u32.to_le_bytes());
        assert_eq!(lattice_h_from_record(&d), None); // out of range
        assert_eq!(lattice_h_from_record(&[0u8; 8]), None);
    }

    /// With a usable record anchor, the anchored orientation is scanned
    /// even though the other would also validate (the ambiguity refusal
    /// applies only when no anchor exists).
    #[test]
    fn record_anchor_resolves_orientation() {
        // Ramp data (validates under any pitch), a (512,128) header, and
        // a record word anchoring h = 128.
        let ramp_cells = 80_000usize;
        let mut file = vec![0x7Fu8; 0x4000 + ramp_cells * 4 + 0x200];
        for i in 0..ramp_cells {
            let v = i as f32 * 0.001;
            let o = 0x4000 + i * 4;
            file[o..o + 4].copy_from_slice(&v.to_le_bytes());
        }
        file[12..16].copy_from_slice(&(128u32 * 256).to_le_bytes());
        let hdr_at = 0x4000 + ramp_cells * 4 + 0x40;
        file[hdr_at..hdr_at + 4].copy_from_slice(&0x2EE2u32.to_le_bytes());
        for (i, v) in [32u32, 32, 32, 32, 512, 128, 2048, 512, 0]
            .into_iter()
            .enumerate()
        {
            let o = hdr_at + 4 + 4 * i;
            file[o..o + 4].copy_from_slice(&v.to_le_bytes());
        }
        let o = hdr_at + 40;
        file[o..o + 4].copy_from_slice(&0x2EE3u32.to_le_bytes());
        let dims = read_dims(&file).expect("dims header parses");
        let g = scan_with_dims(&file, 0x3000, &dims).expect("anchored orientation scanned");
        assert_eq!((g.rows, g.cols), (129, 513));
    }

    /// The pitch funnel: a headerless (512,576)-lattice fixture with a
    /// record word anchoring h=576 measures row pitch 514 and scans at
    /// (512,576).
    #[test]
    fn pitch_anchors_headerless_width() {
        let (mut d, base) = fixture_lattice(512, 576);
        // record extent word: lattice_h × 256
        d[12..16].copy_from_slice(&(576u32 * 256).to_le_bytes());
        let (row_cells, win, run) = measure_pitch(&d).expect("pitch measured");
        assert_eq!(row_cells, 514);
        assert!(win <= run * 0.6, "separation win={win} run={run}");
        let (g, kind) = scan_headerless(&d, 0x3000).expect("ladder finds the grid");
        assert_eq!(kind, HeaderlessHit::PitchAnchored);
        assert_eq!(g.offset, base);
        assert_eq!((g.rows, g.cols), (577, 513));
    }

    /// A record height of 256 with a measured width of 512 must report
    /// the pitch-anchored hit, never the fixed-256 candidate (the
    /// ladder stops once the pitch has spoken). Without a usable
    /// record word there is no result at all.
    #[test]
    fn ladder_never_guesses_after_a_pitch() {
        let (mut d, base) = fixture_lattice(512, 256);
        d[12..16].copy_from_slice(&(256u32 * 256).to_le_bytes());
        let (g, kind) = scan_headerless(&d, 0x3000).expect("pitch-anchored hit");
        assert_eq!(kind, HeaderlessHit::PitchAnchored);
        assert_eq!((g.rows, g.cols), (257, 513));
        assert_eq!(g.offset, base);
        // No record word (filler bytes): the ladder cannot even start.
        let (d2, _) = fixture_lattice(512, 256);
        assert!(scan_headerless(&d2, 0x3000).is_none());
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
        assert_eq!(dims.lattice(), (256, 256));
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
    /// probe's spread cannot see (blocks offset between probe rows and
    /// columns) still passes full validation — the probe must never be
    /// an accept-side shortcut.
    #[test]
    fn finds_probe_invisible_land() {
        let geo = Geometry::default_256();
        let mut d = vec![0u8; 0x4000 + geo.grid_bytes() + 16];
        let base = 0x4000;
        for r in 0..geo.rows {
            for c in 0..geo.row_cells {
                let land = r % 32 >= 16 && r % 32 < 24 && c % 32 >= 16 && c % 32 < 24;
                let v = if land { 30.0f32 } else { 0.0 };
                let o = base + geo.cell_off(r, c);
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
        assert!(g.offset <= base);
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

    /// Install-backed smoke checks: the two render-corroborated maps
    /// scan to sane grids with their own dims geometry (full per-map
    /// coverage lives in the `terrain-scan-v1.json` evidence artifact).
    /// Self-skips without `RT3_HOME`.
    #[test]
    fn scans_reference_maps() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed terrain check");
            return;
        };
        for (file, lattice, max_min, max_max) in [
            (
                "maps/Crossing the Alps.gmp",
                (256usize, 256usize),
                2000.0f32,
                2600.0f32,
            ),
            ("maps/Germany.gmp", (512, 512), 2000.0, 2400.0),
            // Non-square lattice: orientation resolved by the record
            // extent word (h = 640).
            ("maps/Alternate USA.gmp", (1024, 640), 1500.0, 2100.0),
        ] {
            let path = std::path::Path::new(&home).join(file);
            let Ok(bytes) = fs::read(&path) else {
                eprintln!("{} missing; skipping", path.display());
                return;
            };
            let dims =
                read_dims(&bytes).unwrap_or_else(|| panic!("no dims header in {}", path.display()));
            assert_eq!(dims.lattice(), lattice, "{}", path.display());
            let g = scan_with_dims(&bytes, 0x3000, &dims)
                .unwrap_or_else(|| panic!("no grid in {}", path.display()));
            let (lo, hi) = g.height_range().unwrap();
            assert!(
                g.offset < 0x20_0000,
                "{}: offset {:#x}",
                path.display(),
                g.offset
            );
            assert!(
                lo >= -1000.0 && hi >= max_min && hi <= max_max,
                "{}: range {}..{}",
                path.display(),
                lo,
                hi
            );
        }
    }

    /// Install-backed check for the headerless pitch-anchored path
    /// (record height + measured width). Self-skips without RT3_HOME.
    #[test]
    fn scans_pitch_anchored_reference_maps() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed pitch check");
            return;
        };
        for (file, lattice, max_min, max_max) in [
            (
                "maps/Tex-Mex.gmp",
                (512usize, 576usize),
                2000.0f32,
                2800.0f32,
            ),
            ("maps/Chicago to New York.gmp", (640, 512), 1500.0, 2100.0),
        ] {
            let path = std::path::Path::new(&home).join(file);
            let Ok(bytes) = fs::read(&path) else {
                eprintln!("{} missing; skipping", path.display());
                return;
            };
            assert!(
                read_dims(&bytes).is_none(),
                "{}: expected no dims header",
                path.display()
            );
            assert_eq!(
                lattice_h_from_record(&bytes),
                Some(lattice.1),
                "{}",
                path.display()
            );
            let (g, kind) = scan_headerless(&bytes, 0x3000)
                .unwrap_or_else(|| panic!("no pitch-anchored grid in {}", path.display()));
            assert_eq!(kind, HeaderlessHit::PitchAnchored, "{}", path.display());
            assert_eq!((g.cols - 1, g.rows - 1), lattice, "{}", path.display());
            let (lo, hi) = g.height_range().unwrap();
            assert!(
                lo >= -1000.0 && hi >= max_min && hi <= max_max,
                "{}: range {}..{}",
                path.display(),
                lo,
                hi
            );
        }
    }
}
