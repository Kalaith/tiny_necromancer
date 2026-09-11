use super::super::economy::LooseResourcePile;
use super::super::{GameSession, ResourceKind};
use macroquad_toolkit::grid::TilePos;

#[test]
fn loose_piles_merge_and_refresh_legacy_totals() {
    let data = crate::data::GameData::load().unwrap();
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
    let data = crate::data::GameData::load().unwrap();
    let mut economy = GameSession::new(&data.config).economy;
    let source = TilePos::new(2, 2);
    economy.loose_bones = 8;
    economy.loose_bones_source = Some(source);

    economy.normalize_loose_piles();

    assert_eq!(
        economy.loose_piles(ResourceKind::Bones),
        vec![LooseResourcePile {
            position: source,
            amount: 8
        }]
    );
}
