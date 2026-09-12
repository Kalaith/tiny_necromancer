use macroquad::prelude::Rect;
use tiny_necromancer::ui::*;

#[test]
fn desktop_layout_preserves_the_world_first_canvas() {
    let layout = UiLayout::for_dimensions(1280.0, 720.0, Panel::None);

    assert!(!layout.compact);
    assert_eq!(layout.world_rect, Rect::new(0.0, 0.0, 1280.0, 720.0));
    assert_eq!(layout.sheet_rect, Rect::new(0.0, 0.0, 0.0, 0.0));
}

#[test]
fn compact_layout_keeps_world_and_sheet_inside_the_viewport() {
    let layout = UiLayout::for_dimensions(800.0, 600.0, Panel::None);

    assert!(layout.compact);
    assert!(layout.world_rect.bottom() <= 600.0);
    assert!(layout.sheet_rect.y >= layout.world_rect.bottom());
    assert_eq!(layout.sheet_rect.right(), 800.0);
    assert_eq!(layout.sheet_rect.bottom(), 600.0);
    assert!(layout.sheet_rect.h >= 272.0);
}

#[test]
fn management_sheets_grow_for_touch_sized_lists() {
    let layout = UiLayout::for_dimensions(800.0, 600.0, Panel::Orders);

    assert!(
        layout.sheet_rect.h
            > UiLayout::for_dimensions(800.0, 600.0, Panel::None)
                .sheet_rect
                .h
    );
    assert!(layout.world_rect.h >= 128.0);
}

#[test]
fn zone_editor_sheet_fits_storage_guidance_and_touch_controls() {
    let layout = UiLayout::for_dimensions(800.0, 600.0, Panel::Zones);

    assert!(layout.sheet_rect.h >= 360.0);
    assert!(layout.sheet_rect.bottom() <= 600.0);
    assert!(layout.world_rect.h >= 128.0);
}

#[test]
fn compact_camera_controls_are_touch_sized_and_inside_the_world() {
    let layout = UiLayout::for_dimensions(360.0, 640.0, Panel::None);

    for control in layout.compact_camera_controls() {
        assert!(control.w >= 44.0 && control.h >= 44.0);
        assert!(control.y >= layout.world_rect.y);
        assert!(control.bottom() <= layout.world_rect.bottom());
    }
}
