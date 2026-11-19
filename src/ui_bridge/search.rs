//! Search tab controller. Queries run on the UI thread, since they answer in a few
//! milliseconds (measures in TASK-005); the read connection is closed during an import.

use std::cell::RefCell;
use std::rc::Rc;

use rusqlite::Connection;
use slint::language::TableColumn;
use slint::{ModelRc, SharedString, StandardListViewItem, VecModel};

use super::AppWindow;
use crate::db::{self, DbError, ItemHit, RotationFilter, SearchFilter};
use crate::search_view::{
    CategoryChoice, MissionTypeChoice, ResultTable, SearchMode, category_choices,
    mission_type_choices, node_filter_from_index, suggestion_label, summary,
};
use crate::update::DataPaths;

const SUGGESTION_LIMIT: usize = 20;
/// Browsing a whole type with an empty query: the list scrolls, every item of the largest
/// category fits.
const BROWSE_LIMIT: usize = 2000;

/// Number of suggestions shown for `text`.
pub(super) fn suggestion_limit(text: &str) -> usize {
    if text.trim().is_empty() {
        BROWSE_LIMIT
    } else {
        SUGGESTION_LIMIT
    }
}

pub struct SearchState {
    paths: DataPaths,
    conn: Option<Connection>,
    mission_types: Vec<MissionTypeChoice>,
    categories: Vec<CategoryChoice>,
    suggestions: Vec<ItemHit>,
    selected: Vec<ItemHit>,
    table: ResultTable,
}

pub type SharedSearch = Rc<RefCell<SearchState>>;

/// Wires the Search tab callbacks and loads the database; returns log lines for a read error.
pub fn install(window: &AppWindow, paths: DataPaths) -> (SharedSearch, Vec<String>) {
    let state = Rc::new(RefCell::new(SearchState {
        paths,
        conn: None,
        mission_types: Vec::new(),
        categories: Vec::new(),
        suggestions: Vec::new(),
        selected: Vec::new(),
        table: ResultTable::empty(SearchMode::ByItem),
    }));

    window.on_search_query_edited(with_state(
        window,
        &state,
        |window, state, text: SharedString| {
            state.update_suggestions(window, text.as_str());
        },
    ));
    let category_changed = with_state(window, &state, |window, state, ()| {
        let text = window.get_search_query();
        state.update_suggestions(window, text.as_str());
    });
    window.on_search_category_changed(move || category_changed(()));
    window.on_search_add_suggestion(with_state(window, &state, |window, state, index| {
        state.add_suggestion(window, index);
    }));
    window.on_search_remove_item(with_state(window, &state, |window, state, index| {
        if let Some(position) = usize::try_from(index)
            .ok()
            .filter(|&i| i < state.selected.len())
        {
            state.selected.remove(position);
            state.show_selection(window);
            state.run_search(window);
        }
    }));
    let filters_changed = with_state(window, &state, |window, state, ()| {
        state.run_search(window);
    });
    window.on_search_filters_changed(move || filters_changed(()));
    window.on_search_sort_ascending(with_state(window, &state, |window, state, column| {
        state.sort(window, column, false);
    }));
    window.on_search_sort_descending(with_state(window, &state, |window, state, column| {
        state.sort(window, column, true);
    }));

    let lines = state.borrow_mut().reload(window);
    (state, lines)
}

/// Adapts a state method to a Slint callback; the window and state are held weakly.
pub(super) fn with_state<S: 'static, A: 'static>(
    window: &AppWindow,
    state: &Rc<RefCell<S>>,
    action: impl Fn(&AppWindow, &mut S, A) + 'static,
) -> impl Fn(A) + 'static {
    let weak = slint::ComponentHandle::as_weak(window);
    let state = Rc::downgrade(state);
    move |argument| {
        let (Some(window), Some(state)) = (weak.upgrade(), state.upgrade()) else {
            return;
        };
        // Setting Slint properties never calls back synchronously, so the state is free here.
        match state.try_borrow_mut() {
            Ok(mut state) => action(&window, &mut state, argument),
            Err(error) => log::error!("État de la recherche occupé : {error}"),
        };
    }
}

impl SearchState {
    /// Closes the read connection before an import writes the database.
    pub fn close(&mut self) {
        self.conn = None;
    }

    /// Reopens the database after an import, keeping the mission type and the chosen items
    /// whose names still exist (an import renumbers the items).
    pub fn reload(&mut self, window: &AppWindow) -> Vec<String> {
        let mut lines = Vec::new();
        self.conn = match self.open() {
            Ok(conn) => conn,
            Err(error) => {
                log::error!("Recherche, ouverture de la base : {error}");
                lines.push(format!("Recherche indisponible : {error}"));
                None
            }
        };
        let previous_type = self
            .mission_types
            .get(usize::try_from(window.get_search_mission_type_index()).unwrap_or(0))
            .filter(|choice| choice.id.is_some())
            .map(|choice| choice.label.clone());
        let previous_category = self
            .categories
            .get(usize::try_from(window.get_search_category_index()).unwrap_or(0))
            .filter(|choice| choice.category.is_some())
            .map(|choice| choice.label.clone());

        let (has_data, types, categories, selected) = match self.conn.as_ref().map(|conn| {
            Ok::<_, DbError>((
                db::stats(conn)?.items > 0,
                db::mission_types(conn)?,
                db::item_categories(conn)?,
                resolve_by_name(conn, &self.selected)?,
            ))
        }) {
            Some(Ok(loaded)) => loaded,
            Some(Err(error)) => {
                log::error!("Recherche, lecture de la base : {error}");
                lines.push(format!("Recherche indisponible : {error}"));
                (false, Vec::new(), Vec::new(), Vec::new())
            }
            None => (false, Vec::new(), Vec::new(), Vec::new()),
        };
        self.mission_types = mission_type_choices(&types);
        self.categories = category_choices(&categories);
        self.selected = selected;
        self.suggestions.clear();

        let type_index = previous_type
            .and_then(|label| {
                self.mission_types
                    .iter()
                    .position(|choice| choice.label == label)
            })
            .unwrap_or(0);
        window.set_search_has_data(has_data);
        window
            .set_search_mission_types(strings(self.mission_types.iter().map(|c| c.label.as_str())));
        window.set_search_mission_type_index(i32::try_from(type_index).unwrap_or(0));
        let category_index = previous_category
            .and_then(|label| self.categories.iter().position(|c| c.label == label))
            .unwrap_or(0);
        window.set_search_categories(strings(self.categories.iter().map(|c| c.label.as_str())));
        window.set_search_category_index(i32::try_from(category_index).unwrap_or(0));
        window.set_search_query(SharedString::new());
        window.set_search_suggestions(strings(std::iter::empty::<&str>()));
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
        let category = selected_category(&self.categories, window.get_search_category_index());
        self.suggestions = match &self.conn {
            Some(conn) => {
                let shown = suggestion_limit(text);
                let limit = shown + self.selected.len();
                match db::search_items(
                    conn,
                    text,
                    category,
                    u32::try_from(limit).unwrap_or(u32::MAX),
                ) {
                    Ok(hits) => hits
                        .into_iter()
                        .filter(|hit| !self.selected.iter().any(|s| s.id == hit.id))
                        .take(shown)
                        .collect(),
                    Err(error) => {
                        log::error!("Autocomplétion : {error}");
                        Vec::new()
                    }
                }
            }
            None => Vec::new(),
        };
        let labels: Vec<String> = self.suggestions.iter().map(suggestion_label).collect();
        window.set_search_suggestions(strings(labels.iter().map(String::as_str)));
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
        window.set_search_query(SharedString::new());
        window.set_search_suggestions(strings(std::iter::empty::<&str>()));
        self.show_selection(window);
        self.run_search(window);
    }

    fn show_selection(&self, window: &AppWindow) {
        window.set_search_selected_items(strings(self.selected.iter().map(|s| s.name.as_str())));
    }

    fn run_search(&mut self, window: &AppWindow) {
        let mode = SearchMode::from_index(window.get_search_mode_index());
        let filter = SearchFilter {
            mission_type_id: usize::try_from(window.get_search_mission_type_index())
                .ok()
                .and_then(|i| self.mission_types.get(i))
                .and_then(|choice| choice.id),
            rotations: RotationFilter {
                a: window.get_search_rotation_a(),
                b: window.get_search_rotation_b(),
                c: window.get_search_rotation_c(),
            },
            specials: node_filter_from_index(window.get_search_specials_index()),
            dark_sectors: node_filter_from_index(window.get_search_dark_sectors_index()),
            item_ids: self.selected.iter().map(|s| s.id).collect(),
        };
        let result = match (&self.conn, mode) {
            (None, _) => Ok(ResultTable::empty(mode)),
            (Some(conn), SearchMode::ByItem) => {
                db::search_by_item(conn, &filter).map(|rows| ResultTable::by_item(&rows))
            }
            (Some(conn), SearchMode::Combined) => {
                db::search_combined(conn, &filter).map(|rows| ResultTable::combined(&rows))
            }
        };
        let summary_text = match result {
            Ok(table) => {
                self.table = table;
                summary(self.selected.len(), self.table.rows.len())
            }
            Err(error) => {
                log::error!("Recherche : {error}");
                self.table = ResultTable::empty(mode);
                format!("Recherche impossible : {error}")
            }
        };
        window.set_search_summary(summary_text.into());
        window.set_search_columns(columns(&self.table));
        self.show_rows(window);
    }

    fn sort(&mut self, window: &AppWindow, column: i32, descending: bool) {
        if let Ok(column) = usize::try_from(column) {
            self.table.sort(column, descending);
            self.show_rows(window);
        }
    }

    fn show_rows(&self, window: &AppWindow) {
        window.set_search_rows(rows(&self.table));
    }
}

/// Chosen items found again by exact name after an import.
pub(super) fn resolve_by_name(
    conn: &Connection,
    selected: &[ItemHit],
) -> Result<Vec<ItemHit>, DbError> {
    let mut found = Vec::with_capacity(selected.len());
    for item in selected {
        // Prefix matches come first by name, so the exact name is among the first hits.
        if let Some(hit) = db::search_items(conn, &item.name, None, 10)?
            .into_iter()
            .find(|hit| hit.name == item.name)
        {
            found.push(hit);
        }
    }
    Ok(found)
}

/// Category of the item type selector entry at `index`; `None` for "every type".
pub(super) fn selected_category(choices: &[CategoryChoice], index: i32) -> Option<&str> {
    usize::try_from(index)
        .ok()
        .and_then(|i| choices.get(i))
        .and_then(|choice| choice.category.as_deref())
}

pub(super) fn columns(table: &ResultTable) -> ModelRc<TableColumn> {
    let columns: Vec<TableColumn> = table
        .columns
        .iter()
        .map(|title| {
            let mut column = TableColumn::default();
            column.title = (*title).into();
            column.horizontal_stretch = 1.0;
            column
        })
        .collect();
    ModelRc::new(VecModel::from(columns))
}

/// Cells of a result table for a `StandardTableView`.
pub(super) fn rows(table: &ResultTable) -> ModelRc<ModelRc<StandardListViewItem>> {
    let rows: Vec<ModelRc<StandardListViewItem>> = table
        .rows
        .iter()
        .map(|row| {
            let cells: Vec<StandardListViewItem> = row
                .cells
                .iter()
                .map(|cell| StandardListViewItem::from(cell.as_str()))
                .collect();
            ModelRc::new(VecModel::from(cells))
        })
        .collect();
    ModelRc::new(VecModel::from(rows))
}

pub(super) fn strings<'a>(texts: impl Iterator<Item = &'a str>) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(
        texts.map(SharedString::from).collect::<Vec<_>>(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chosen_items_are_found_again_by_exact_name() -> Result<(), Box<dyn std::error::Error>> {
        let conn = db::open_in_memory()?;
        conn.execute_batch(
            "INSERT INTO item (name, category) VALUES
                 ('Lith A1 Relic', 'Relic'), ('Lith A1', 'Relic'), ('Lith A10', 'Relic');",
        )?;
        let stale = |name: &str| ItemHit {
            id: -1,
            name: name.to_owned(),
            category: "Relic".to_owned(),
        };
        let found = resolve_by_name(&conn, &[stale("Lith A1"), stale("Gone")])?;
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Lith A1");
        assert!(found[0].id > 0);
        Ok(())
    }
}
