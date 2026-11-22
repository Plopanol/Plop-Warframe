use plop_warframe::recipes::{RecipeError, plan};
use plop_warframe::{
    db,
    recipes::{Ingredient, Recipe},
    source::{BlueprintSource, DropSource},
};
use std::collections::BTreeMap;

#[test]
fn rhino_totals_group_resources_and_keep_blueprints_separate() {
    let data = fixture();
    let recipes = data
        .recipes
        .into_iter()
        .map(|r| (r.result.clone(), r))
        .collect();
    let result = plan(&recipes, "Rhino", 1).unwrap();
    assert_eq!(result.credits, 70_000);
    assert_eq!(result.blueprints.len(), 4);
    assert_eq!(
        result.resources,
        BTreeMap::from([
            ("Alloy Plate".into(), 150),
            ("Circuits".into(), 700),
            ("Ferrite".into(), 1500),
            ("Neurodes".into(), 2),
            ("Polymer Bundle".into(), 250),
            ("Rubedo".into(), 1200)
        ])
    );
}

#[cfg(test)]
fn recipe(name: &str, yield_count: u32, parts: &[(&str, u32)]) -> Recipe {
    Recipe {
        result: name.into(),
        blueprint: format!("{name} Blueprint"),
        quantity: yield_count,
        credits: 1,
        ingredients: parts
            .iter()
            .map(|(n, q)| Ingredient {
                name: (*n).into(),
                quantity: *q,
                crafted: false,
            })
            .collect(),
    }
}

#[test]
fn shared_recipe_demands_are_rounded_after_aggregation() {
    let recipes = [
        recipe("Root", 1, &[("A", 1), ("B", 1)]),
        recipe("A", 1, &[("Shared", 1)]),
        recipe("B", 1, &[("Shared", 1)]),
        recipe("Shared", 2, &[("Ferrite", 5)]),
    ]
    .into_iter()
    .map(|r| (r.result.clone(), r))
    .collect();
    let result = plan(&recipes, "Root", 1).unwrap();
    assert_eq!(result.resources["Ferrite"], 5);
    assert_eq!(
        result
            .steps
            .iter()
            .find(|s| s.result == "Shared")
            .unwrap()
            .runs,
        1
    );
    assert_eq!(plan(&recipes, "Root", 3).unwrap().resources["Ferrite"], 15);
}

#[test]
fn cycles_missing_recipes_and_overflows_are_explicit_errors() {
    let mut recipes: BTreeMap<_, _> = [recipe("A", 1, &[("B", 1)]), recipe("B", 1, &[("A", 1)])]
        .into_iter()
        .map(|r| (r.result.clone(), r))
        .collect();
    assert!(matches!(plan(&recipes, "A", 1), Err(RecipeError::Cycle(_))));
    recipes.remove("B");
    recipes.get_mut("A").unwrap().ingredients[0].crafted = true;
    assert!(matches!(
        plan(&recipes, "A", 1),
        Err(RecipeError::Missing(_))
    ));
    let recipes = [
        recipe("A", 1, &[("B", u32::MAX)]),
        recipe("B", 1, &[("Ferrite", u32::MAX)]),
    ]
    .into_iter()
    .map(|r| (r.result.clone(), r))
    .collect();
    assert!(matches!(
        plan(&recipes, "A", u32::MAX),
        Err(RecipeError::Overflow)
    ));
    assert!(plan(&recipes, "A", 0).is_err());
}

#[test]
fn spots_apply_overlap_and_type_to_both_reward_and_enemy_sources() {
    let conn = db::open_in_memory().unwrap();
    conn.execute_batch(r#"
        INSERT INTO planet VALUES (1,'Earth');
        INSERT INTO mission_type VALUES (1,'Defense',1),(2,'Survival',1);
        INSERT INTO node(id,key,name,planet_id,mission_type_id,min_level,max_level) VALUES
            (1,'overlap','Overlap',1,1,15,25),(2,'high','High',1,1,21,30),(3,'unknown','Unknown',1,1,NULL,NULL),(4,'survival','Survival',1,2,10,20);
        INSERT INTO node_faction VALUES (1,'grineer'),(2,'grineer'),(3,'grineer'),(4,'grineer');
        INSERT INTO item VALUES (1,'Ferrite','Resource');
        INSERT INTO enemy VALUES (1,'Lancer','Lancer','grineer');
        INSERT INTO enemy_drop VALUES (1,1,1,0.1,'common');
        INSERT INTO drop_table VALUES (1,'t','Test','Defense',NULL,'none');
        INSERT INTO node_drop_table VALUES (1,1,'main'),(2,1,'main'),(3,1,'main'),(4,1,'main');
        INSERT INTO "drop" VALUES (1,1,'NONE',1,0.5,'common',1);
    "#).unwrap();
    let found = db::recipe_spots(&conn, "Ferrite", Some(20), Some(1)).unwrap();
    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|r| r.node == "Overlap"));
    assert!(found.iter().any(|r| r.enemy.is_some() && r.chance == 0.1));
    assert!(
        found
            .iter()
            .any(|r| r.location.is_some() && r.chance == 0.5)
    );
    assert_eq!(
        db::recipe_spots(&conn, "Ferrite", None, None)
            .unwrap()
            .len(),
        8
    );
    assert!(
        db::recipe_spots(&conn, "Inconnu", Some(20), None)
            .unwrap()
            .is_empty()
    );
}

#[cfg(test)]
fn fixture() -> plop_warframe::model::DropDataset {
    BlueprintSource(include_str!("fixtures/blueprints.lua"))
        .load()
        .unwrap()
}

#[test]
fn wiki_resolves_relative_parts_without_inventing_resource_recipes() {
    let data = fixture();
    assert_eq!(data.recipes.len(), 4);
    let rhino = data.recipes.iter().find(|r| r.result == "Rhino").unwrap();
    assert!(
        rhino
            .ingredients
            .iter()
            .any(|i| i.name == "Rhino Neuroptics" && i.crafted)
    );
    assert!(
        rhino
            .ingredients
            .iter()
            .any(|i| i.name == "Neurodes" && !i.crafted)
    );
    assert!(
        BlueprintSource("return {Suits={X={Parts={{Count=-1}}}}}")
            .load()
            .is_err()
    );
}

#[test]
fn embedded_costs_are_kept_and_direct_weapon_parts_are_not_crafted() {
    let source = r#"return {Blueprints={X={Name="X Blueprint",Result="X",Parts={
        {Name="Chassis",Type="Item",Count=1},
        {Name="Barrel",Type="PrimePart",Count=1,Cost={Credits=10,Parts={{Name="Ferrite",Type="Resource",Count=4}}}}
    }}}}"#;
    let data = BlueprintSource(source).load().unwrap();
    assert_eq!(data.recipes.len(), 2);
    assert!(
        data.recipes[0]
            .ingredients
            .iter()
            .any(|i| i.name == "X Chassis" && !i.crafted)
    );
    assert_eq!(data.recipes[1].result, "X Barrel");
    assert_eq!(data.recipes[1].ingredients[0].quantity, 4);
}

#[test]
fn contradictory_inline_costs_are_excluded_with_a_warning() {
    let source = r#"return {Blueprints={A={Name="A Blueprint",Result="A",Parts={
        {Name="Shared",Type="Resource",Count=1,Cost={Parts={{Name="Ferrite",Type="Resource",Count=4}}}}
    }}, B={Name="B Blueprint",Result="B",Parts={
        {Name="Shared",Type="Resource",Count=1,Cost={Parts={{Name="Ferrite",Type="Resource",Count=8}}}}
    }}}}"#;
    let data = BlueprintSource(source).load().unwrap();
    assert_eq!(data.recipes.len(), 2);
    assert_eq!(data.warnings.len(), 1);
    assert!(data.recipes.iter().all(|r| r.result != "Shared"));
}

#[test]
fn imports_preserve_local_recipes_and_character_profile() {
    let mut conn = db::open_in_memory().unwrap();
    let mut local = Recipe {
        result: "Local".into(),
        blueprint: "Plan local".into(),
        quantity: 2,
        credits: 10,
        ingredients: vec![Ingredient {
            name: "Ferrite".into(),
            quantity: 5,
            crafted: false,
        }],
    };
    db::save_local_recipe(&conn, &local).unwrap();
    db::save_profile_level(&conn, Some(20)).unwrap();
    for _ in 0..2 {
        db::import_dataset(
            &mut conn,
            &fixture(),
            db::ImportRun {
                source: "test",
                source_revision: None,
            },
            &|| 1,
        )
        .unwrap();
        assert_eq!(db::recipes(&conn).unwrap().len(), 5);
        assert_eq!(db::profile_level(&conn).unwrap(), Some(20));
    }
    local.result = "Rhino".into();
    db::save_local_recipe(&conn, &local).unwrap();
    assert_eq!(db::recipes(&conn).unwrap()["Rhino"].quantity, 2);
    local.quantity = 0;
    assert!(db::save_local_recipe(&conn, &local).is_err());
}
