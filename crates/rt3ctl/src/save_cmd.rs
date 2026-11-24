//! `rt3ctl save inspect` — read-only original `.gmc` inventory.

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use rt3_kit::save;

pub struct SaveInspect {
    pub path: PathBuf,
    pub json: bool,
}

pub fn run(command: SaveInspect) -> ExitCode {
    let bytes = match fs::read(&command.path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("rt3ctl: cannot read {}: {e}", command.path.display());
            return ExitCode::from(1);
        }
    };
    let Some(save) = save::from_bytes(&bytes) else {
        eprintln!(
            "rt3ctl: {}: not a supported version-9701 save with all proven sections",
            command.path.display()
        );
        return ExitCode::from(1);
    };
    if !command.json {
        println!(
            "{}: {} bytes, terrain {}x{}, tiles {}, cities {}, buildings {}, events {}",
            command.path.display(),
            save.file_bytes,
            save.terrain.cols,
            save.terrain.rows,
            save.tiles.records.len(),
            save.cities.records.len(),
            save.buildings.records.len(),
            save.events.records.len()
        );
        return ExitCode::SUCCESS;
    }
    let name = command
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("save.gmc");
    let (lo, hi) = save.terrain.height_range().unwrap_or((0.0, 0.0));
    println!("{{");
    println!("  \"file\": {},", crate::map_cmd::json_escape(name));
    println!("  \"version\": 9701,");
    println!("  \"file_bytes\": {},", save.file_bytes);
    println!(
        "  \"sha256\": {},",
        crate::map_cmd::json_escape(&save.sha256_hex())
    );
    println!(
        "  \"terrain\": {{\"grid_offset\": {}, \"rows\": {}, \"cols\": {}, \"min_height_m\": {lo:.1}, \"max_height_m\": {hi:.1}}},",
        save.terrain.offset, save.terrain.rows, save.terrain.cols
    );
    println!(
        "  \"tiles\": {{\"section_offset\": {}, \"records\": {}, \"dynamic_entries\": {}, \"end_offset\": {}}},",
        save.tiles.section_offset,
        save.tiles.records.len(),
        save.tiles.dynamic_entries(),
        save.tiles.end_offset
    );
    println!(
        "  \"cities\": {{\"section_offset\": {}, \"capacity\": {}, \"records\": {}, \"end_offset\": {}}},",
        save.cities.section_offset, save.cities.capacity, save.cities.records.len(), save.cities.end_offset
    );
    println!(
        "  \"buildings\": {{\"section_offset\": {}, \"capacity\": {}, \"records\": {}, \"end_offset\": {}}},",
        save.buildings.section_offset,
        save.buildings.capacity,
        save.buildings.records.len(),
        save.buildings.end_offset
    );
    println!(
        "  \"events\": {{\"section_offset\": {}, \"capacity\": {}, \"records\": {}, \"end_offset\": {}}}",
        save.events.section_offset, save.events.capacity, save.events.records.len(), save.events.end_offset
    );
    println!("}}");
    ExitCode::SUCCESS
}
