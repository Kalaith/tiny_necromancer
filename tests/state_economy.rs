use macroquad_toolkit::grid::TilePos;
use tiny_necromancer::state::*;

#[test]
fn loose_piles_merge_and_refresh_legacy_totals() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut economy = GameSession::new(&data.config).economy;
    let first = TilePos::new(2, 2);
    let second = TilePos::new(0, 0);

    economy.add_loose(ResourceKind::Wood, first, 5);
    economy.add_loose(ResourceKind::Wood, first, 2);
    economy.add_loose(ResourceKind::Wood, second, 4);

    assert_eq!(economy.wood, 12);
    assert_eq!(economy.loose_wood, 11);
    assert_eq!(economy.loose_amount_at(ResourceKind::Wood, first), 7);
    assert_eq!(economy.loose_amount_at(ResourceKind::Wood, second), 4);
    assert_eq!(economy.loose_wood_source, Some(second));

    assert_eq!(economy.take_loose(ResourceKind::Wood, first, 6), 6);
    assert_eq!(economy.loose_wood, 5);
    assert_eq!(economy.loose_amount_at(ResourceKind::Wood, first), 1);
}

#[test]
fn old_scalar_loose_material_becomes_one_compatibility_pile() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut economy = GameSession::new(&data.config).economy;
    let source = TilePos::new(2, 2);
    economy.loose_bones = 8;
    economy.loose_bones_source = Some(source);

    economy.normalize_loose_piles();

    assert_eq!(
        economy.loose_piles(ResourceKind::Bones, source),
        vec![LooseResourcePile {
            position: source,
            amount: 8
        }]
    );
}

#[test]
fn old_scalar_save_infers_a_missing_bone_source_from_dug_ground() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    let source = session.world.plots[0].position;
    session.world.plots[0].status = PlotStatus::Dug;
    session.economy.loose_bones = 8;
    session.economy.loose_bones_source = None;
    let mut value = serde_json::to_value(session.to_save(&data.config.version)).unwrap();
    value
        .get_mut("economy")
        .and_then(serde_json::Value::as_object_mut)
        .expect("economy object")
        .remove("loose_bones_piles");

    let restored = GameSession::from_save(serde_json::from_value(value).unwrap(), &data.config);

    assert_eq!(
        restored.economy.loose_piles(ResourceKind::Bones, source),
        vec![LooseResourcePile {
            position: source,
            amount: 8
        }]
    );
}

#[test]
fn normalization_compacts_duplicate_and_empty_source_piles() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut economy = GameSession::new(&data.config).economy;
    let first = TilePos::new(2, 2);
    let second = TilePos::new(0, 0);
    economy.loose_wood_piles = vec![
        LooseResourcePile {
            position: first,
            amount: 3,
        },
        LooseResourcePile {
            position: first,
            amount: 2,
        },
        LooseResourcePile {
            position: second,
            amount: 0,
        },
    ];
    economy.loose_wood = 99;

    economy.normalize_loose_piles();

    assert_eq!(
        economy.loose_piles(ResourceKind::Wood, first),
        vec![LooseResourcePile {
            position: first,
            amount: 5
        }]
    );
    assert_eq!(economy.loose_wood, 5);
    assert_eq!(economy.loose_wood_source, Some(first));
}

#[test]
fn storage_space_clamps_material_deposits_at_capacity() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut economy = GameSession::new(&data.config).economy;
    economy.storage_capacity = 10;
    economy.bones = 8;
    economy.wood = 1;

    assert_eq!(economy.storage_space(10), 1);
    assert_eq!(economy.store_resource(ResourceKind::Wood, 4, 10), 1);
    assert_eq!(economy.stored_materials(), 10);
    assert_eq!(economy.wood, 2);
    assert_eq!(economy.store_resource(ResourceKind::Bones, 2, 10), 0);
}
