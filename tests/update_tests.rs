//! Cache and offline import tests (no network).

use plop_warframe::db;
use plop_warframe::source::{CacheError, CachedModule, ModuleCache};
use plop_warframe::update::{self, DataPaths, MODULES, Origin, UpdateError};

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
    };
    cache.store("m.lua", &module).expect("store");
    assert_eq!(cache.load("m.lua").expect("load"), module);
    assert!(!cache.dir().join("m.lua.part").exists());
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
                },
            )
            .expect("store");
    }

    let mut steps = Vec::new();
    let report = update::run_import(&paths, Origin::Cache, &clock, &mut |progress| {
        steps.push((progress.done, progress.total, progress.message.to_owned()))
    })
    .expect("import");
    assert_eq!(report.summary.nodes, 524);
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
    assert_eq!(update::step_count(Origin::Wiki), update::MODULES.len() + 2);
}
