//! `rt3ctl sim inspect` — load a shipped map into the engine world
//! (milestone 5, claim C1). Structural data only: terrain heightfield, city
//! and placed-building record counts, and the typed cargo catalogue.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use engine::world;

use crate::map_cmd::json_escape;

/// Arguments of `rt3ctl sim inspect`.
pub struct SimInspect {
    pub name: String,
    pub path: Option<PathBuf>,
    pub json: bool,
}

/// Parses the arguments after `sim inspect`.
pub fn parse<'a, I>(args: I) -> Result<SimInspect, String>
where
    I: Iterator<Item = &'a String>,
{
    let mut name = None;
    let mut path = None;
    let mut json = false;
    let mut rest = args;
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--path" => {
                let value = rest
                    .next()
                    .filter(|v| !v.starts_with("--"))
                    .ok_or_else(|| "sim inspect: --path needs a value".to_string())?;
                path = Some(PathBuf::from(value));
            }
            other if other.starts_with("--") => {
                return Err(format!("sim inspect: unknown option {other}"));
            }
            other => {
                if name.is_some() {
                    return Err(format!("sim inspect: unexpected extra argument {other}"));
                }
                name = Some(other.to_string());
            }
        }
    }
    let name = name.ok_or_else(|| "sim inspect needs a map name".to_string())?;
    Ok(SimInspect { name, path, json })
}

fn resolve_map(root: &Path, name: &str) -> Result<PathBuf, String> {
    let direct = PathBuf::from(name);
    if direct.is_file() {
        return Ok(direct);
    }
    let mut found = Vec::new();
    for dir in ["maps", "Data/Tutorial"] {
        let Ok(entries) = fs::read_dir(root.join(dir)) else {
            continue;
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            let is_gmp = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("gmp"))
                .unwrap_or(false);
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if is_gmp && stem.eq_ignore_ascii_case(name) {
                found.push(path);
            }
        }
    }
    found.sort();
    match found.len() {
        0 => Err(format!("no .gmp named {name:?} under {}", root.display())),
        1 => Ok(found.remove(0)),
        _ => Err(format!(
            "more than one .gmp named {name:?} under {}",
            root.display()
        )),
    }
}

fn to_json(world: &world::World) -> String {
    let (min, max) = world.terrain.height_range().unwrap_or((0.0, 0.0));
    let names: Vec<String> = world
        .cargo
        .iter()
        .map(|cargo| json_escape(&cargo.name))
        .collect();
    format!(
        "{{\n  \"map\": {},\n  \"terrain\": {{\"rows\": {}, \"cols\": {}, \"min_height_m\": {:.1}, \"max_height_m\": {:.1}}},\n  \"cities\": {},\n  \"buildings\": {},\n  \"cargo\": {{\"types\": {}, \"names\": [{}]}}\n}}",
        json_escape(&world.map_name),
        world.terrain.rows,
        world.terrain.cols,
        min,
        max,
        world.city_records,
        world.building_records,
        world.cargo.len(),
        names.join(", ")
    )
}

/// Runs `rt3ctl sim inspect`.
pub fn run(inspect: SimInspect) -> ExitCode {
    let Some(root) = crate::install_root(inspect.path) else {
        eprintln!("rt3ctl: no installation root: pass --path or set RT3_HOME");
        return ExitCode::from(2);
    };
    let map = match resolve_map(&root, &inspect.name) {
        Ok(path) => path,
        Err(message) => {
            eprintln!("rt3ctl: {message}");
            return ExitCode::from(2);
        }
    };
    let world = match world::load(&map, &root) {
        Ok(world) => world,
        Err(error) => {
            eprintln!("rt3ctl: {error}");
            return ExitCode::from(1);
        }
    };
    if inspect.json {
        println!("{}", to_json(&world));
    } else {
        let (min, max) = world.terrain.height_range().unwrap_or((0.0, 0.0));
        println!(
            "{}: terrain {}x{} ({min:.1} m..{max:.1} m), cities {}, buildings {}, cargo {}",
            world.map_name,
            world.terrain.cols,
            world.terrain.rows,
            world.city_records,
            world.building_records,
            world.cargo.len()
        );
    }
    ExitCode::SUCCESS
}
