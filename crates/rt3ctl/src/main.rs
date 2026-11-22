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

mod inventory;
mod map_cmd;
mod pk4_cmd;
mod typed_cmd;

use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "rt3ctl — RT3Engine command line

Usage:
  rt3ctl inventory [--path <install-root>] [--json]
  rt3ctl pk4 list [--path <install-root>] [--archive <file>]... [--json]
  rt3ctl typed list [--path <install-root>] [--json]
  rt3ctl map list [--path <install-root>] [--json]
  rt3ctl --help

Environment:
  RT3_HOME   default installation root (see docs/environment.md)
";

enum Command {
    Inventory { path: Option<PathBuf>, json: bool },
    Pk4List(pk4_cmd::Pk4List),
    TypedList(typed_cmd::TypedList),
    MapList(map_cmd::MapList),
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
            Some("--help") | Some("help") | None => Err("map needs a subcommand: list".to_string()),
            Some(other) => Err(format!("unknown map subcommand: {other}")),
        },
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

fn install_root(explicit: Option<PathBuf>) -> Option<PathBuf> {
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
    }
}
