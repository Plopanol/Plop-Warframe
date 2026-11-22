use plop_warframe::{
    db,
    recipes::{Ingredient, Recipe},
    source::{BlueprintSource, DropSource},
};

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
