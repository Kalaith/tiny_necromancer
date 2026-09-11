use super::*;

#[test]
fn broker_standing_advances_at_persistent_favor_thresholds() {
    let mut market = MarketState::default();
    assert_eq!(market.standing_label(), "Whisper");
    assert_eq!(market.next_standing_target(), Some(3));

    market.favor = 3;
    assert_eq!(market.standing_label(), "Acquainted");
    assert_eq!(market.next_standing_target(), Some(8));

    market.favor = 8;
    assert_eq!(market.standing_label(), "Trusted");
    assert_eq!(market.next_standing_target(), None);
}
