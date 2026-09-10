use super::*;

#[test]
fn narrow_navigation_keeps_seven_touch_targets_inside_the_canvas() {
    let width = 360.0;
    let button_width = compact_nav_button_width(width);
    let occupied = COMPACT_NAV_MARGIN
        + COMPACT_NAV_GAP * (COMPACT_NAV_ENTRIES - 1.0)
        + button_width * COMPACT_NAV_ENTRIES;

    assert!(button_width >= COMPACT_NAV_MIN_BUTTON);
    assert!(occupied <= width);
    assert_eq!(compact_nav_text_size(button_width), 9.0);
}

#[test]
fn wider_compact_navigation_keeps_full_labels() {
    let button_width = compact_nav_button_width(800.0);

    assert!(button_width >= 56.0);
    assert_eq!(compact_nav_text_size(button_width), 12.0);
}
