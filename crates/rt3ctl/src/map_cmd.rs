//! `rt3ctl map list` — inventory the shipped `.gmp` scenario maps.
//! `rt3ctl map terrain <name>` — extract and preview the coarse
//! terrain heightfield of one map (milestone 3b).

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use rt3_kit::map::GmpMap;
use rt3_kit::render;
use rt3_kit::terrain;

/// Arguments of `rt3ctl map list`.
pub struct MapList {
    pub path: Option<PathBuf>,
    pub json: bool,
}

/// Arguments of `rt3ctl map terrain`.
pub struct MapTerrain {
    pub path: Option<PathBuf>,
    pub name: Option<String>,
    pub all: bool,
    pub json: bool,
    pub png: Option<PathBuf>,
}

/// Parses the arguments after `map list`.
pub fn parse<'a, I>(args: I) -> MapList
where
    I: Iterator<Item = &'a String>,
{
    let mut list = MapList {
        path: None,
        json: false,
    };
    let mut rest = args;
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--json" => list.json = true,
            "--path" => {
                if let Some(value) = rest.next() {
                    list.path = Some(PathBuf::from(value));
                }
            }
            other => eprintln!("rt3ctl: ignoring unknown map list argument: {other}"),
        }
    }
    list
}

struct Row {
    name: String,
    version: u32,
    status: &'static str,
    text_len: usize,
    tail_len: u64,
    file_len: u64,
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn checked_png_path(root: &std::path::Path, out: &std::path::Path) -> Result<PathBuf, String> {
    let root = root
        .canonicalize()
        .map_err(|e| format!("cannot resolve installation root: {e}"))?;
    let file = out
        .file_name()
        .ok_or_else(|| "PNG output needs a file name".to_string())?;
    let parent = out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."))
        .canonicalize()
        .map_err(|e| format!("cannot resolve PNG output parent: {e}"))?;
    let resolved = if out.exists() {
        out.canonicalize()
            .map_err(|e| format!("cannot resolve PNG output: {e}"))?
    } else {
        parent.join(file)
    };
    if resolved.starts_with(&root) {
        return Err("refusing to write a PNG inside the read-only installation".to_string());
    }
    Ok(resolved)
}

/// Runs `map terrain`: one map (`<name>`) prints statistics plus an
/// ASCII relief preview and optionally writes a north-up RGB PNG;
/// `--all` scans every current-format map and prints one summary line
/// each; `--all --json` emits the machine form that backs
/// `docs/evidence/terrain-scan-v1.json`.
pub fn run_terrain(t: MapTerrain) -> ExitCode {
    let Some(root) = t
        .path
        .clone()
        .or_else(|| std::env::var_os("RT3_HOME").map(PathBuf::from))
    else {
        eprintln!("rt3ctl: map terrain needs --path or RT3_HOME");
        return ExitCode::from(2);
    };
    if !root.is_dir() {
        eprintln!(
            "rt3ctl: installation root is not a directory: {}",
            root.display()
        );
        return ExitCode::from(2);
    }
    if t.all {
        return run_terrain_all(&root, t.json);
    }
    let Some(name) = t.name.clone() else {
        eprintln!("rt3ctl: map terrain needs a map name or --all");
        return ExitCode::from(2);
    };
    let mut candidates = Vec::new();
    for dir in ["maps", "Data/Tutorial"] {
        let dir = root.join(dir);
        if let Ok(rd) = fs::read_dir(&dir) {
            for entry in rd.filter_map(|e| e.ok()) {
                let p = entry.path();
                let stem = p.file_stem().and_then(|n| n.to_str()).unwrap_or("");
                if p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("gmp"))
                    .unwrap_or(false)
                    && stem.eq_ignore_ascii_case(&name)
                {
                    candidates.push(p);
                }
            }
        }
    }
    let Some(path) = candidates.first() else {
        eprintln!("rt3ctl: no .gmp named {name:?} under the installation");
        return ExitCode::from(2);
    };
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("rt3ctl: {}: {e}", path.display());
            return ExitCode::from(1);
        }
    };
    if GmpMap::peek_version(&bytes) != Some(9701) {
        eprintln!(
            "rt3ctl: {}: not a current-format (9701) map",
            path.display()
        );
        return ExitCode::from(1);
    }
    let Some((g, kind)) = scan_map(&bytes) else {
        eprintln!("rt3ctl: {}: no validated coarse grid found", path.display());
        return ExitCode::from(1);
    };
    if let Some(out) = &t.png {
        let out = match checked_png_path(&root, out) {
            Ok(path) => path,
            Err(message) => {
                eprintln!("rt3ctl: {message}");
                return ExitCode::from(1);
            }
        };
        if let Err(e) = fs::write(&out, render::encode_png(&g)) {
            eprintln!("rt3ctl: cannot write {}: {e}", out.display());
            return ExitCode::from(1);
        }
        println!("wrote north-up RGB PNG: {}", out.display());
    }
    let (lo, hi) = g.height_range().unwrap_or((0.0, 0.0));
    println!(
        "{}: grid at {:#x} ({}x{}, {} m..{} m, NaN {} / zero {}) [{}]",
        path.display(),
        g.offset,
        g.cols,
        g.rows,
        lo,
        hi,
        g.nan_cells,
        g.zero_cells,
        kind.as_str()
    );
    print_relief(&g);
    ExitCode::SUCCESS
}

fn terrain_paths(root: &std::path::Path) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    for dir in ["maps", "Data/Tutorial"] {
        let dir = root.join(dir);
        if let Ok(rd) = fs::read_dir(&dir) {
            for entry in rd.filter_map(|e| e.ok()) {
                let p = entry.path();
                if p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("gmp"))
                    .unwrap_or(false)
                {
                    if let Some(stem) = p.file_stem().and_then(|n| n.to_str()) {
                        out.push((stem.to_string(), p.clone()));
                    }
                }
            }
        }
    }
    out.sort();
    out
}

/// A hit counts as *verified* when its exact start and geometry come
/// from the map's explicit terrain frame, or when it was scanned with
/// the geometry read from its older dims header. A hit from the fixed
/// 256-lattice geometry without either frame is a *candidate*.
#[derive(Clone, Copy, PartialEq)]
enum HitKind {
    Verified,
    PitchAnchored,
    Candidate,
}

impl HitKind {
    fn as_str(self) -> &'static str {
        match self {
            HitKind::Verified => "verified",
            HitKind::PitchAnchored => "pitch-anchored",
            HitKind::Candidate => "candidate",
        }
    }
}

/// Scans one map. The status ladder:
/// - **verified**: exact start and geometry from the explicit terrain
///   frame, or geometry from the older record-anchored dims header.
/// - **pitch-anchored** / **candidate**: the unframed refusal ladder
///   (`terrain::scan_headerless`) — a measured pitch that fails
///   validation ends the ladder; the fixed-256 candidate runs only
///   with no measurable pitch and a record height of 256.
fn scan_map(bytes: &[u8]) -> Option<(terrain::HeightGrid, HitKind)> {
    if let Some(g) = terrain::read_framed(bytes) {
        return Some((g, HitKind::Verified));
    }
    if let Some(dims) = terrain::read_dims(bytes) {
        return terrain::scan_with_dims(bytes, 0x3000, &dims).map(|g| (g, HitKind::Verified));
    }
    terrain::scan_headerless(bytes, 0x3000).map(|(g, kind)| {
        let kind = match kind {
            terrain::HeaderlessHit::PitchAnchored => HitKind::PitchAnchored,
            terrain::HeaderlessHit::Candidate => HitKind::Candidate,
        };
        (g, kind)
    })
}

fn run_terrain_all(root: &std::path::Path, json: bool) -> ExitCode {
    let paths = terrain_paths(root);
    if paths.is_empty() {
        // A silent empty listing would look like a successful scan and
        // could replace the evidence artifact with an empty result.
        eprintln!(
            "rt3ctl: no .gmp maps under {} (expected maps/ or Data/Tutorial)",
            root.display()
        );
        return ExitCode::from(2);
    }
    struct Row {
        name: String,
        dims: Option<terrain::TerrainDims>,
        grid: Option<(usize, usize, usize, f32, f32, usize, usize)>,
        status: Option<&'static str>,
        missed: Option<&'static str>,
    }

    let mut rows = Vec::new();
    for (name, path) in &paths {
        let Ok(bytes) = fs::read(path) else {
            rows.push(Row {
                name: name.clone(),
                dims: None,
                grid: None,
                status: None,
                missed: Some("unreadable"),
            });
            continue;
        };
        if GmpMap::peek_version(&bytes) != Some(9701) {
            continue; // legacy layout: separate claim
        }
        let dims = terrain::read_dims(&bytes);
        let scanned = scan_map(&bytes);
        let grid = scanned.as_ref().map(|(g, _)| {
            let (lo, hi) = g.height_range().unwrap_or((0.0, 0.0));
            (g.offset, g.nan_cells, g.zero_cells, lo, hi, g.rows, g.cols)
        });
        let status = scanned.as_ref().map(|(_, kind)| kind.as_str());
        let missed = grid.is_none().then_some("no validated grid");
        rows.push(Row {
            name: name.clone(),
            dims,
            grid,
            status,
            missed,
        });
    }
    if json {
        println!("{{");
        println!("  \"maps\": [");
        let n = rows.len();
        for (i, r) in rows.iter().enumerate() {
            let comma = if i + 1 < n { "," } else { "" };
            let dims = match &r.dims {
                Some(d) => format!(
                    "{{\"offset\": {}, \"values\": [{}]}}",
                    d.header_offset,
                    d.values
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                None => "null".to_string(),
            };
            let grid = match &r.grid {
                Some((off, nan, zero, lo, hi, rows, cols)) => format!(
                    "{{\"grid_offset\": {off}, \"rows\": {rows}, \"cols\": {cols}, \"nan_cells\": {nan}, \"zero_cells\": {zero}, \"min_height_m\": {lo:.1}, \"max_height_m\": {hi:.1}}}"
                ),
                None => "null".to_string(),
            };
            let status = match r.status {
                Some(st) => format!("\"{st}\""),
                None => "null".to_string(),
            };
            let missed = match r.missed {
                Some(why) => format!("\"{why}\""),
                None => "null".to_string(),
            };
            println!(
                "    {{\"map\": {}, \"dims\": {dims}, \"grid\": {grid}, \"status\": {status}, \"missed\": {missed}}}{comma}",
                json_escape(&r.name)
            );
        }
        println!("  ]");
        println!("}}");
    } else {
        for r in &rows {
            let status = r.status.unwrap_or_default();
            match (&r.dims, &r.grid) {
                (Some(d), Some(gr)) => println!(
                    "{:<26} dims@{:#8x} [{},{},{},{}]  grid@{:#8x} {:.0}..{:.0} m  {status}",
                    r.name,
                    d.header_offset,
                    d.values[0],
                    d.values[2],
                    d.values[4],
                    d.values[6],
                    gr.0,
                    gr.3,
                    gr.4
                ),
                (Some(d), None) => println!(
                    "{:<26} dims@{:#8x} [{},{},{},{}]  no raw grid",
                    r.name, d.header_offset, d.values[0], d.values[2], d.values[4], d.values[6]
                ),
                (None, Some(gr)) => println!(
                    "{:<26} no dims header            grid@{:#8x} {:.0}..{:.0} m  {status}",
                    r.name, gr.0, gr.3, gr.4
                ),
                (None, None) => println!("{:<26} no dims header, no raw grid", r.name),
            }
        }
    }
    ExitCode::SUCCESS
}

/// ASCII relief: block means over the grid, shaded from the 5th to the
/// 95th percentile so plains and peaks both show (validated against the
/// oracle's in-game screenshots of the same map).
fn print_relief(g: &terrain::HeightGrid) {
    let mut sorted: Vec<f32> = g.heights.iter().copied().filter(|v| !v.is_nan()).collect();
    if sorted.is_empty() {
        return;
    }
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p5 = sorted[sorted.len() / 20];
    let p95 = sorted[sorted.len() * 19 / 20];
    let chars: Vec<char> = " .:-=+*#%@".chars().collect();
    for r in 0..g.rows / 8 {
        let mut line = String::new();
        for c in 0..g.cols / 6 {
            let mut s = 0.0f32;
            let mut n = 0usize;
            for dr in 0..8 {
                for dc in 0..6 {
                    let v = g.get(r * 8 + dr, c * 6 + dc);
                    if !v.is_nan() {
                        s += v;
                        n += 1;
                    }
                }
            }
            let t = if n == 0 {
                0.0
            } else {
                ((s / n as f32 - p5) / (p95 - p5).max(1.0)).clamp(0.0, 0.999)
            };
            line.push(chars[(t * 10.0) as usize]);
        }
        println!("{line}");
    }
}

/// Runs `map list`.
pub fn run(list: MapList) -> ExitCode {
    let Some(root) = list
        .path
        .clone()
        .or_else(|| std::env::var_os("RT3_HOME").map(PathBuf::from))
    else {
        eprintln!("rt3ctl: map list needs --path or RT3_HOME");
        return ExitCode::from(2);
    };
    if !root.is_dir() {
        eprintln!(
            "rt3ctl: installation root is not a directory: {}",
            root.display()
        );
        return ExitCode::from(2);
    }

    let mut paths = Vec::new();
    for dir in ["maps", "Data/Tutorial"] {
        let dir = root.join(dir);
        if let Ok(rd) = fs::read_dir(&dir) {
            for entry in rd.filter_map(|e| e.ok()) {
                let p = entry.path();
                let is_gmp = p
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("gmp"))
                    .unwrap_or(false);
                if is_gmp {
                    paths.push(p);
                }
            }
        }
    }
    paths.sort();

    let mut rows = Vec::new();
    let mut failures = Vec::new();
    for path in &paths {
        let name = path
            .file_stem()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        match fs::read(path) {
            Ok(bytes) => match GmpMap::peek_version(&bytes) {
                Some(v @ 9609..=9610) => rows.push(Row {
                    name,
                    version: v,
                    status: "legacy",
                    text_len: 0,
                    tail_len: 0,
                    file_len: bytes.len() as u64,
                }),
                Some(9701) => match GmpMap::from_bytes(path, &bytes) {
                    Ok(map) => rows.push(Row {
                        name,
                        version: map.version,
                        status: "current",
                        text_len: map.text_len,
                        tail_len: map.tail_len,
                        file_len: map.file_len,
                    }),
                    Err(e) => failures.push(e.to_string()),
                },
                other => failures.push(format!("{}: unexpected version {other:?}", path.display())),
            },
            Err(e) => failures.push(format!("{}: {e}", path.display())),
        }
    }
    if !failures.is_empty() {
        for f in &failures {
            eprintln!("rt3ctl: {f}");
        }
        return ExitCode::from(1);
    }

    let current = rows.iter().filter(|r| r.status == "current").count();
    let legacy = rows.len() - current;
    if list.json {
        println!("{{");
        println!("  \"maps\": [");
        for (i, r) in rows.iter().enumerate() {
            let comma = if i + 1 < rows.len() { "," } else { "" };
            println!(
                "    {{\"map\": {}, \"version\": {}, \"status\": {}, \"text_len\": {}, \"tail_bytes\": {}, \"file_bytes\": {}}}{}",
                json_escape(&r.name),
                r.version,
                json_escape(r.status),
                r.text_len,
                r.tail_len,
                r.file_len,
                comma
            );
        }
        println!("  ],");
        println!(
            "  \"total\": {{\"maps\": {}, \"current\": {}, \"legacy\": {}}}",
            rows.len(),
            current,
            legacy
        );
        println!("}}");
    } else {
        for r in &rows {
            println!(
                "{:<28} v{} {:>7} text {:>10} tail {:>10} bytes",
                r.name, r.version, r.status, r.text_len, r.tail_len
            );
        }
        println!(
            "total: {} maps ({} current, {} legacy)",
            rows.len(),
            current,
            legacy
        );
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_path_refuses_install_root_and_symlink_escape() {
        let base = std::env::temp_dir().join(format!("rt3ctl-png-path-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let install = base.join("install");
        let output = base.join("output");
        fs::create_dir_all(&install).unwrap();
        fs::create_dir_all(&output).unwrap();
        assert!(checked_png_path(&install, &output.join("map.png")).is_ok());
        assert!(checked_png_path(&install, &install.join("map.png")).is_err());
        #[cfg(unix)]
        {
            fs::write(install.join("original.gmp"), b"original").unwrap();
            std::os::unix::fs::symlink(install.join("original.gmp"), output.join("escape.png"))
                .unwrap();
            assert!(checked_png_path(&install, &output.join("escape.png")).is_err());
        }
        fs::remove_dir_all(base).unwrap();
    }
}
