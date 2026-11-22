//! Items tab controller. Queries run on the UI thread at each keystroke, like the other tabs:
//! the results are capped at `RESULT_LIMIT` rows; the read connection is closed during an import.

use std::cell::RefCell;
use std::rc::Rc;

use rusqlite::Connection;
use slint::{ComponentHandle, SharedString};

use super::AppWindow;
use super::search::{columns, rows, strings, with_state};
use crate::db::{self, DbError};
use crate::item_view::{
    RESULT_LIMIT, category_labels, detail_table, detail_title, empty_detail, empty_items,
    item_table, ordered_categories, selected_category, summary,
};
use crate::search_view::ResultTable;
use crate::update::DataPaths;

pub struct ItemState {
    paths: DataPaths,
    conn: Option<Connection>,
    categories: Vec<String>,
    table: ResultTable,
    detail: ResultTable,
    /// Id of the item whose detail is shown.
    picked: Option<i64>,
}

pub type SharedItems = Rc<RefCell<ItemState>>;

/// Wires the Items tab callbacks and loads the database; returns log lines for a read error.
pub fn install(window: &AppWindow, paths: DataPaths) -> (SharedItems, Vec<String>) {
    window.on_item_is_web_url(|text| crate::item_view::web_url(&text).is_some());
    let weak = window.as_weak();
    window.on_item_open_url(move |text| {
        let Some(url) = crate::item_view::web_url(&text) else {
            return;
        };
        let weak = weak.clone();
        std::thread::spawn(move || {
            if let Err(error) = open_browser(url.as_str()) {
                log::error!("Ouverture du navigateur pour {url} : {error}");
                if let Err(error) = weak.upgrade_in_event_loop(move |window| {
                    window.set_item_link_error(
                        format!("Impossible d'ouvrir le navigateur : {error}").into(),
                    );
                }) {
                    log::error!("Affichage de l'erreur du navigateur : {error}");
                }
            }
        });
    });
    let state = Rc::new(RefCell::new(ItemState {
        paths,
        conn: None,
        categories: Vec::new(),
        table: empty_items(),
        detail: empty_detail(),
        picked: None,
    }));

    window.on_item_query_edited(with_state(
        window,
        &state,
        |window, state: &mut ItemState, _text: SharedString| {
            state.run_search(window);
        },
    ));
    let category_changed = with_state(window, &state, |window, state: &mut ItemState, ()| {
        state.run_search(window);
    });
    window.on_item_category_changed(move || category_changed(()));
    window.on_item_selected(with_state(
        window,
        &state,
        |window, state: &mut ItemState, row| {
            state.pick(window, row);
        },
    ));
    window.on_item_sort_ascending(with_state(
        window,
        &state,
        |window, state: &mut ItemState, column| {
            state.sort_items(window, column, false);
        },
    ));
    window.on_item_sort_descending(with_state(
        window,
        &state,
        |window, state: &mut ItemState, column| {
            state.sort_items(window, column, true);
        },
    ));
    window.on_item_detail_sort_ascending(with_state(
        window,
        &state,
        |window, state: &mut ItemState, column| {
            state.sort_detail(window, column, false);
        },
    ));
    window.on_item_detail_sort_descending(with_state(
        window,
        &state,
        |window, state: &mut ItemState, column| {
            state.sort_detail(window, column, true);
        },
    ));

    let lines = state.borrow_mut().reload(window);
    (state, lines)
}

impl ItemState {
    /// Closes the read connection before an import writes the database.
    pub fn close(&mut self) {
        self.conn = None;
    }

    /// Reopens the database after an import, keeping the chosen category when it still exists
    /// and the typed name; the shown detail is dropped, since an import renumbers the items.
    pub fn reload(&mut self, window: &AppWindow) -> Vec<String> {
        let mut lines = Vec::new();
        let previous = selected_category(&self.categories, window.get_item_category_index())
            .map(str::to_owned);
        self.conn = match self.open() {
            Ok(conn) => conn,
            Err(error) => {
                log::error!("Items, ouverture de la base : {error}");
                lines.push(format!("Onglet Items indisponible : {error}"));
                None
            }
        };
        self.categories = match self.conn.as_ref().map(db::wfcd_categories) {
            Some(Ok(categories)) => ordered_categories(&categories),
            Some(Err(error)) => {
                log::error!("Items, lecture de la base : {error}");
                lines.push(format!("Onglet Items indisponible : {error}"));
                Vec::new()
            }
            None => Vec::new(),
        };
        let index = previous
            .and_then(|category| self.categories.iter().position(|c| *c == category))
            .map_or(0, |position| position + 1);
        window.set_item_has_data(!self.categories.is_empty());
        window.set_item_categories(strings(category_labels(&self.categories).into_iter()));
        window.set_item_category_index(i32::try_from(index).unwrap_or(0));
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

    /// New name or category: the results are recomputed and the detail is cleared, since the
    /// item may no longer be listed.
    fn run_search(&mut self, window: &AppWindow) {
        let category = selected_category(&self.categories, window.get_item_category_index());
        let query = window.get_item_query();
        let found = match &self.conn {
            Some(conn) => db::search_wfcd_items(
                conn,
                category,
                query.as_str(),
                u32::try_from(RESULT_LIMIT).unwrap_or(u32::MAX),
            ),
            None => Ok((Vec::new(), 0)),
        };
        let text = match found {
            Ok((hits, total)) => {
                self.table = item_table(&hits);
                summary(!self.categories.is_empty(), hits.len(), total)
            }
            Err(error) => {
                log::error!("Items : {error}");
                self.table = empty_items();
                format!("Recherche impossible : {error}")
            }
        };
        window.set_item_summary(text.into());
        window.set_item_columns(columns(&self.table));
        window.set_item_current_row(-1);
        window.set_item_rows(rows(&self.table));
        self.picked = None;
        self.show_detail(window, None, empty_detail());
    }

    fn pick(&mut self, window: &AppWindow, row: i32) {
        let Some((id, name)) = usize::try_from(row)
            .ok()
            .and_then(|i| self.table.rows.get(i))
            .and_then(|row| Some((row.id?, row.cells.first()?.clone())))
        else {
            return;
        };
        if self.picked == Some(id) {
            return;
        }
        self.picked = Some(id);
        let detail = match &self.conn {
            Some(conn) => db::wfcd_item_detail(conn, id),
            None => Ok(None),
        };
        let table = match detail {
            Ok(Some((json, json_fr))) => match detail_table(&json, json_fr.as_deref()) {
                Ok(table) => table,
                Err(error) => {
                    log::error!("Items, JSON de l'item {id} illisible : {error}");
                    empty_detail()
                }
            },
            Ok(None) => empty_detail(),
            Err(error) => {
                log::error!("Items, détail de l'item {id} : {error}");
                empty_detail()
            }
        };
        self.show_detail(window, Some(&name), table);
    }

    fn show_detail(&mut self, window: &AppWindow, name: Option<&str>, table: ResultTable) {
        window.set_item_link_error(SharedString::default());
        self.detail = table;
        window.set_item_detail_title(detail_title(name).into());
        window.set_item_detail_columns(columns(&self.detail));
        window.set_item_detail_rows(rows(&self.detail));
    }

    /// Sorting moves the rows under the highlighted index, so the highlight is cleared; the
    /// shown detail stays.
    fn sort_items(&mut self, window: &AppWindow, column: i32, descending: bool) {
        if let Ok(column) = usize::try_from(column) {
            self.table.sort(column, descending);
            window.set_item_current_row(-1);
            window.set_item_rows(rows(&self.table));
        }
    }

    fn sort_detail(&mut self, window: &AppWindow, column: i32, descending: bool) {
        if let Ok(column) = usize::try_from(column) {
            self.detail.sort(column, descending);
            window.set_item_detail_rows(rows(&self.detail));
        }
    }
}

fn open_browser(url: &str) -> std::io::Result<()> {
    use std::process::Command;

    #[cfg(target_os = "windows")]
    let mut command = {
        use std::os::windows::process::CommandExt;
        let mut command = Command::new("rundll32.exe");
        command.arg("url.dll,FileProtocolHandler");
        command.creation_flags(0x0800_0000);
        command
    };
    #[cfg(not(target_os = "windows"))]
    let mut command = Command::new("xdg-open");

    let output = command.arg(url).output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "{} : {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}
