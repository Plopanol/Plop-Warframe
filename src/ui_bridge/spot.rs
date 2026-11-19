//! Spot Search tab controller. Queries run on the UI thread like the Mission Search tab
//! (tens of milliseconds, measured in TASK-011); the read connection is closed during an import.

use std::cell::RefCell;
use std::rc::Rc;

use rusqlite::Connection;
use slint::SharedString;

use super::AppWindow;
use super::search::{columns, resolve_by_name, rows, strings, with_state};
use crate::db::{self, DbError, ItemHit, SpotFilter};
use crate::search_view::{MissionTypeChoice, ResultTable, mission_type_choices, suggestion_label};
use crate::spot_view::{
    PlanetChoice, empty_enemies, empty_nodes, enemy_table, node_table, nodes_title, planet_choices,
    summary,
};
use crate::update::DataPaths;

const SUGGESTION_LIMIT: usize = 20;

/// Enemy picked in the enemy table, which restricts the node table.
#[derive(Debug, Clone)]
struct PickedEnemy {
    id: i64,
    name: String,
}

pub struct SpotState {
    paths: DataPaths,
    conn: Option<Connection>,
    mission_types: Vec<MissionTypeChoice>,
    planets: Vec<PlanetChoice>,
    suggestions: Vec<ItemHit>,
    selected: Vec<ItemHit>,
    enemies: ResultTable,
    nodes: ResultTable,
    picked: Option<PickedEnemy>,
}

pub type SharedSpot = Rc<RefCell<SpotState>>;

/// Wires the Spot Search tab callbacks and loads the database; returns log lines for a read
/// error.
pub fn install(window: &AppWindow, paths: DataPaths) -> (SharedSpot, Vec<String>) {
    let state = Rc::new(RefCell::new(SpotState {
        paths,
        conn: None,
        mission_types: Vec::new(),
        planets: Vec::new(),
        suggestions: Vec::new(),
        selected: Vec::new(),
        enemies: empty_enemies(),
        nodes: empty_nodes(),
        picked: None,
    }));

    window.on_spot_query_edited(with_state(
        window,
        &state,
        |window, state: &mut SpotState, text: SharedString| {
            state.update_suggestions(window, text.as_str());
        },
    ));
    window.on_spot_add_suggestion(with_state(
        window,
        &state,
        |window, state: &mut SpotState, index| {
            state.add_suggestion(window, index);
        },
    ));
    window.on_spot_remove_item(with_state(
        window,
        &state,
        |window, state: &mut SpotState, index| {
            if let Some(position) = usize::try_from(index)
                .ok()
                .filter(|&i| i < state.selected.len())
            {
                state.selected.remove(position);
                state.show_selection(window);
                state.run_search(window);
            }
        },
    ));
    let filters_changed = with_state(window, &state, |window, state: &mut SpotState, ()| {
        state.run_search(window);
    });
    window.on_spot_filters_changed(move || filters_changed(()));
    window.on_spot_enemy_selected(with_state(
        window,
        &state,
        |window, state: &mut SpotState, row| {
            state.pick_enemy(window, row);
        },
    ));
    let clear_enemy = with_state(window, &state, |window, state: &mut SpotState, ()| {
        state.picked = None;
        window.set_spot_enemy_current_row(-1);
        state.run_nodes(window);
    });
    window.on_spot_clear_enemy(move || clear_enemy(()));
    window.on_spot_enemy_sort_ascending(with_state(
        window,
        &state,
        |window, state: &mut SpotState, column| {
            state.sort_enemies(window, column, false);
        },
    ));
    window.on_spot_enemy_sort_descending(with_state(
        window,
        &state,
        |window, state: &mut SpotState, column| {
            state.sort_enemies(window, column, true);
        },
    ));
    window.on_spot_node_sort_ascending(with_state(
        window,
        &state,
        |window, state: &mut SpotState, column| {
            state.sort_nodes(window, column, false);
        },
    ));
    window.on_spot_node_sort_descending(with_state(
        window,
        &state,
        |window, state: &mut SpotState, column| {
            state.sort_nodes(window, column, true);
        },
    ));

    let lines = state.borrow_mut().reload(window);
    (state, lines)
}

impl SpotState {
    /// Closes the read connection before an import writes the database.
    pub fn close(&mut self) {
        self.conn = None;
    }

    /// Reopens the database after an import, keeping the filters and chosen items whose names
    /// still exist (an import renumbers ids).
    pub fn reload(&mut self, window: &AppWindow) -> Vec<String> {
        let mut lines = Vec::new();
        self.conn = match self.open() {
            Ok(conn) => conn,
            Err(error) => {
                log::error!("Recherche Spot, ouverture de la base : {error}");
                lines.push(format!("Recherche Spot indisponible : {error}"));
                None
            }
        };
        let previous_type = label_at(
            &self.mission_types,
            window.get_spot_mission_type_index(),
            |c| c.id.map(|_| c.label.clone()),
        );
        let previous_planet = label_at(&self.planets, window.get_spot_planet_index(), |c| {
            c.id.map(|_| c.label.clone())
        });

        let (has_data, types, planets, selected) = match self.conn.as_ref().map(|conn| {
            Ok::<_, DbError>((
                db::stats(conn)?.items > 0,
                db::mission_types(conn)?,
                db::planets(conn)?,
                resolve_by_name(conn, &self.selected)?,
            ))
        }) {
            Some(Ok(loaded)) => loaded,
            Some(Err(error)) => {
                log::error!("Recherche Spot, lecture de la base : {error}");
                lines.push(format!("Recherche Spot indisponible : {error}"));
                (false, Vec::new(), Vec::new(), Vec::new())
            }
            None => (false, Vec::new(), Vec::new(), Vec::new()),
        };
        self.mission_types = mission_type_choices(&types);
        self.planets = planet_choices(&planets);
        self.selected = selected;
        self.suggestions.clear();

        let type_index = index_of(&self.mission_types, previous_type, |c| &c.label);
        let planet_index = index_of(&self.planets, previous_planet, |c| &c.label);
        window.set_spot_has_data(has_data);
        window.set_spot_mission_types(strings(self.mission_types.iter().map(|c| c.label.as_str())));
        window.set_spot_mission_type_index(type_index);
        window.set_spot_planets(strings(self.planets.iter().map(|c| c.label.as_str())));
        window.set_spot_planet_index(planet_index);
        window.set_spot_query(SharedString::new());
        window.set_spot_suggestions(strings(std::iter::empty::<&str>()));
        self.show_selection(window);
        self.run_search(window);
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

    fn update_suggestions(&mut self, window: &AppWindow, text: &str) {
        self.suggestions = match &self.conn {
            Some(conn) => {
                let limit = SUGGESTION_LIMIT + self.selected.len();
                match db::search_items(conn, text, u32::try_from(limit).unwrap_or(u32::MAX)) {
                    Ok(hits) => hits
                        .into_iter()
                        .filter(|hit| !self.selected.iter().any(|s| s.id == hit.id))
                        .take(SUGGESTION_LIMIT)
                        .collect(),
                    Err(error) => {
                        log::error!("Autocomplétion (spot) : {error}");
                        Vec::new()
                    }
                }
            }
            None => Vec::new(),
        };
        let labels: Vec<String> = self.suggestions.iter().map(suggestion_label).collect();
        window.set_spot_suggestions(strings(labels.iter().map(String::as_str)));
    }

    fn add_suggestion(&mut self, window: &AppWindow, index: i32) {
        let Some(hit) = usize::try_from(index)
            .ok()
            .and_then(|i| self.suggestions.get(i))
            .cloned()
        else {
            return;
        };
        if !self.selected.iter().any(|s| s.id == hit.id) {
            self.selected.push(hit);
        }
        self.suggestions.clear();
        window.set_spot_query(SharedString::new());
        window.set_spot_suggestions(strings(std::iter::empty::<&str>()));
        self.show_selection(window);
        self.run_search(window);
    }

    fn show_selection(&self, window: &AppWindow) {
        window.set_spot_selected_items(strings(self.selected.iter().map(|s| s.name.as_str())));
    }

    fn filter(&self, window: &AppWindow) -> SpotFilter {
        SpotFilter {
            item_ids: self.selected.iter().map(|s| s.id).collect(),
            mission_type_id: usize::try_from(window.get_spot_mission_type_index())
                .ok()
                .and_then(|i| self.mission_types.get(i))
                .and_then(|choice| choice.id),
            planet_id: usize::try_from(window.get_spot_planet_index())
                .ok()
                .and_then(|i| self.planets.get(i))
                .and_then(|choice| choice.id),
            enemy_id: self.picked.as_ref().map(|picked| picked.id),
        }
    }

    /// New items or filters: both tables are recomputed and the picked enemy is dropped,
    /// since it may no longer be listed.
    fn run_search(&mut self, window: &AppWindow) {
        self.picked = None;
        window.set_spot_enemy_current_row(-1);
        let filter = self.filter(window);
        self.enemies = match &self.conn {
            None => empty_enemies(),
            Some(conn) => match db::enemies_for_items(conn, &filter) {
                Ok(found) => enemy_table(&found),
                Err(error) => {
                    log::error!("Recherche Spot, ennemis : {error}");
                    window.set_spot_summary(format!("Recherche impossible : {error}").into());
                    empty_enemies()
                }
            },
        };
        window.set_spot_enemy_columns(columns(&self.enemies));
        window.set_spot_enemy_rows(rows(&self.enemies));
        self.run_nodes(window);
    }

    fn run_nodes(&mut self, window: &AppWindow) {
        let filter = self.filter(window);
        let result = match &self.conn {
            None => Ok(empty_nodes()),
            Some(conn) => db::spots_for_items(conn, &filter).map(|found| node_table(&found)),
        };
        let summary_text = match result {
            Ok(table) => {
                self.nodes = table;
                summary(
                    self.selected.len(),
                    self.enemies.rows.len(),
                    self.nodes.rows.len(),
                )
            }
            Err(error) => {
                log::error!("Recherche Spot, nœuds : {error}");
                self.nodes = empty_nodes();
                format!("Recherche impossible : {error}")
            }
        };
        window.set_spot_summary(summary_text.into());
        window.set_spot_enemy_picked(self.picked.is_some());
        window.set_spot_nodes_title(
            nodes_title(self.picked.as_ref().map(|picked| picked.name.as_str())).into(),
        );
        window.set_spot_node_columns(columns(&self.nodes));
        window.set_spot_node_rows(rows(&self.nodes));
    }

    fn pick_enemy(&mut self, window: &AppWindow, row: i32) {
        let picked = usize::try_from(row)
            .ok()
            .and_then(|i| self.enemies.rows.get(i))
            .and_then(|row| {
                Some(PickedEnemy {
                    id: row.id?,
                    name: row.cells.first()?.clone(),
                })
            });
        let Some(picked) = picked else {
            return;
        };
        if self.picked.as_ref().is_some_and(|p| p.id == picked.id) {
            return;
        }
        self.picked = Some(picked);
        self.run_nodes(window);
    }

    /// Sorting moves the rows under the highlighted index, so the highlight is cleared; the
    /// picked enemy itself stays.
    fn sort_enemies(&mut self, window: &AppWindow, column: i32, descending: bool) {
        if let Ok(column) = usize::try_from(column) {
            self.enemies.sort(column, descending);
            window.set_spot_enemy_current_row(-1);
            window.set_spot_enemy_rows(rows(&self.enemies));
        }
    }

    fn sort_nodes(&mut self, window: &AppWindow, column: i32, descending: bool) {
        if let Ok(column) = usize::try_from(column) {
            self.nodes.sort(column, descending);
            window.set_spot_node_rows(rows(&self.nodes));
        }
    }
}

/// Label of the selector entry at `index`, when `label` keeps it.
fn label_at<T>(choices: &[T], index: i32, label: impl Fn(&T) -> Option<String>) -> Option<String> {
    usize::try_from(index)
        .ok()
        .and_then(|i| choices.get(i))
        .and_then(label)
}

/// Index of the entry labelled `label`, or 0 ("every…").
fn index_of<T>(choices: &[T], label: Option<String>, text: impl Fn(&T) -> &String) -> i32 {
    label
        .and_then(|label| choices.iter().position(|c| *text(c) == label))
        .and_then(|i| i32::try_from(i).ok())
        .unwrap_or(0)
}
