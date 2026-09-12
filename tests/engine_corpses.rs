use tiny_necromancer::engine::corpses::*;
use tiny_necromancer::state::*;

#[test]
fn notable_corpse_is_the_brute_gate() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.bones = 30;
    session.economy.mana = 14;
    session.economy.corpses.push(Corpse {
        id: 1,
        integrity: 0.9,
        strength: 0.9,
        skill: 0.4,
        magical_residue: 0.8,
        cause_of_death: "test".to_owned(),
        quality: CorpseQuality::Notable,
    });
    assert!(can_raise(&session, &data, UndeadKind::BruteSkeleton).is_ok());
    assert!(can_raise(&session, &data, UndeadKind::Skeleton).is_ok());
}

#[test]
fn resurrection_spends_costs_and_adds_worker() {
    let data = tiny_necromancer::data::GameData::load().unwrap();
    let mut session = GameSession::new(&data.config);
    session.economy.corpses.push(Corpse {
        id: 1,
        integrity: 0.6,
        strength: 0.5,
        skill: 0.2,
        magical_residue: 0.5,
        cause_of_death: "test".to_owned(),
        quality: CorpseQuality::Sound,
    });
    let bones = session.economy.bones;
    raise(&mut session, &data, UndeadKind::Skeleton).unwrap();
    assert_eq!(session.workforce.workers.len(), 2);
    assert_eq!(session.economy.bones, bones - 18);
    assert!(session.economy.corpses.is_empty());
}
