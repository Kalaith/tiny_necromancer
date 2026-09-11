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
    assert_eq!(market.standing_progress(), 1.0);

    market.favor = 2;
    assert!((market.standing_progress() - (2.0 / 3.0)).abs() < 0.001);
    assert!((market.refresh_interval() - 30.0).abs() < 0.001);

    market.favor = 3;
    assert!((market.refresh_interval() - 27.0).abs() < 0.001);
    market.favor = 8;
    assert!((market.refresh_interval() - 24.0).abs() < 0.001);
}

#[test]
fn older_market_saves_infer_favor_from_completed_trades() {
    let mut market = MarketState {
        completed_trades: 5,
        ..MarketState::default()
    };
    market.normalize();
    assert_eq!(market.favor, 5);
    assert_eq!(market.standing_label(), "Acquainted");
}
