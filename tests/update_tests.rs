//! Cache and offline import tests (no network).

use plop_warframe::db;
use plop_warframe::source::{CacheError, CachedModule, ModuleCache};
use plop_warframe::update::{
    self, DE_CACHE_FILE, DataPaths, MODULES, Origin, UpdateError, WFCD_CACHE_DIR, WFCD_CACHE_FILE,
};

#[cfg(test)]
fn clock() -> i64 {
    1_763_500_000
}

#[test]
fn cache_round_trip_keeps_metadata() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cache = ModuleCache::new(dir.path().join("cache"));
    let module = CachedModule {
        text: "return {}".to_owned(),
        downloaded_at: 1_763_500_000,
        last_modified: Some("Wed, 19 Nov 2026 10:00:00 GMT".to_owned()),
        version: Some("1.1276.51".to_owned()),
    };
    cache.store("m.lua", &module).expect("store");
    assert_eq!(cache.load("m.lua").expect("load"), module);
    assert!(!cache.dir().join("m.lua.part").exists());

    // `version=` is optional: a `.meta` written before it still loads.
    std::fs::write(
        cache.dir().join("m.lua.meta"),
        "downloaded_at=1763500000
",
    )
    .expect("meta");
    assert_eq!(cache.load("m.lua").expect("load").version, None);
}

#[test]
fn missing_cache_is_reported() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cache = ModuleCache::new(dir.path());
    assert!(matches!(
        cache.load("absent.lua"),
        Err(CacheError::Missing(_))
    ));

    let paths = DataPaths::new(dir.path().join("data"));
    let result = update::run_import(&paths, Origin::Cache, &clock, &mut |_| {});
    assert!(matches!(
        result,
        Err(UpdateError::Cache(CacheError::Missing(_)))
    ));
}

#[test]
fn import_from_cache_fills_the_database() {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = DataPaths::new(dir.path().join("data"));
    let cache = ModuleCache::new(&paths.cache);
    let fixtures = [
        include_str!("fixtures/droptables_data.lua"),
        include_str!("fixtures/missions_data.lua"),
    ];
    for ((_, file), text) in MODULES.iter().zip(fixtures) {
        cache
            .store(
                file,
                &CachedModule {
                    text: text.to_owned(),
                    downloaded_at: clock(),
                    last_modified: None,
                    version: None,
                },
            )
            .expect("store");
    }

    let mut steps = Vec::new();
    let report = update::run_import(&paths, Origin::Cache, &clock, &mut |progress| {
        steps.push((progress.done, progress.total, progress.message.to_owned()))
    })
    .expect("import");
    assert_eq!(report.summary.nodes, 523);
    assert_eq!(report.warnings.len(), report.summary.warnings);
    let total = update::step_count(Origin::Cache);
    assert_eq!(steps.len(), total);
    assert!(
        steps
            .iter()
            .enumerate()
            .all(|(index, (done, steps_total, _))| *done == index && *steps_total == total)
    );

    let conn = db::open(&paths.database).expect("open");
    let run = db::stats(&conn).expect("stats").last_run.expect("run");
    assert_eq!(run.source, "wiki-cache");
    assert_eq!(
        run.source_revision.as_deref(),
        Some(
            "Module:DropTables/data : 2025-11-18 21:06:40 UTC ; \
             Module:Missions/data : 2025-11-18 21:06:40 UTC"
        )
    );
}

#[test]
fn utc_formatting() {
    assert_eq!(update::format_utc(0), "1970-01-01 00:00:00 UTC");
    assert_eq!(update::format_utc(951_782_400), "2000-02-29 00:00:00 UTC");
    assert_eq!(update::format_utc(1_763_500_000), "2025-11-18 21:06:40 UTC");
}

#[test]
fn progress_fraction_is_bounded() {
    let at = |done, total| {
        update::Progress {
            done,
            total,
            message: "",
        }
        .fraction()
    };
    assert_eq!(at(0, 4), 0.0);
    assert_eq!(at(2, 4), 0.5);
    assert_eq!(at(5, 4), 1.0);
    assert_eq!(at(0, 0), 1.0);
    assert_eq!(
        update::step_count(Origin::Wiki),
        update::MODULES.len()
            + update::enemy_modules().len()
            + 4
            + plop_warframe::source::wfcd_item_files().len()
    );
}

#[cfg(test)]
fn store(cache: &ModuleCache, file: &str, text: &str, last_modified: Option<&str>) {
    cache
        .store(
            file,
            &CachedModule {
                text: text.to_owned(),
                downloaded_at: clock(),
                last_modified: last_modified.map(str::to_owned),
                version: None,
            },
        )
        .expect("store");
}

#[test]
fn relics_are_imported_from_the_cached_de_page() {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = DataPaths::new(dir.path().join("data"));
    let cache = ModuleCache::new(&paths.cache);
    let fixtures = [
        include_str!("fixtures/droptables_data.lua"),
        include_str!("fixtures/missions_data.lua"),
    ];
    for ((_, file), text) in MODULES.iter().zip(fixtures) {
        store(&cache, file, text, None);
    }

    // Without the DE page: no relic, one warning.
    let report = update::run_import(&paths, Origin::Cache, &clock, &mut |_| {}).expect("import");
    assert_eq!(report.summary.relics, 0);
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.contains("aucune relique"))
    );

    store(
        &cache,
        DE_CACHE_FILE,
        include_str!("fixtures/droptables_de_relics.html"),
        Some("Thu, 22 Oct 2026 19:04:13 GMT"),
    );
    let report = update::run_import(&paths, Origin::Cache, &clock, &mut |_| {}).expect("import");
    assert_eq!(report.summary.relics, 8);
    assert!(
        !report
            .warnings
            .iter()
            .any(|warning| warning.contains("aucune relique"))
    );
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.contains("statut vaulted inconnu"))
    );

    store(
        &cache,
        WFCD_CACHE_FILE,
        include_str!("fixtures/wfcd_relics.json"),
        None,
    );
    let report = update::run_import(&paths, Origin::Cache, &clock, &mut |_| {}).expect("import");
    assert!(
        !report
            .warnings
            .iter()
            .any(|warning| warning.contains("statut vaulted inconnu"))
    );
    let conn = db::open(&paths.database).expect("open");
    let vaulted: Option<bool> = conn
        .query_row(
            "SELECT vaulted FROM relic WHERE name = 'Lith Q3 Relic'",
            [],
            |row| row.get(0),
        )
        .expect("vaulted");
    assert_eq!(vaulted, Some(true));
    let rewards: i64 = conn
        .query_row("SELECT count(*) FROM relic_reward", [], |row| row.get(0))
        .expect("count");
    assert_eq!(rewards, 79);
    let run = db::stats(&conn).expect("stats").last_run.expect("run");
    assert!(run.source_revision.as_deref().is_some_and(|revision| {
        revision.contains(" ; tables officielles DE : Thu, 22 Oct 2026 19:04:13 GMT ; ")
    }));
}

#[test]
fn wfcd_items_are_imported_from_their_cache_directory() {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = DataPaths::new(dir.path().join("data"));
    let cache = ModuleCache::new(&paths.cache);
    let fixtures = [
        include_str!("fixtures/droptables_data.lua"),
        include_str!("fixtures/missions_data.lua"),
    ];
    for ((_, file), text) in MODULES.iter().zip(fixtures) {
        store(&cache, file, text, None);
    }
    store(
        &cache,
        DE_CACHE_FILE,
        include_str!("fixtures/droptables_de_relics.html"),
        None,
    );
    // The former relic list must be ignored once `wfcd/Relics.json` exists.
    store(&cache, WFCD_CACHE_FILE, "[]", None);
    let wfcd = ModuleCache::new(paths.cache.join(WFCD_CACHE_DIR));
    store(
        &wfcd,
        "Warframes.json",
        include_str!("fixtures/wfcd/Warframes.json"),
        None,
    );
    store(
        &wfcd,
        "Mods.json",
        include_str!("fixtures/wfcd/Mods.json"),
        None,
    );
    store(
        &wfcd,
        "fr.json",
        include_str!("fixtures/wfcd/fr.json"),
        None,
    );
    wfcd.store(
        "Relics.json",
        &CachedModule {
            text: include_str!("fixtures/wfcd_relics.json").to_owned(),
            downloaded_at: clock(),
            last_modified: None,
            version: Some("1.1276.51".to_owned()),
        },
    )
    .expect("store");

    let report = update::run_import(&paths, Origin::Cache, &clock, &mut |_| {}).expect("import");
    assert_eq!(
        report.summary.wfcd_items_by_category,
        [
            ("Mods".to_owned(), 1),
            ("Relics".to_owned(), 33),
            ("Warframes".to_owned(), 2),
        ]
    );
    assert_eq!(report.summary.wfcd_items(), 36);
    // 27 files, 4 in the cache: the 23 missing ones share one warning.
    let missing: Vec<_> = report
        .warnings
        .iter()
        .filter(|warning| warning.starts_with("Fichiers WFCD absents"))
        .collect();
    assert_eq!(missing.len(), 1);
    assert!(missing[0].contains("Arcanes.json") && !missing[0].contains("Mods.json"));
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.contains("Mods.json : entrée n° 2"))
    );
    assert!(
        !report
            .warnings
            .iter()
            .any(|warning| warning.contains("statut vaulted inconnu"))
    );

    let conn = db::open(&paths.database).expect("open");
    let vaulted: Option<bool> = conn
        .query_row(
            "SELECT vaulted FROM relic WHERE name = 'Lith Q3 Relic'",
            [],
            |row| row.get(0),
        )
        .expect("vaulted");
    assert_eq!(vaulted, Some(true));
    let name_fr: Option<String> = conn
        .query_row(
            "SELECT name_fr FROM wfcd_item WHERE name = 'Vitality'",
            [],
            |row| row.get(0),
        )
        .expect("name_fr");
    assert_eq!(name_fr.as_deref(), Some("Vitalité"));
    let run = db::stats(&conn).expect("stats").last_run.expect("run");
    assert!(
        run.source_revision.as_deref().is_some_and(
            |revision| revision.ends_with(" ; WFCD 1.1276.51 : 2025-11-18 21:06:40 UTC")
        )
    );
}
