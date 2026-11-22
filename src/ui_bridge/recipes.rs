//! Recipe editing stays local; calculations and drop queries run on a worker thread.

use super::{
    AppWindow,
    generated::{Crafting, RecipeIngredient},
    search::{strings, with_state},
};
use crate::{
    db,
    recipe_view::{self, RecipePresentation},
    recipes::{self, Ingredient, Recipe},
    search_view::{MissionTypeChoice, mission_type_choices},
    update::DataPaths,
};
use slint::{ComponentHandle, ModelRc, StandardListViewItem, VecModel};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

pub struct RecipeState {
    paths: DataPaths,
    catalogue: BTreeMap<String, Recipe>,
    filtered: Vec<String>,
    ingredients: Vec<Ingredient>,
    missions: Vec<MissionTypeChoice>,
}

pub fn install(window: &AppWindow, paths: DataPaths) -> Rc<RefCell<RecipeState>> {
    let state = Rc::new(RefCell::new(RecipeState {
        paths,
        catalogue: BTreeMap::new(),
        filtered: Vec::new(),
        ingredients: Vec::new(),
        missions: Vec::new(),
    }));
    let ui = window.global::<Crafting>();
    let callback = with_state(window, &state, |w, s: &mut RecipeState, ()| s.search(w));
    ui.on_search(move || callback(()));
    ui.on_select(with_state(
        window,
        &state,
        |w, s: &mut RecipeState, index| s.select(w, index),
    ));
    let callback = with_state(window, &state, |w, s: &mut RecipeState, ()| s.new_recipe(w));
    ui.on_new_recipe(move || callback(()));
    let callback = with_state(window, &state, |w, s: &mut RecipeState, ()| {
        s.add_ingredient(w)
    });
    ui.on_add_ingredient(move || callback(()));
    ui.on_remove_ingredient(with_state(
        window,
        &state,
        |w, s: &mut RecipeState, index| {
            if let Ok(index) = usize::try_from(index)
                && index < s.ingredients.len()
            {
                s.ingredients.remove(index);
                s.show_ingredients(w);
                invalidate(w);
            }
        },
    ));
    let callback = with_state(window, &state, |w, s: &mut RecipeState, ()| s.save(w));
    ui.on_save_recipe(move || callback(()));
    let callback = with_state(window, &state, |w, s: &mut RecipeState, ()| {
        s.save_profile(w)
    });
    ui.on_save_profile(move || callback(()));
    let callback = with_state(window, &state, |w, s: &mut RecipeState, ()| s.calculate(w));
    ui.on_calculate(move || callback(()));
    let weak = window.as_weak();
    ui.on_invalidate(move || {
        if let Some(w) = weak.upgrade() {
            invalidate(&w);
        }
    });
    state.borrow_mut().reload(window);
    state
}

fn invalidate(window: &AppWindow) {
    let ui = window.global::<Crafting>();
    ui.set_resources(ModelRc::default());
    ui.set_blueprints(ModelRc::default());
    ui.set_steps(ModelRc::default());
    ui.set_summary(
        "Recette ou filtres modifiés : calculez les besoins pour afficher les résultats.".into(),
    );
}

fn positive(value: &str, label: &str) -> Result<u32, String> {
    value
        .trim()
        .parse::<u32>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| format!("{label} : saisissez un entier positif."))
}

fn fail(window: &AppWindow, message: String) {
    log::error!("Recettes : {message}");
    window.global::<Crafting>().set_message(message.into());
}

impl RecipeState {
    pub fn reload(&mut self, window: &AppWindow) {
        let previous_mission = usize::try_from(window.global::<Crafting>().get_mission_index())
            .ok()
            .and_then(|i| self.missions.get(i))
            .map(|m| m.label.clone());
        let read = || -> Result<_, db::DbError> {
            if !self.paths.database.is_file() {
                return Ok((BTreeMap::new(), None, Vec::new()));
            }
            let conn = db::open(&self.paths.database)?;
            Ok((
                db::recipes(&conn)?,
                db::profile_level(&conn)?,
                db::mission_types(&conn)?,
            ))
        };
        match read() {
            Ok((catalogue, level, missions)) => {
                self.catalogue = catalogue;
                self.missions = mission_type_choices(&missions);
                let ui = window.global::<Crafting>();
                ui.set_level(level.map(|n| n.to_string()).unwrap_or_default().into());
                ui.set_profile_message(level.map_or_else(||"Profil enregistré : sans plafond.".to_owned(),|n|format!("Profil enregistré : niveau maximal {n}, filtre par chevauchement." )).into());
                ui.set_mission_types(strings(self.missions.iter().map(|m| m.label.as_str())));
                let index = previous_mission
                    .and_then(|label| self.missions.iter().position(|m| m.label == label))
                    .unwrap_or(0);
                ui.set_mission_index(i32::try_from(index).unwrap_or(0));
                invalidate(window);
                self.search(window);
            }
            Err(e) => fail(window, format!("Lecture des recettes impossible : {e}")),
        }
    }

    fn search(&mut self, window: &AppWindow) {
        let ui = window.global::<Crafting>();
        let needle = db::fold_text(ui.get_query().trim());
        self.filtered = self
            .catalogue
            .iter()
            .filter(|(n, r)| {
                db::fold_text(n).contains(&needle) || db::fold_text(&r.blueprint).contains(&needle)
            })
            .take(100)
            .map(|(n, _)| n.clone())
            .collect();
        ui.set_catalogue(strings(self.filtered.iter().map(String::as_str)));
        ui.set_catalogue_index(-1);
        ui.set_catalogue_summary(format!("{} recettes disponibles · {} propositions affichées (maximum 100). Affinez la recherche si nécessaire.",self.catalogue.len(),self.filtered.len()).into());
    }

    fn select(&mut self, window: &AppWindow, index: i32) {
        let Some(recipe) = usize::try_from(index)
            .ok()
            .and_then(|i| self.filtered.get(i))
            .and_then(|n| self.catalogue.get(n))
            .cloned()
        else {
            return;
        };
        let ui = window.global::<Crafting>();
        ui.set_result(recipe.result.into());
        ui.set_blueprint(recipe.blueprint.into());
        ui.set_produced(recipe.quantity.to_string().into());
        ui.set_credits(recipe.credits.to_string().into());
        self.ingredients = recipe.ingredients;
        self.show_ingredients(window);
        invalidate(window);
        ui.set_message(
            "Recette chargée. La saisie permet d’enregistrer une copie locale prioritaire.".into(),
        );
    }

    fn new_recipe(&mut self, window: &AppWindow) {
        let ui = window.global::<Crafting>();
        ui.set_result("".into());
        ui.set_blueprint("".into());
        ui.set_produced("1".into());
        ui.set_credits("0".into());
        ui.set_ingredient("".into());
        ui.set_ingredient_quantity("1".into());
        ui.set_ingredient_crafted(false);
        self.ingredients.clear();
        self.show_ingredients(window);
        invalidate(window);
        ui.set_message(
            "Renseignez la recette dans l’onglet Saisie de la recette, puis enregistrez-la.".into(),
        );
    }

    fn show_ingredients(&self, window: &AppWindow) {
        let rows = self
            .ingredients
            .iter()
            .map(|i| RecipeIngredient {
                name: i.name.clone().into(),
                quantity: i.quantity.to_string().into(),
                crafted: i.crafted,
            })
            .collect::<Vec<_>>();
        window
            .global::<Crafting>()
            .set_ingredients(ModelRc::new(VecModel::from(rows)));
    }

    fn add_ingredient(&mut self, window: &AppWindow) {
        let ui = window.global::<Crafting>();
        let name = ui.get_ingredient().trim().to_owned();
        let quantity = match positive(
            ui.get_ingredient_quantity().as_str(),
            "Quantité de l’ingrédient",
        ) {
            Ok(q) => q,
            Err(e) => {
                fail(window, e);
                return;
            }
        };
        if name.is_empty() {
            fail(window, "Renseignez le nom complet de l’ingrédient.".into());
            return;
        }
        self.ingredients.push(Ingredient {
            name,
            quantity,
            crafted: ui.get_ingredient_crafted(),
        });
        self.show_ingredients(window);
        invalidate(window);
        ui.set_ingredient("".into());
        ui.set_message("Ingrédient ajouté à la recette en cours de saisie.".into());
    }

    fn draft(&self, window: &AppWindow) -> Result<Recipe, String> {
        let ui = window.global::<Crafting>();
        let recipe = Recipe {
            result: ui.get_result().trim().to_owned(),
            blueprint: ui.get_blueprint().trim().to_owned(),
            quantity: positive(ui.get_produced().as_str(), "Quantité produite")?,
            credits: ui
                .get_credits()
                .trim()
                .parse()
                .map_err(|_| "Crédits : saisissez un entier positif ou nul.".to_owned())?,
            ingredients: self.ingredients.clone(),
        };
        recipe.validate().map_err(|e| e.to_string())?;
        Ok(recipe)
    }

    fn writable(&self) -> Result<rusqlite::Connection, String> {
        std::fs::create_dir_all(&self.paths.dir)
            .map_err(|e| format!("Création du dossier de données impossible : {e}"))?;
        db::open(&self.paths.database).map_err(|e| e.to_string())
    }

    fn save(&mut self, window: &AppWindow) {
        let write = || -> Result<Recipe, String> {
            let draft = self.draft(window)?;
            let conn = self.writable()?;
            db::save_local_recipe(&conn, &draft).map_err(|e| e.to_string())?;
            Ok(draft)
        };
        match write() {
            Ok(draft) => {
                self.catalogue.insert(draft.result.clone(), draft);
                self.search(window);
                invalidate(window);
                window.global::<Crafting>().set_message(
                    "Recette locale enregistrée. Elle sera conservée aux mises à jour.".into(),
                );
            }
            Err(e) => fail(window, e),
        }
    }

    fn save_profile(&mut self, window: &AppWindow) {
        let ui = window.global::<Crafting>();
        let value = ui.get_level();
        let level = if value.trim().is_empty() {
            None
        } else {
            match positive(&value, "Niveau maximal des ennemis") {
                Ok(n) => Some(n),
                Err(e) => {
                    ui.set_profile_message(e.into());
                    return;
                }
            }
        };
        let write = || -> Result<(), String> {
            db::save_profile_level(&self.writable()?, level).map_err(|e| e.to_string())
        };
        match write() {
            Ok(()) => {
                invalidate(window);
                ui.set_profile_message(level.map_or_else(||"Profil enregistré : sans plafond.".to_owned(),|n|format!("Profil enregistré : niveau maximal {n}, filtre par chevauchement.")).into());
                ui.set_message(
                    "Profil enregistré : recalculez les besoins pour appliquer le plafond.".into(),
                );
            }
            Err(e) => {
                log::error!("Profil : {e}");
                ui.set_profile_message(e.into());
            }
        }
    }

    fn calculate(&mut self, window: &AppWindow) {
        let ui = window.global::<Crafting>();
        let draft = match self.draft(window) {
            Ok(r) => r,
            Err(e) => {
                fail(window, e);
                return;
            }
        };
        let quantity = match positive(ui.get_target().as_str(), "Quantité à fabriquer") {
            Ok(n) => n,
            Err(e) => {
                fail(window, e);
                return;
            }
        };
        let mission = usize::try_from(ui.get_mission_index())
            .ok()
            .and_then(|i| self.missions.get(i))
            .and_then(|m| m.id);
        let paths = self.paths.clone();
        if let Err(e) = self.writable() {
            fail(window, e);
            return;
        }
        invalidate(window);
        ui.set_calculating(true);
        ui.set_message("Calcul des besoins et recherche des spots…".into());
        let weak = window.as_weak();
        let job=std::thread::Builder::new().name("recipes".into()).spawn(move|| {
            let calculate=||->Result<RecipePresentation,String>{
                let mut conn=db::open(&paths.database).map_err(|e|e.to_string())?;
                let tx=conn.transaction().map_err(|e|e.to_string())?;
                let mut catalogue=db::recipes(&tx).map_err(|e|e.to_string())?;
                let name=draft.result.clone();catalogue.insert(name.clone(),draft);
                let plan=recipes::plan(&catalogue,&name,quantity).map_err(|e|e.to_string())?;
                let level=db::profile_level(&tx).map_err(|e|e.to_string())?;
                recipe_view::present(&tx,&plan,level,mission).map_err(|e|e.to_string())
            };
            let result=calculate();
            if let Err(e)=slint::invoke_from_event_loop(move||{if let Some(window)=weak.upgrade(){
                let ui=window.global::<Crafting>();ui.set_calculating(false);
                match result{Ok(view)=>{ui.set_summary(view.summary.into());ui.set_resources(matrix(view.resources));ui.set_blueprints(matrix(view.blueprints));ui.set_steps(strings(view.steps.iter().map(String::as_str)));ui.set_message("Besoins calculés. Consultez aussi les schémas requis et le détail des fabrications.".into());},Err(e)=>fail(&window,e)}
            }}) {log::error!("Retour du calcul vers l’interface impossible : {e}");}
        });
        if let Err(e) = job {
            ui.set_calculating(false);
            fail(window, format!("Lancement du calcul impossible : {e}"));
        }
    }
}

fn matrix(rows: Vec<Vec<String>>) -> ModelRc<ModelRc<StandardListViewItem>> {
    ModelRc::new(VecModel::from(
        rows.into_iter()
            .map(|row| {
                ModelRc::new(VecModel::from(
                    row.into_iter()
                        .map(|text| StandardListViewItem::from(text.as_str()))
                        .collect::<Vec<_>>(),
                ))
            })
            .collect::<Vec<_>>(),
    ))
}
