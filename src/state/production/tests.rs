use super::*;

#[test]
fn an_empty_ledger_explains_that_no_cycles_have_finished() {
    assert_eq!(
        ProductionLedger::default().summary(),
        "Kiln ledger · no cycles sealed yet."
    );
}

#[test]
fn the_ledger_separates_recipe_counts_and_accumulates_quieting() {
    let mut ledger = ProductionLedger::default();
    ledger.record(ProductionRecipeKind::WardCharge, 1, 0.0);
    ledger.record(ProductionRecipeKind::HushAsh, 1, -5.0);

    assert_eq!(ledger.total_cycles, 2);
    assert_eq!(ledger.ward_cycles, 1);
    assert_eq!(ledger.hush_ash_cycles, 1);
    assert_eq!(ledger.wards_sealed, 2);
    assert_eq!(ledger.suspicion_quieted, 5.0);
    assert!(ledger.summary().contains("2 cycles"));
}
