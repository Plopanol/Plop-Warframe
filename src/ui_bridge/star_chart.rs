//! Star Chart tab controller. The query runs on the UI thread at each filter change, like the
//! other tabs: a few hundred nodes at most; the read connection is closed during an import.

use std::cell::RefCell;
use std::rc::Rc;

use rusqlite::Connection;

use super::AppWindow;
use super::search::{columns, rows, strings, with_state};
use super::spot::{index_of, label_at};
use crate::db::{self, DbError, StarChartChoices, StarChartFilter};
use crate::search_view::{
    MissionTypeChoice, ResultTable, mission_type_choices, node_filter_from_index,
};
use crate::spot_view::{PlanetChoice, planet_choices};
use crate::star_chart_view::{
    FactionChoice, empty_nodes, faction_choices, node_table, parse_level, summary,
};
use crate::update::DataPaths;

pub struct StarChartState {
    paths: DataPaths,
    conn: Option<Connection>,
    mission_types: Vec<MissionTypeChoice>,
    planets: Vec<PlanetChoice>,
    factions: Vec<FactionChoice>,
    table: ResultTable,
}

pub type SharedStarChart = Rc<RefCell<StarChartState>>;

/// Wires the Star Chart tab callbacks and loads the database; returns log lines for a read
/// error.
pub fn install(window: &AppWindow, paths: DataPaths) -> (SharedStarChart, Vec<String>) {
    let state = Rc::new(RefCell::new(StarChartState {
        paths,
        conn: None,
        mission_types: Vec::new(),
        planets: Vec::new(),
        factions: Vec::new(),
        table: empty_nodes(),
    }));

    let filters_changed = with_state(window, &state, |window, state: &mut StarChartState, ()| {
        state.run_query(window);
    });
    window.on_star_chart_filters_changed(move || filters_changed(()));
    window.on_star_chart_sort_ascending(with_state(
        window,
        &state,
        |window, state: &mut StarChartState, column| {
            state.sort(window, column, false);
        },
    ));
    window.on_star_chart_sort_descending(with_state(
        window,
        &state,
        |window, state: &mut StarChartState, column| {
            state.sort(window, column, true);
        },
    ));

    let lines = state.borrow_mut().reload(window);
    (state, lines)
}

impl StarChartState {
    /// Closes the read connection before an import writes the database.
    pub fn close(&mut self) {
        self.conn = None;
    }

    /// Reopens the database after an import, keeping the chosen filters that still exist.
    pub fn reload(&mut self, window: &AppWindow) -> Vec<String> {
        let mut lines = Vec::new();
        let previous_type = label_at(
            &self.mission_types,
            window.get_star_chart_mission_type_index(),
            |c| c.id.map(|_| c.label.clone()),
        );
        let previous_planet = label_at(&self.planets, window.get_star_chart_planet_index(), |c| {
            c.id.map(|_| c.label.clone())
        });
        let previous_faction =
            label_at(&self.factions, window.get_star_chart_faction_index(), |c| {
                c.code.as_ref().map(|_| c.label.clone())
            });

        self.conn = match self.open() {
            Ok(conn) => conn,
            Err(error) => {
                log::error!("Carte stellaire, ouverture de la base : {error}");
                lines.push(format!("Onglet Carte stellaire indisponible : {error}"));
                None
            }
        };
        let choices = match self.conn.as_ref().map(db::star_chart_choices) {
            Some(Ok(choices)) => choices,
            Some(Err(error)) => {
                log::error!("Carte stellaire, lecture de la base : {error}");
                lines.push(format!("Onglet Carte stellaire indisponible : {error}"));
                StarChartChoices::default()
            }
            None => StarChartChoices::default(),
        };
        let has_data = !choices.planets.is_empty();
        self.mission_types = mission_type_choices(&choices.mission_types);
        self.planets = planet_choices(&choices.planets);
        self.factions = faction_choices(&choices.factions);

        window.set_star_chart_has_data(has_data);
        window.set_star_chart_mission_types(strings(
            self.mission_types.iter().map(|c| c.label.as_str()),
        ));
        window.set_star_chart_mission_type_index(index_of(
            &self.mission_types,
            previous_type,
            |c| &c.label,
        ));
        window.set_star_chart_planets(strings(self.planets.iter().map(|c| c.label.as_str())));
        window.set_star_chart_planet_index(index_of(&self.planets, previous_planet, |c| &c.label));
        window.set_star_chart_factions(strings(self.factions.iter().map(|c| c.label.as_str())));
        window
            .set_star_chart_faction_index(index_of(&self.factions, previous_faction, |c| &c.label));
        self.run_query(window);
        lines
    }

    /// The database is not created when absent.
    fn open(&self) -> Result<Option<Connection>, DbError> {
        if self.paths.database.is_file() {
            db::open(&self.paths.database).map(Some)
        } else {
            Ok(None)
        }
    }

    fn run_query(&mut self, window: &AppWindow) {
        let text = match self.filter(window) {
            Err(message) => {
                self.table = empty_nodes();
                message
            }
            Ok(filter) => {
                let found = match &self.conn {
                    Some(conn) => db::star_chart_nodes(conn, &filter),
                    None => Ok(Vec::new()),
                };
                match found {
                    Ok(nodes) => {
                        self.table = node_table(&nodes);
                        summary(!self.planets.is_empty(), nodes.len())
                    }
                    Err(error) => {
                        log::error!("Carte stellaire : {error}");
                        self.table = empty_nodes();
                        format!("Recherche impossible : {error}")
                    }
                }
            }
        };
        window.set_star_chart_summary(text.into());
        window.set_star_chart_columns(columns(&self.table));
        window.set_star_chart_rows(rows(&self.table));
    }

    /// Filter from the controls; an invalid level gives the message to show.
    fn filter(&self, window: &AppWindow) -> Result<StarChartFilter, String> {
        let pick = |index: i32| usize::try_from(index).ok();
        let min_level =
            parse_level(window.get_star_chart_min_level().as_str()).map_err(|e| e.to_string())?;
        let max_level =
            parse_level(window.get_star_chart_max_level().as_str()).map_err(|e| e.to_string())?;
        Ok(StarChartFilter {
            mission_type_id: pick(window.get_star_chart_mission_type_index())
                .and_then(|i| self.mission_types.get(i))
                .and_then(|c| c.id),
            planet_id: pick(window.get_star_chart_planet_index())
                .and_then(|i| self.planets.get(i))
                .and_then(|c| c.id),
            faction: pick(window.get_star_chart_faction_index())
                .and_then(|i| self.factions.get(i))
                .and_then(|c| c.code.clone()),
            min_level,
            max_level,
            dark_sectors: node_filter_from_index(window.get_star_chart_dark_sectors_index()),
        })
    }

    fn sort(&mut self, window: &AppWindow, column: i32, descending: bool) {
        if let Ok(column) = usize::try_from(column) {
            self.table.sort(column, descending);
            window.set_star_chart_rows(rows(&self.table));
        }
    }
}
