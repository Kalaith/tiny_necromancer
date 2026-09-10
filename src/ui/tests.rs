use super::*;

#[test]
fn world_grid_rect_is_large_enough_for_a_touch_first_map() {
    let rect = world_grid_rect();
    assert!(rect.w >= 700.0);
    assert!(rect.h >= 400.0);
}

#[test]
fn action_names_are_visible_and_stable() {
    assert_eq!(JobKind::Dig.label(), "Dig");
    assert_eq!(JobKind::Guard.label(), "Guard");
    assert_eq!(JobKind::Refine.label(), "Refine");
    assert_eq!(UndeadKind::BruteSkeleton.id(), "brute_skeleton");
}

#[test]
fn recovery_controls_have_touch_sized_targets() {
    let pause = super::components::pause_control_rect();
    let placement_cancel = super::components::placement_cancel_rect();

    assert!(pause.w >= 44.0 && pause.h >= 44.0);
    assert!(placement_cancel.w >= 44.0 && placement_cancel.h >= 44.0);
}

#[test]
fn domain_overlays_keep_zones_visible_and_toggle_independently() {
    let mut overlays = DomainOverlays::default();
    assert!(overlays.zones);
    assert!(!overlays.routes);
    assert!(!overlays.pressure);

    overlays.toggle(DomainOverlay::Routes);
    overlays.toggle(DomainOverlay::Pressure);
    overlays.toggle(DomainOverlay::Zones);

    assert!(!overlays.zones);
    assert!(overlays.routes);
    assert!(overlays.pressure);
}
