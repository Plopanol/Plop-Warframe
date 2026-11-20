//! Entry point: the window without a command, otherwise the import command line.

// Release builds open no console beside the window (Q-003); the command line then shows no
// output, so it is meant for debug builds.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use plop_warframe::db;
use plop_warframe::ui_bridge;
use plop_warframe::update::{self, DataPaths, Origin};

const USAGE: &str = "Usage : plop-warframe [--data-dir <dossier>] [commande]
Sans commande, ouvre la fenêtre. Commandes :
  --import        télécharge les modules du wiki, met à jour le cache puis importe
  --import-cache  importe depuis le cache, sans réseau
  --stats         affiche l'état de la base";

enum Command {
    Import(Origin),
    Stats,
}

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log::error!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let (data_dir, command) = parse_args(std::env::args().skip(1))?;
    let paths = match data_dir {
        Some(dir) => DataPaths::new(dir),
        None => {
            DataPaths::default_location().context("dossier de données par défaut introuvable")?
        }
    };
    log::info!("Dossier de données : {}", paths.dir.display());

    let Some(command) = command else {
        return ui_bridge::run(paths, now).context("fenêtre de l'application");
    };
    match command {
        Command::Import(origin) => {
            let report = update::run_import(&paths, origin, &now, &mut |progress| {
                log::info!("{}", progress.message)
            })?;
            for warning in &report.warnings {
                log::warn!("{warning}");
            }
            let summary = report.summary;
            log::info!(
                "Import terminé : {} nœuds, {} tables, {} objets, {} drops, {} reliques, {} avertissements",
                summary.nodes,
                summary.tables,
                summary.items,
                summary.drops,
                summary.relics,
                summary.warnings
            );
        }
        Command::Stats => print_stats(&paths)?,
    }
    Ok(())
}

fn parse_args(
    mut args: impl Iterator<Item = String>,
) -> Result<(Option<PathBuf>, Option<Command>)> {
    let mut data_dir = None;
    let mut command = None;
    while let Some(arg) = args.next() {
        let next = match arg.as_str() {
            "--data-dir" => {
                let dir = args.next().context("--data-dir attend un dossier")?;
                data_dir = Some(PathBuf::from(dir));
                continue;
            }
            "--import" => Command::Import(Origin::Wiki),
            "--import-cache" => Command::Import(Origin::Cache),
            "--stats" => Command::Stats,
            other => bail!("option inconnue « {other} »\n{USAGE}"),
        };
        if command.replace(next).is_some() {
            bail!("une seule commande à la fois\n{USAGE}");
        }
    }
    Ok((data_dir, command))
}

fn print_stats(paths: &DataPaths) -> Result<()> {
    // Reading stats must not create an empty database as a side effect.
    if !paths.database.is_file() {
        println!(
            "Aucune base dans « {} » : lancez --import.",
            paths.dir.display()
        );
        return Ok(());
    }
    let conn = db::open(&paths.database)?;
    let stats = db::stats(&conn)?;
    println!("Base : {}", paths.database.display());
    println!(
        "Nœuds : {}, objets : {}, drops : {}",
        stats.nodes, stats.items, stats.drops
    );
    match stats.last_run {
        Some(run) => println!(
            "Dernier import : {} ({}, {}) — {}",
            update::format_utc(run.started_at),
            run.status,
            run.source,
            run.message.unwrap_or_default()
        ),
        None => println!("Aucun import"),
    }
    Ok(())
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_secs()).ok())
        .unwrap_or(0)
}
