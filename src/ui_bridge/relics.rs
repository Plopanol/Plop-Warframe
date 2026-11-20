//! Relics tab controller. One query per Warframe choice, on the UI thread like the search tabs
//! (a few hundred rows at most); the read connection is closed during an import.

use std::cell::RefCell;
use std::rc::Rc;

use rusqlite::Connection;

use super::AppWindow;
use super::search::{columns, rows, strings, with_state};
use crate::db::{self, DbError};
use crate::relic_view::{empty_relics, relic_table, summary};
use crate::search_view::ResultTable;
use crate::update::DataPaths;

pub struct RelicState {
    paths: DataPaths,
    conn: Option<Connection>,
    warframes: Vec<String>,
    table: ResultTable,
}

pub type SharedRelics = Rc<RefCell<RelicState>>;

/// Wires the Relics tab callbacks and loads the database; returns log lines for a read error.
pub fn install(window: &AppWindow, paths: DataPaths) -> (SharedRelics, Vec<String>) {
    let state = Rc::new(RefCell::new(RelicState {
        paths,
        conn: None,
        warframes: Vec::new(),
        table: empty_relics(),
    }));

    let changed = with_state(window, &state, |window, state: &mut RelicState, ()| {
        state.show_relics(window);
    });
    window.on_relic_warframe_changed(move || changed(()));
    window.on_relic_sort_ascending(with_state(
        window,
        &state,
        |window, state: &mut RelicState, column| {
            state.sort(window, column, false);
        },
    ));
    window.on_relic_sort_descending(with_state(
        window,
        &state,
        |window, state: &mut RelicState, column| {
            state.sort(window, column, true);
        },
    ));

    let lines = state.borrow_mut().reload(window);
    (state, lines)
}

impl RelicState {
    /// Closes the read connection before an import writes the database.
    pub fn close(&mut self) {
        self.conn = None;
    }

    /// Reopens the database after an import, keeping the chosen Warframe when it still exists.
    pub fn reload(&mut self, window: &AppWindow) -> Vec<String> {
        let mut lines = Vec::new();
        let previous = self.selected(window).map(str::to_owned);
        self.conn = match self.open() {
            Ok(conn) => conn,
            Err(error) => {
                log::error!("Reliques, ouverture de la base : {error}");
                lines.push(format!("Onglet Reliques indisponible : {error}"));
                None
            }
        };
        self.warframes = match self.conn.as_ref().map(db::warframes) {
            Some(Ok(warframes)) => warframes,
            Some(Err(error)) => {
                log::error!("Reliques, lecture de la base : {error}");
                lines.push(format!("Onglet Reliques indisponible : {error}"));
                Vec::new()
            }
            None => Vec::new(),
        };
        let index = previous
            .and_then(|name| self.warframes.iter().position(|w| *w == name))
            .and_then(|i| i32::try_from(i).ok())
            .unwrap_or(0);
        window.set_relic_has_data(!self.warframes.is_empty());
        window.set_relic_warframes(strings(self.warframes.iter().map(String::as_str)));
        window.set_relic_warframe_index(index);
        self.show_relics(window);
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

    fn selected(&self, window: &AppWindow) -> Option<&str> {
        usize::try_from(window.get_relic_warframe_index())
            .ok()
            .and_then(|i| self.warframes.get(i))
            .map(String::as_str)
    }

    fn show_relics(&mut self, window: &AppWindow) {
        let warframe = self.selected(window).map(str::to_owned);
        let found = match (&self.conn, &warframe) {
            (Some(conn), Some(name)) => db::relics_for_warframe(conn, name),
            _ => Ok(Vec::new()),
        };
        let text = match found {
            Ok(found) => {
                self.table = relic_table(&found);
                summary(!self.warframes.is_empty(), warframe.as_deref(), &found)
            }
            Err(error) => {
                log::error!("Reliques : {error}");
                self.table = empty_relics();
                format!("Recherche impossible : {error}")
            }
        };
        window.set_relic_summary(text.into());
        window.set_relic_columns(columns(&self.table));
        window.set_relic_rows(rows(&self.table));
    }

    fn sort(&mut self, window: &AppWindow, column: i32, descending: bool) {
        if let Ok(column) = usize::try_from(column) {
            self.table.sort(column, descending);
            window.set_relic_rows(rows(&self.table));
        }
    }
}
