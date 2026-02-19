use std::sync::{LazyLock, Mutex, MutexGuard};

use super::{
    control, menu_state, popup, popup_state, select_state, selection_state, slider_axis,
    table_state, text_input_state, tree_state,
};

static STATE_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

struct StateTestGuard {
    _lock: MutexGuard<'static, ()>,
}

fn guard() -> StateTestGuard {
    let lock = match STATE_TEST_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    control::clear_all();
    StateTestGuard { _lock: lock }
}

impl Drop for StateTestGuard {
    fn drop(&mut self) {
        control::clear_all();
    }
}

fn base_table_input<'a>(id: &'a str) -> table_state::TableStateInput<'a> {
    table_state::TableStateInput {
        id,
        total_rows: 100,
        page_size: 20,
        page_size_options: vec![10, 20, 50],
        pagination_enabled: true,
        page_controlled: false,
        page: None,
        default_page: 1,
        max_height_px: Some(320.0),
        sticky_header: true,
        has_headers: true,
        virtual_window: None,
        auto_virtualization: true,
        virtualization_min_rows: 30,
        virtualization_overscan_rows: 4,
        virtual_row_height_px: None,
        default_row_height_px: 28.0,
        line_thickness_px: 1.0,
        sticky_header_reserved_height_px: 38.0,
        min_scroll_height_px: 80.0,
    }
}

fn demo_visible_nodes() -> Vec<tree_state::TreeVisibleNode> {
    vec![
        tree_state::TreeVisibleNode {
            value: "root".into(),
            parent: None,
            label: Some("Root".into()),
            depth: 0,
            path: "0".into(),
            disabled: false,
            has_children: true,
            first_child: Some("child-a".into()),
        },
        tree_state::TreeVisibleNode {
            value: "child-a".into(),
            parent: Some("root".into()),
            label: Some("Child A".into()),
            depth: 1,
            path: "0-0".into(),
            disabled: false,
            has_children: false,
            first_child: None,
        },
        tree_state::TreeVisibleNode {
            value: "child-b".into(),
            parent: Some("root".into()),
            label: Some("Child B".into()),
            depth: 1,
            path: "0-1".into(),
            disabled: true,
            has_children: false,
            first_child: None,
        },
    ]
}

#[test]
fn popup_state_resolve_supports_controlled_and_uncontrolled_modes() {
    let _guard = guard();

    let uncontrolled = popup::PopupState::resolve("popup-u", None, true);
    assert!(uncontrolled.opened);
    assert!(!uncontrolled.controlled);

    let controlled = popup::PopupState::resolve("popup-c", Some(false), true);
    assert!(!controlled.opened);
    assert!(controlled.controlled);
}

#[test]
fn popup_state_value_disables_opened_flag_when_disabled() {
    let _guard = guard();

    let resolved = popup_state::PopupStateValue::resolve(popup_state::PopupStateInput {
        id: "popup-disabled",
        opened: Some(true),
        default_opened: true,
        disabled: true,
    });

    assert!(!resolved.opened);
    assert!(resolved.controlled);
}

#[test]
fn popup_apply_opened_respects_controlled_mode() {
    let _guard = guard();

    assert!(!popup_state::apply_opened("popup-apply", true, true));
    assert!(popup_state::apply_opened("popup-apply", false, true));
    assert!(popup::PopupState::resolve("popup-apply", None, false).opened);
}

#[test]
fn menu_state_resolves_dropdown_width_with_fallback_and_minimum() {
    let _guard = guard();

    let state = menu_state::MenuState::resolve(menu_state::MenuStateInput {
        id: "menu-width",
        opened: None,
        default_opened: false,
        disabled: false,
        dropdown_width_fallback: 96.0,
        dropdown_min_width: 120.0,
    });
    assert_eq!(state.dropdown_width_px, 120.0);

    menu_state::set_dropdown_width("menu-width", 176.0);
    let resolved = menu_state::MenuState::resolve(menu_state::MenuStateInput {
        id: "menu-width",
        opened: None,
        default_opened: false,
        disabled: false,
        dropdown_width_fallback: 96.0,
        dropdown_min_width: 120.0,
    });
    assert_eq!(resolved.dropdown_width_px, 176.0);
}

#[test]
fn menu_state_item_click_respects_close_flag() {
    let _guard = guard();

    assert!(menu_state::apply_opened("menu-close", false, true));
    assert!(!menu_state::on_item_click("menu-close", false, false));
    assert!(
        menu_state::MenuState::resolve(menu_state::MenuStateInput {
            id: "menu-close",
            opened: None,
            default_opened: false,
            disabled: false,
            dropdown_width_fallback: 100.0,
            dropdown_min_width: 100.0,
        })
        .opened
    );

    assert!(menu_state::on_item_click("menu-close", false, true));
    assert!(
        !menu_state::MenuState::resolve(menu_state::MenuStateInput {
            id: "menu-close",
            opened: None,
            default_opened: false,
            disabled: false,
            dropdown_width_fallback: 100.0,
            dropdown_min_width: 100.0,
        })
        .opened
    );
}

#[test]
fn selection_state_optional_text_and_list_respect_controlled_mode() {
    let _guard = guard();

    assert!(selection_state::apply_optional_text(
        "sel-opt",
        "value",
        false,
        Some("abc".into())
    ));
    assert_eq!(
        selection_state::resolve_optional_text("sel-opt", "value", false, None, None),
        Some("abc".into())
    );
    assert!(!selection_state::apply_optional_text(
        "sel-opt",
        "value",
        true,
        Some("def".into())
    ));
    assert_eq!(
        selection_state::resolve_optional_text("sel-opt", "value", false, None, None),
        Some("abc".into())
    );

    assert!(selection_state::apply_list(
        "sel-list",
        "values",
        false,
        vec!["a".into(), "b".into()]
    ));
    assert_eq!(
        selection_state::resolve_list("sel-list", "values", false, vec![], vec![]),
        vec!["a", "b"]
    );
    assert!(!selection_state::apply_list(
        "sel-list",
        "values",
        true,
        vec!["c".into()]
    ));
    assert_eq!(
        selection_state::resolve_list("sel-list", "values", false, vec![], vec![]),
        vec!["a", "b"]
    );
}

#[test]
fn selection_state_usize_family_behaves_consistently() {
    let _guard = guard();

    assert!(selection_state::apply_usize("sel-size", "page", false, 3));
    assert_eq!(
        selection_state::resolve_usize("sel-size", "page", false, 0, 1),
        3
    );
    assert!(!selection_state::apply_usize("sel-size", "page", true, 9));
    assert_eq!(
        selection_state::resolve_usize("sel-size", "page", false, 0, 1),
        3
    );

    assert!(selection_state::apply_optional_usize(
        "sel-size",
        "cursor",
        false,
        Some(7)
    ));
    assert_eq!(
        selection_state::resolve_optional_usize("sel-size", "cursor", None, None),
        Some(7)
    );
    assert!(!selection_state::apply_optional_usize(
        "sel-size",
        "cursor",
        true,
        Some(9)
    ));
    assert_eq!(
        selection_state::resolve_optional_usize("sel-size", "cursor", None, None),
        Some(7)
    );
}

#[test]
fn select_state_toggled_values_are_unique_and_sorted() {
    let _guard = guard();

    let values = vec!["b".to_string(), "a".to_string()];
    let added = select_state::toggled_values(&values, "c");
    assert_eq!(added, vec!["a", "b", "c"]);
    let removed = select_state::toggled_values(&added, "b");
    assert_eq!(removed, vec!["a", "c"]);
}

#[test]
fn select_state_single_commit_updates_value_and_closes_popup() {
    let _guard = guard();

    assert!(select_state::apply_opened("select-commit", false, true));
    let refreshed =
        select_state::apply_single_option_commit("select-commit", false, false, "hello");
    assert!(refreshed);

    assert_eq!(
        select_state::resolve_single_value("select-commit", false, None, None),
        Some("hello".into())
    );
    let popup = popup::PopupState::resolve("select-commit", None, false);
    assert!(!popup.opened);
}

#[test]
fn select_state_dropdown_width_uses_fallback_before_measurement() {
    let _guard = guard();

    assert_eq!(
        select_state::dropdown_width_px("select-width", 144.0),
        144.0
    );
    select_state::set_dropdown_width("select-width", 188.0);
    assert_eq!(
        select_state::dropdown_width_px("select-width", 144.0),
        188.0
    );
}

#[test]
fn table_state_resolve_clamps_page_to_valid_range() {
    let _guard = guard();

    let mut input = base_table_input("table-page");
    input.total_rows = 95;
    input.default_page = 999;
    input.page = Some(999);
    input.page_controlled = true;
    let state = table_state::TableState::resolve(input);

    assert_eq!(state.page_count, 5);
    assert_eq!(state.resolved_page, 5);
}

#[test]
fn table_state_auto_virtualization_works_with_scroll_window() {
    let _guard = guard();

    let mut input = base_table_input("table-virtual");
    input.pagination_enabled = false;
    input.virtual_window = None;
    input.total_rows = 500;
    input.max_height_px = Some(240.0);
    input.virtualization_min_rows = 50;
    let state = table_state::TableState::resolve(input);

    assert!(state.auto_virtualization_enabled);
    assert!(state.window_count > 0);
    assert!(state.max_scroll_y >= 0.0);
    assert!(state.top_spacer_height() >= 0.0);
    assert!(state.bottom_spacer_height(500, 20) >= 0.0);
}

#[test]
fn table_state_page_and_size_callbacks_update_state_store() {
    let _guard = guard();

    assert!(table_state::on_page_change("table-cb", false, 3));
    assert_eq!(control::usize_state("table-cb", "page", None, 1), 3);
    assert!(!table_state::on_page_change("table-cb", true, 4));
    assert_eq!(control::usize_state("table-cb", "page", None, 1), 3);

    table_state::on_page_size_change("table-cb", 50);
    assert_eq!(control::usize_state("table-cb", "page-size", None, 1), 50);
    assert_eq!(control::usize_state("table-cb", "page", None, 99), 1);
}

#[test]
fn table_state_row_measurement_and_virtual_scroll_have_thresholds() {
    let _guard = guard();

    assert!(table_state::on_row_height_measured("table-measure", 24.0));
    assert!(!table_state::on_row_height_measured("table-measure", 24.3));
    assert!(table_state::on_row_height_measured("table-measure", 25.0));

    assert!(!table_state::on_virtual_scroll(
        "table-scroll",
        48.0,
        20.0,
        2
    ));
    assert!(!table_state::on_virtual_scroll(
        "table-scroll",
        48.1,
        20.0,
        2
    ));
    assert!(table_state::on_virtual_scroll(
        "table-scroll",
        90.0,
        20.0,
        2
    ));
}

#[test]
fn tree_state_toggle_and_key_navigation_follow_expected_rules() {
    let _guard = guard();
    let nodes = demo_visible_nodes();

    let toggled = tree_state::toggled_values(vec!["root".into()], "root");
    assert!(toggled.is_empty());
    let toggled = tree_state::toggled_values(vec![], "root");
    assert_eq!(toggled, vec!["root"]);

    let down = tree_state::key_transition("down", None, &nodes, &[]);
    assert_eq!(down.next_selected, Some("root".into()));

    let right_expand = tree_state::key_transition("right", Some("root"), &nodes, &[]);
    assert_eq!(right_expand.next_expanded, Some(vec!["root".into()]));

    let right_to_child =
        tree_state::key_transition("right", Some("root"), &nodes, &["root".into()]);
    assert_eq!(right_to_child.next_selected, Some("child-a".into()));

    let left_collapse = tree_state::key_transition("left", Some("root"), &nodes, &["root".into()]);
    assert_eq!(left_collapse.next_expanded, Some(Vec::<String>::new()));

    let left_to_parent = tree_state::key_transition("left", Some("child-a"), &nodes, &[]);
    assert_eq!(left_to_parent.next_selected, Some("root".into()));

    let home = tree_state::key_transition("home", Some("child-a"), &nodes, &[]);
    assert_eq!(home.next_selected, Some("root".into()));
}

#[test]
fn text_input_state_handles_selection_and_unicode_correctly() {
    let _guard = guard();

    let mut state = text_input_state::InputState::new("ab中d", 10, 0, Some((3, 1)));
    assert_eq!(state.len(), 4);
    assert_eq!(state.selection, Some((1, 3)));
    assert_eq!(state.selected_text(), "b中");

    state.move_left(false);
    assert_eq!(state.caret, 1);
    assert!(state.selection.is_none());

    state.set_selection_from_anchor(1, 3);
    assert!(state.delete_backward());
    assert_eq!(state.value, "ad");
    assert_eq!(state.caret, 1);

    assert!(state.insert_text("XYZ"));
    assert_eq!(state.value, "aXYZd");
    assert_eq!(
        text_input_state::InputState::byte_index_at_char("a中b", 2),
        "a中".len()
    );

    assert!(state.clamp_to_max_length(Some(3)));
    assert_eq!(state.value, "aXY");
}

#[test]
fn slider_axis_math_functions_are_stable() {
    let _guard = guard();

    let normalized = slider_axis::normalize(0.0, 10.0, 2.0, 5.1);
    assert_eq!(normalized, 6.0);

    let pair = slider_axis::normalize_pair(0.0, 10.0, 1.0, 9.2, 2.2);
    assert_eq!(pair, (2.0, 9.0));

    let ratio = slider_axis::ratio(0.0, 100.0, 25.0);
    assert!((ratio - 0.25).abs() < 0.0001);

    let horizontal =
        slider_axis::value_from_local(slider_axis::SliderAxis::Horizontal, 25.0, 100.0, 0.0, 10.0);
    let vertical =
        slider_axis::value_from_local(slider_axis::SliderAxis::Vertical, 25.0, 100.0, 0.0, 10.0);
    assert!((horizontal - 2.5).abs() < 0.0001);
    assert!((vertical - 7.5).abs() < 0.0001);

    assert_eq!(
        slider_axis::thumb_offset(slider_axis::SliderAxis::Horizontal, 100.0, 20.0, 0.5),
        40.0
    );
    assert_eq!(
        slider_axis::thumb_offset(slider_axis::SliderAxis::Vertical, 100.0, 20.0, 0.5),
        40.0
    );
}

#[test]
fn slider_axis_geometry_store_and_restore_round_trips() {
    let _guard = guard();

    slider_axis::RailGeometry::store("rail-geom", 10.0, 12.0, 0.0, -8.0);
    let geom = slider_axis::RailGeometry::from_state("rail-geom", 200.0, 24.0);
    assert_eq!(geom.origin_x, 10.0);
    assert_eq!(geom.origin_y, 12.0);
    assert_eq!(geom.width, 1.0);
    assert_eq!(geom.height, 1.0);
}
