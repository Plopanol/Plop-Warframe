//! Temporary entry point: parses two module files and prints a summary, until the database
//! (TASK-004) and the UI (TASK-006) exist.

use std::process::ExitCode;

use plop_warframe::source::{DropSource, WikiSource};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [flag, drop_tables, missions] = args.as_slice() else {
        eprintln!("Usage : plop-warframe --parse <DropTables_data.lua> <Missions_data.lua>");
        return ExitCode::from(2);
    };
    if flag != "--parse" {
        eprintln!("Option inconnue : {flag}");
        return ExitCode::from(2);
    }
    match parse(drop_tables, missions) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("Erreur : {message}");
            ExitCode::FAILURE
        }
    }
}

fn parse(drop_tables: &str, missions: &str) -> Result<(), String> {
    let read = |path: &str| {
        std::fs::read_to_string(path).map_err(|error| format!("lecture de « {path} » : {error}"))
    };
    let dataset = WikiSource::new(read(drop_tables)?, read(missions)?)
        .load()
        .map_err(|error| error.to_string())?;
    let entries: usize = dataset.tables.iter().map(|table| table.entries.len()).sum();
    let specials = dataset.nodes.iter().filter(|node| node.is_special).count();
    println!("Types de mission : {}", dataset.mission_types.len());
    println!(
        "Nœuds : {} (dont {specials} missions spéciales)",
        dataset.nodes.len()
    );
    println!("Tables : {}, entrées : {entries}", dataset.tables.len());
    println!("Avertissements : {}", dataset.warnings.len());
    for warning in &dataset.warnings {
        println!("  - {warning}");
    }
    Ok(())
}
