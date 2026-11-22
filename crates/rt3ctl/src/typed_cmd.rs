//! `rt3ctl typed list` — parse the typed game-data families and summarize.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use rt3_kit::typed::{
    scan_family, BuildingBehavior, BuildingType, CarType, CargoType, EngineConfig, GameConfig,
    LanguageFile, Locomotive, TextTable,
};

/// Arguments of `rt3ctl typed list`.
pub struct TypedList {
    pub path: Option<PathBuf>,
    pub json: bool,
}

/// Parses the arguments after `typed list`.
pub fn parse<'a, I>(args: I) -> TypedList
where
    I: Iterator<Item = &'a String>,
{
    let mut list = TypedList {
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
            other => eprintln!("rt3ctl: ignoring unknown typed list argument: {other}"),
        }
    }
    list
}

fn count<T>(scan: &rt3_kit::typed::FamilyScan<T>) -> usize {
    scan.records.len()
}

fn failures(scan_failures: &[rt3_kit::typed::TypedError]) -> Vec<String> {
    scan_failures.iter().map(|e| e.to_string()).collect()
}

/// Runs `typed list`.
pub fn run(list: TypedList) -> ExitCode {
    let Some(root) = list
        .path
        .clone()
        .or_else(|| std::env::var_os("RT3_HOME").map(PathBuf::from))
    else {
        eprintln!("rt3ctl: typed list needs --path or RT3_HOME");
        return ExitCode::from(2);
    };
    if !root.is_dir() {
        eprintln!(
            "rt3ctl: installation root is not a directory: {}",
            root.display()
        );
        return ExitCode::from(2);
    }
    let data = root.join("Data");

    let cargo = scan_family(&data.join("CargoTypes"), "cty", CargoType::parse);
    let buildings = scan_family(&data.join("BuildingTypes"), "bty", BuildingType::parse);
    let behaviors = scan_family(&data.join("BuildingTypes"), "bca", BuildingBehavior::parse);
    let locos = scan_family(&data.join("EngineTypes"), "lco", Locomotive::parse);
    let cars = scan_family(&data.join("EngineTypes"), "car", CarType::parse);

    let tree_rows = fs::read_to_string(data.join("TreeTypes/TreeTypes.txt"))
        .ok()
        .and_then(|t| TextTable::parse(Path::new("TreeTypes.txt"), &t).ok())
        .map(|t| t.rows.len());
    let plane_rows = fs::read_to_string(data.join("Other/AirplaneTypes.txt"))
        .ok()
        .and_then(|t| TextTable::parse(Path::new("AirplaneTypes.txt"), &t).ok())
        .map(|t| t.rows.len());
    let language = fs::read(data.join("Language/RT3.lng"))
        .ok()
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .and_then(|t| LanguageFile::parse(Path::new("RT3.lng"), &t).ok())
        .map(|l| l.strings.len());
    let engine_cfg = fs::read(data.join("Configuration/engine.cfg"))
        .ok()
        .and_then(|b| EngineConfig::parse(Path::new("engine.cfg"), &b).ok());
    let game_cfg = fs::read(data.join("Configuration/game.cfg"))
        .ok()
        .and_then(|b| GameConfig::parse(Path::new("game.cfg"), &b).ok());

    let mut errors = Vec::new();
    errors.extend(failures(&cargo.failures));
    errors.extend(failures(&buildings.failures));
    errors.extend(failures(&behaviors.failures));
    errors.extend(failures(&locos.failures));
    errors.extend(failures(&cars.failures));
    for (name, missing) in [
        ("TreeTypes.txt", tree_rows.is_none()),
        ("AirplaneTypes.txt", plane_rows.is_none()),
        ("Language/RT3.lng", language.is_none()),
        ("Configuration/engine.cfg", engine_cfg.is_none()),
        ("Configuration/game.cfg", game_cfg.is_none()),
    ] {
        if missing {
            errors.push(format!("missing or unparsable: {name}"));
        }
    }
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("rt3ctl: {error}");
        }
        return ExitCode::from(1);
    }

    let subrecords: u32 = behaviors.records.iter().map(|(_, b)| b.count).sum();
    let scripts: usize = behaviors.records.iter().map(|(_, b)| b.scripts.len()).sum();

    if list.json {
        println!(
            "{{\n  \"cargo_types\": {{\"files\": {}}},\n  \"building_types\": {{\"files\": {}}},\n  \"building_behaviors\": {{\"files\": {}, \"subrecords\": {}, \"scripts\": {}}},\n  \"locomotives\": {{\"files\": {}}},\n  \"car_types\": {{\"files\": {}}},\n  \"text_tables\": {{\"tree_types_rows\": {}, \"airplane_types_rows\": {}}},\n  \"language\": {{\"strings\": {}}},\n  \"engine_cfg\": {{\"width\": {}, \"height\": {}}},\n  \"game_cfg\": {{\"leading_u32\": {}}}\n}}",
            count(&cargo),
            count(&buildings),
            count(&behaviors),
            subrecords,
            scripts,
            count(&locos),
            count(&cars),
            tree_rows.unwrap_or(0),
            plane_rows.unwrap_or(0),
            language.unwrap_or(0),
            engine_cfg.as_ref().map(|c| c.width).unwrap_or(0),
            engine_cfg.as_ref().map(|c| c.height).unwrap_or(0),
            game_cfg.as_ref().map(|c| c.leading).unwrap_or(0),
        );
    } else {
        println!("cargo types:          {}", count(&cargo));
        println!("building types:       {}", count(&buildings));
        println!(
            "building behaviors:   {} ({} subrecords, {} scripts)",
            count(&behaviors),
            subrecords,
            scripts
        );
        println!("locomotives:          {}", count(&locos));
        println!("car types:            {}", count(&cars));
        println!("tree types rows:      {}", tree_rows.unwrap_or(0));
        println!("airplane types rows:  {}", plane_rows.unwrap_or(0));
        println!("language strings:     {}", language.unwrap_or(0));
        if let Some(cfg) = &engine_cfg {
            println!("engine.cfg:           {}x{}", cfg.width, cfg.height);
        }
        println!(
            "game.cfg leading u32:  {}",
            game_cfg.map(|c| c.leading).unwrap_or(0)
        );
    }
    ExitCode::SUCCESS
}
