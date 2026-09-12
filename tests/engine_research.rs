use tiny_necromancer::engine::progression::{
    advance_construction, queue_building, start_research, study_bindings,
};
use tiny_necromancer::state::{BuildingKind, GameSession, Technology};

fn session_with_restored_shed(data: &tiny_necromancer::data::GameData) -> GameSession {
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 100;
    session.economy.mana = 100;
    session.economy.wood = 100;
    queue_building(&mut session, data, BuildingKind::WorkShed).unwrap();
    advance_construction(&mut session, data, 10.0);
    session
}

#[test]
fn bindings_can_only_begin_from_the_shed_and_spend_the_authored_cost() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = session_with_restored_shed(&data);
    let cost = Technology::BindingRoutines.cost(&data.config);
    let bones_before = session.economy.bones;
    let mana_before = session.economy.mana;

    assert!(start_research(&mut session, &data, Technology::BindingRoutines).is_err());
    study_bindings(&mut session, &data).unwrap();

    assert_eq!(session.economy.bones, bones_before - cost.bones);
    assert_eq!(session.economy.mana, mana_before - cost.mana);
    assert_eq!(session.research.current, Some(Technology::BindingRoutines));
}

#[test]
fn research_screen_projects_spend_cost_after_their_prerequisite() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = session_with_restored_shed(&data);
    session.research.completed.push(Technology::BindingRoutines);
    let cost = Technology::Gravecraft.cost(&data.config);
    let bones_before = session.economy.bones;
    let mana_before = session.economy.mana;

    start_research(&mut session, &data, Technology::Gravecraft).unwrap();

    assert_eq!(session.economy.bones, bones_before - cost.bones);
    assert_eq!(session.economy.mana, mana_before - cost.mana);
    assert_eq!(session.research.current, Some(Technology::Gravecraft));
}

#[test]
fn research_does_not_start_or_spend_when_the_cost_is_unaffordable() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = session_with_restored_shed(&data);
    let cost = Technology::BindingRoutines.cost(&data.config);
    session.economy.bones = cost.bones - 1;
    session.economy.mana = cost.mana;

    assert!(study_bindings(&mut session, &data).is_err());
    assert_eq!(session.economy.bones, cost.bones - 1);
    assert_eq!(session.economy.mana, cost.mana);
    assert_eq!(session.research.current, None);
}
