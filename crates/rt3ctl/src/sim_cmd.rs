//! `rt3ctl sim` — headless engine commands (milestone 5).
//!
//! - `sim inspect <map>` loads a map-grounded world (claim C1) and reports
//!   structural data only.
//! - `sim run <map>` runs the declared cargo economy (claims C2–C4), writing
//!   one JSON dump line per simulated month and a summary with an explicit
//!   stop reason and the dump's SHA-256.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use engine::economy::{Economy, ModelRules};
use engine::world;
use rt3_kit::save::sha256;

use crate::map_cmd::json_escape;

/// Arguments of `rt3ctl sim inspect`.
pub struct SimInspect {
    pub name: String,
    pub path: Option<PathBuf>,
    pub json: bool,
}

/// Arguments of `rt3ctl sim run`.
pub struct SimRun {
    pub name: String,
    pub path: Option<PathBuf>,
    pub seed: u64,
    pub months: u32,
    pub ticks_per_month: u32,
    pub dump: Option<PathBuf>,
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

/// Parses the arguments after `sim run`.
pub fn parse_run<'a, I>(args: I) -> Result<SimRun, String>
where
    I: Iterator<Item = &'a String>,
{
    let mut name = None;
    let mut path = None;
    let mut seed = 0u64;
    let mut months = 120u32;
    let mut ticks_per_month = ModelRules::default().ticks_per_month;
    let mut dump = None;
    let mut rest = args;
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--path" => {
                let value = rest
                    .next()
                    .filter(|v| !v.starts_with("--"))
                    .ok_or_else(|| "sim run: --path needs a value".to_string())?;
                path = Some(PathBuf::from(value));
            }
            "--seed" => {
                let value = rest
                    .next()
                    .filter(|v| !v.starts_with("--"))
                    .ok_or_else(|| "sim run: --seed needs a value".to_string())?;
                seed = value
                    .parse()
                    .map_err(|_| format!("sim run: bad --seed {value:?}"))?;
            }
            "--months" => {
                let value = rest
                    .next()
                    .filter(|v| !v.starts_with("--"))
                    .ok_or_else(|| "sim run: --months needs a value".to_string())?;
                months = value
                    .parse()
                    .map_err(|_| format!("sim run: bad --months {value:?}"))?;
            }
            "--ticks-per-month" => {
                let value = rest
                    .next()
                    .filter(|v| !v.starts_with("--"))
                    .ok_or_else(|| "sim run: --ticks-per-month needs a value".to_string())?;
                ticks_per_month = value
                    .parse()
                    .map_err(|_| format!("sim run: bad --ticks-per-month {value:?}"))?;
            }
            "--dump" => {
                let value = rest
                    .next()
                    .filter(|v| !v.starts_with("--"))
                    .ok_or_else(|| "sim run: --dump needs a value".to_string())?;
                dump = Some(PathBuf::from(value));
            }
            other if other.starts_with("--") => {
                return Err(format!("sim run: unknown option {other}"));
            }
            other => {
                if name.is_some() {
                    return Err(format!("sim run: unexpected extra argument {other}"));
                }
                name = Some(other.to_string());
            }
        }
    }
    let name = name.ok_or_else(|| "sim run needs a map name".to_string())?;
    if months == 0 {
        return Err("sim run: --months must be at least 1".to_string());
    }
    if ticks_per_month == 0 {
        return Err("sim run: --ticks-per-month must be at least 1".to_string());
    }
    Ok(SimRun {
        name,
        path,
        seed,
        months,
        ticks_per_month,
        dump,
    })
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

fn hex(bytes: &[u8]) -> String {
    sha256(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn summary(run: &SimRun, world: &world::World, dump: &str) -> String {
    format!(
        "{{\"map\":{},\"seed\":{},\"months\":{},\"stop_reason\":\"completed\",\"dump_sha256\":{}}}",
        json_escape(&world.map_name),
        run.seed,
        run.months,
        json_escape(&hex(dump.as_bytes()))
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

/// Runs `rt3ctl sim run`.
pub fn run_simulation(run: SimRun) -> ExitCode {
    let Some(root) = crate::install_root(run.path.clone()) else {
        eprintln!("rt3ctl: no installation root: pass --path or set RT3_HOME");
        return ExitCode::from(2);
    };
    let map = match resolve_map(&root, &run.name) {
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
    let rules = ModelRules {
        ticks_per_month: run.ticks_per_month,
        ..ModelRules::default()
    };
    let mut economy = Economy::new(&world, rules, run.seed);
    let dump = economy.run_months(run.months);
    let stop = summary(&run, &world, &dump);
    match &run.dump {
        Some(path) => {
            if let Err(error) = fs::write(path, dump.as_bytes()) {
                eprintln!("rt3ctl: cannot write {}: {error}", path.display());
                return ExitCode::from(1);
            }
            println!("{stop}");
        }
        None => {
            print!("{dump}");
            eprintln!("{stop}");
        }
    }
    ExitCode::SUCCESS
}
