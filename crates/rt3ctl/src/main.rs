//! `rt3ctl` — RT3Engine command line.
//!
//! Subcommands:
//!
//! - `inventory [--path DIR] [--json]` — a metadata-only inventory of a
//!   Railroad Tycoon 3 installation (file counts and byte sizes, never file
//!   contents). It backs the `docs/evidence/install-inventory-v1.json`
//!   artifact and is the milestone-0 bootstrap tool.
//! - `pk4 list [--path DIR] [--archive FILE]... [--json]` — parse PK4 asset
//!   archives and summarize them; with `--archive` prints that archive's
//!   entries instead of the aggregate summary. Backs
//!   `docs/evidence/pk4-archives-v1.json`.
//!
//! The installation root resolves from `--path`, else `RT3_HOME`. The
//! installation is treated read-only.

mod company_cmd;
mod inventory;
mod map_cmd;
mod orders_cmd;
mod pk4_cmd;
mod save_cmd;
mod save_diff;
mod sim_cmd;
mod typed_cmd;

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "rt3ctl — RT3Engine command line

Usage:
  rt3ctl inventory [--path <install-root>] [--json]
  rt3ctl pk4 list [--path <install-root>] [--archive <file>]... [--json]
  rt3ctl typed list [--path <install-root>] [--json]
  rt3ctl map list [--path <install-root>] [--json]
  rt3ctl map terrain <name> [--path <install-root>] [--png <output.png>]
  rt3ctl sim inspect <map> [--path <install-root>] [--json]
  rt3ctl sim run <map> [--seed <n>] [--months <n>] [--ticks-per-month <n>] [--dump <file>] [--path <install-root>]
  rt3ctl sim compare <map> --save <file> [--path <install-root>] [--json]
  rt3ctl sim init --checkpoint <file> --log <file> [--station <slot>:<label>]... [--stations-from <save>]
  rt3ctl sim build-track --checkpoint <file> --log <file> --from <slot> --to <slot> --length <units>
  rt3ctl sim buy-train --checkpoint <file> --log <file> --locomotive <name> [--car <name>]...
  rt3ctl sim assign-schedule --checkpoint <file> --log <file> [--dwell <ticks>]
  rt3ctl sim replay --checkpoint <file> --log <file> [--json]
  rt3ctl sim company --track-save <file> [--path <install-root>] [--trips <n>] [--dwell <ticks>] [--competitor-start <tick>] [--competitor-interval <ticks>] [--json]
  rt3ctl save inspect --path <save-file> [--json]
  rt3ctl save diff --path-a <first-save> --path-b <second-save> [--json]
  rt3ctl --help

Environment:
  RT3_HOME   default installation root (see docs/environment.md)
";

enum Command {
    Inventory { path: Option<PathBuf>, json: bool },
    Pk4List(pk4_cmd::Pk4List),
    TypedList(typed_cmd::TypedList),
    MapList(map_cmd::MapList),
    MapTerrain(map_cmd::MapTerrain),
    SimInspect(sim_cmd::SimInspect),
    SimRun(sim_cmd::SimRun),
    SimCompare(sim_cmd::SimCompare),
    SimInit(orders_cmd::Init),
    SimOrder(orders_cmd::OrderCommand),
    SimReplay(orders_cmd::Replay),
    SimCompany(company_cmd::CompanyRun),
    SaveInspect(save_cmd::SaveInspect),
    SaveDiff(save_diff::SaveDiff),
    Help,
}

fn parse_args(args: &[String]) -> Result<Command, String> {
    let mut it = args.iter();
    let name = match it.next() {
        Some(n) => n.as_str(),
        None => return Ok(Command::Help),
    };
    match name {
        "inventory" => {
            let mut path = None;
            let mut json = false;
            let mut rest = it;
            while let Some(arg) = rest.next() {
                match arg.as_str() {
                    "--json" => json = true,
                    "--path" => {
                        let value = rest
                            .next()
                            .ok_or_else(|| "--path needs a value".to_string())?;
                        path = Some(PathBuf::from(value));
                    }
                    other => return Err(format!("unknown argument for inventory: {other}")),
                }
            }
            Ok(Command::Inventory { path, json })
        }
        "pk4" => match it.next().map(String::as_str) {
            Some("list") => Ok(Command::Pk4List(pk4_cmd::parse(it))),
            Some("--help") | Some("help") | None => Err("pk4 needs a subcommand: list".to_string()),
            Some(other) => Err(format!("unknown pk4 subcommand: {other}")),
        },
        "map" => match it.next().map(String::as_str) {
            Some("list") => Ok(Command::MapList(map_cmd::parse(it))),
            Some("terrain") => {
                let mut path = None;
                let mut name: Option<String> = None;
                let mut all = false;
                let mut json = false;
                let mut png = None;
                while let Some(arg) = it.next() {
                    match arg.as_str() {
                        "--path" => {
                            if let Some(value) = it.next() {
                                path = Some(PathBuf::from(value));
                            } else {
                                return Err("map terrain: --path needs a value".to_string());
                            }
                        }
                        "--all" => all = true,
                        "--json" => json = true,
                        "--png" => {
                            if let Some(value) = it.next() {
                                if value.starts_with("--") {
                                    return Err(
                                        "map terrain: --png needs a value before another option"
                                            .to_string(),
                                    );
                                }
                                png = Some(PathBuf::from(value));
                            } else {
                                return Err("map terrain: --png needs a value".to_string());
                            }
                        }
                        other if other.starts_with("--") => {
                            return Err(format!("map terrain: unknown option {other}"));
                        }
                        other => {
                            if name.is_some() {
                                return Err(format!(
                                    "map terrain: unexpected extra argument {other}"
                                ));
                            }
                            name = Some(other.to_string());
                        }
                    }
                }
                if name.is_some() && all {
                    return Err("map terrain: --all takes no map name".to_string());
                }
                if all && png.is_some() {
                    return Err("map terrain: --png needs one map name, not --all".to_string());
                }
                if all || name.is_some() {
                    Ok(Command::MapTerrain(map_cmd::MapTerrain {
                        path,
                        name,
                        all,
                        json,
                        png,
                    }))
                } else {
                    Err("map terrain needs a map name or --all".to_string())
                }
            }
            Some("--help") | Some("help") | None => {
                Err("map needs a subcommand: list, terrain <name>".to_string())
            }
            Some(other) => Err(format!("unknown map subcommand: {other}")),
        },
        "sim" => match it.next().map(String::as_str) {
            Some("inspect") => Ok(Command::SimInspect(sim_cmd::parse(it)?)),
            Some("run") => Ok(Command::SimRun(sim_cmd::parse_run(it)?)),
            Some("compare") => Ok(Command::SimCompare(sim_cmd::parse_compare(it)?)),
            Some("init") => Ok(Command::SimInit(orders_cmd::parse_init(it)?)),
            Some("build-track") => Ok(Command::SimOrder(orders_cmd::parse_build_track(it)?)),
            Some("buy-train") => Ok(Command::SimOrder(orders_cmd::parse_buy_train(it)?)),
            Some("assign-schedule") => {
                Ok(Command::SimOrder(orders_cmd::parse_assign_schedule(it)?))
            }
            Some("replay") => Ok(Command::SimReplay(orders_cmd::parse_replay(it)?)),
            Some("company") => Ok(Command::SimCompany(company_cmd::parse(it)?)),
            Some("--help") | Some("help") | None => Err(
                "sim needs a subcommand: inspect, run, compare, init, build-track, buy-train, \
                 assign-schedule, replay, company"
                    .to_string(),
            ),
            Some(other) => Err(format!("unknown sim subcommand: {other}")),
        },
        "save" => {
            match it.next().map(String::as_str) {
                Some("inspect") => {
                    let mut path = None;
                    let mut json = false;
                    while let Some(arg) = it.next() {
                        match arg.as_str() {
                            "--path" => {
                                let value = it.next().filter(|v| !v.starts_with("--")).ok_or_else(
                                    || "save inspect: --path needs a value".to_string(),
                                )?;
                                path = Some(PathBuf::from(value));
                            }
                            "--json" => json = true,
                            other => return Err(format!("save inspect: unknown option {other}")),
                        }
                    }
                    let path =
                        path.ok_or_else(|| "save inspect needs --path <save-file>".to_string())?;
                    Ok(Command::SaveInspect(save_cmd::SaveInspect { path, json }))
                }
                Some("diff") => {
                    let mut left = None;
                    let mut right = None;
                    let mut json = false;
                    while let Some(arg) = it.next() {
                        match arg.as_str() {
                            "--path-a" => {
                                let value = it.next().filter(|v| !v.starts_with("--")).ok_or_else(
                                    || "save diff: --path-a needs a value".to_string(),
                                )?;
                                left = Some(PathBuf::from(value));
                            }
                            "--path-b" => {
                                let value = it.next().filter(|v| !v.starts_with("--")).ok_or_else(
                                    || "save diff: --path-b needs a value".to_string(),
                                )?;
                                right = Some(PathBuf::from(value));
                            }
                            "--json" => json = true,
                            other => return Err(format!("save diff: unknown option {other}")),
                        }
                    }
                    let left =
                        left.ok_or_else(|| "save diff needs --path-a <save-file>".to_string())?;
                    let right =
                        right.ok_or_else(|| "save diff needs --path-b <save-file>".to_string())?;
                    Ok(Command::SaveDiff(save_diff::SaveDiff { left, right, json }))
                }
                Some("--help") | Some("help") | None => {
                    Err("save needs a subcommand: inspect, diff".to_string())
                }
                Some(other) => Err(format!("unknown save subcommand: {other}")),
            }
        }
        "typed" => match it.next().map(String::as_str) {
            Some("list") => Ok(Command::TypedList(typed_cmd::parse(it))),
            Some("--help") | Some("help") | None => {
                Err("typed needs a subcommand: list".to_string())
            }
            Some(other) => Err(format!("unknown typed subcommand: {other}")),
        },
        "--help" | "help" => Ok(Command::Help),
        other => Err(format!("unknown command: {other}")),
    }
}

/// Resolves an install-backed fixture named by an environment variable. A
/// relative value is always taken from the repository root, as the evidence
/// docs give it, never from the crate directory the tests run in.
#[cfg(test)]
pub(crate) fn fixture_path(name: &str) -> Option<PathBuf> {
    let path = PathBuf::from(std::env::var_os(name)?);
    if path.is_absolute() {
        return Some(path);
    }
    Some(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path),
    )
}

pub(crate) fn install_root(explicit: Option<PathBuf>) -> Option<PathBuf> {
    explicit.or_else(|| std::env::var_os("RT3_HOME").map(PathBuf::from))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = match parse_args(&args) {
        Ok(command) => command,
        Err(message) => {
            eprintln!("rt3ctl: {message}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match command {
        Command::Help => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Command::Inventory { path, json } => {
            let Some(root) = install_root(path) else {
                eprintln!("rt3ctl: no installation root: pass --path or set RT3_HOME\n\n{USAGE}");
                return ExitCode::from(2);
            };
            match inventory::inventory(&root) {
                Ok(report) => {
                    if json {
                        println!("{}", report.to_json());
                    } else {
                        report.print(&mut std::io::stdout());
                    }
                    ExitCode::SUCCESS
                }
                Err(message) => {
                    eprintln!("rt3ctl: inventory of {}: {message}", root.display());
                    ExitCode::from(1)
                }
            }
        }
        Command::Pk4List(list) => pk4_cmd::run(list),
        Command::TypedList(list) => typed_cmd::run(list),
        Command::MapList(list) => map_cmd::run(list),
        Command::MapTerrain(t) => map_cmd::run_terrain(t),
        Command::SimInspect(i) => sim_cmd::run(i),
        Command::SimRun(r) => sim_cmd::run_simulation(r),
        Command::SimCompare(c) => sim_cmd::run_compare(c),
        Command::SimInit(i) => orders_cmd::run_init(i),
        Command::SimOrder(o) => orders_cmd::run_order(o),
        Command::SimReplay(r) => orders_cmd::run_replay(r),
        Command::SimCompany(c) => company_cmd::run(c),
        Command::SaveInspect(s) => save_cmd::run(s),
        Command::SaveDiff(s) => save_diff::run(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_diff_parses_two_paths() {
        let args = [
            "save".to_string(),
            "diff".to_string(),
            "--path-a".to_string(),
            "first.gms".to_string(),
            "--path-b".to_string(),
            "second.gms".to_string(),
            "--json".to_string(),
        ];
        let Ok(Command::SaveDiff(diff)) = parse_args(&args) else {
            panic!("save diff arguments must parse");
        };
        assert_eq!(diff.left, PathBuf::from("first.gms"));
        assert_eq!(diff.right, PathBuf::from("second.gms"));
        assert!(diff.json);
    }

    #[test]
    fn png_option_does_not_consume_another_option_as_filename() {
        let args = [
            "map".to_string(),
            "terrain".to_string(),
            "California Island".to_string(),
            "--png".to_string(),
            "--all".to_string(),
        ];
        assert!(parse_args(&args).is_err());
        let explicit = [
            "map".to_string(),
            "terrain".to_string(),
            "California Island".to_string(),
            "--png".to_string(),
            "./--all".to_string(),
        ];
        let Ok(Command::MapTerrain(t)) = parse_args(&explicit) else {
            panic!("explicit option-looking filename must be accepted");
        };
        assert_eq!(t.png, Some(PathBuf::from("./--all")));
    }
}
