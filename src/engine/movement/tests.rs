use super::*;
use crate::state::{Building, BuildingKind, GameSession};

#[test]
fn necromancer_order_names_the_obstruction() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.world.necromancer_position = TilePos::new(2, 2);
    for y in 0..session.world.height as i32 {
        session.world.buildings.push(Building {
            kind: BuildingKind::WorkShed,
            progress: 10.0,
            complete: true,
            position: TilePos::new(3, y),
            width: 1,
            height: 1,
        });
    }

    let result = request_necromancer_destination(&mut session, TilePos::new(5, 2));

    assert_eq!(
        result,
        Err("The necromancer cannot reach that tile: obstructions seal the way.".to_owned())
    );
}

#[test]
fn necromancer_simulation_reports_a_route_that_becomes_sealed() {
    let data = crate::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.world.necromancer_position = TilePos::new(2, 2);
    session.world.necromancer_destination = Some(TilePos::new(5, 2));
    for y in 0..session.world.height as i32 {
        session.world.buildings.push(Building {
            kind: BuildingKind::WorkShed,
            progress: 10.0,
            complete: true,
            position: TilePos::new(3, y),
            width: 1,
            height: 1,
        });
    }

    assert_eq!(
        simulate_necromancer(&mut session),
        Some("The necromancer's route failed: obstructions seal the way.".to_owned())
    );
    assert_eq!(session.world.necromancer_destination, None);
}
