struct FlattenInvariant {
    file: &'static str,
    src: &'static str,
}

const INVARIANTS: &[FlattenInvariant] = &[
    FlattenInvariant {
        file: "accordion.rs",
        src: include_str!("../../src/components/accordion.rs"),
    },
    FlattenInvariant {
        file: "action_icon.rs",
        src: include_str!("../../src/components/action_icon.rs"),
    },
    FlattenInvariant {
        file: "alert.rs",
        src: include_str!("../../src/components/alert.rs"),
    },
    FlattenInvariant {
        file: "app_shell.rs",
        src: include_str!("../../src/components/app_shell.rs"),
    },
    FlattenInvariant {
        file: "badge.rs",
        src: include_str!("../../src/components/badge.rs"),
    },
    FlattenInvariant {
        file: "breadcrumbs.rs",
        src: include_str!("../../src/components/breadcrumbs.rs"),
    },
    FlattenInvariant {
        file: "button.rs",
        src: include_str!("../../src/components/button.rs"),
    },
    FlattenInvariant {
        file: "checkbox.rs",
        src: include_str!("../../src/components/checkbox.rs"),
    },
    FlattenInvariant {
        file: "chip.rs",
        src: include_str!("../../src/components/chip.rs"),
    },
    FlattenInvariant {
        file: "control.rs",
        src: include_str!("../../src/components/control.rs"),
    },
    FlattenInvariant {
        file: "divider.rs",
        src: include_str!("../../src/components/divider.rs"),
    },
    FlattenInvariant {
        file: "drawer.rs",
        src: include_str!("../../src/components/drawer.rs"),
    },
    FlattenInvariant {
        file: "field_variant.rs",
        src: include_str!("../../src/components/field_variant.rs"),
    },
    FlattenInvariant {
        file: "hovercard.rs",
        src: include_str!("../../src/components/hovercard.rs"),
    },
    FlattenInvariant {
        file: "icon.rs",
        src: include_str!("../../src/components/icon.rs"),
    },
    FlattenInvariant {
        file: "indicator.rs",
        src: include_str!("../../src/components/indicator.rs"),
    },
    FlattenInvariant {
        file: "input.rs",
        src: include_str!("../../src/components/input.rs"),
    },
    FlattenInvariant {
        file: "interaction_adapter.rs",
        src: include_str!("../../src/components/interaction_adapter.rs"),
    },
    FlattenInvariant {
        file: "layers.rs",
        src: include_str!("../../src/components/layers.rs"),
    },
    FlattenInvariant {
        file: "layout.rs",
        src: include_str!("../../src/components/layout.rs"),
    },
    FlattenInvariant {
        file: "loader.rs",
        src: include_str!("../../src/components/loader.rs"),
    },
    FlattenInvariant {
        file: "loading_overlay.rs",
        src: include_str!("../../src/components/loading_overlay.rs"),
    },
    FlattenInvariant {
        file: "markdown.rs",
        src: include_str!("../../src/components/markdown.rs"),
    },
    FlattenInvariant {
        file: "menu.rs",
        src: include_str!("../../src/components/menu.rs"),
    },
    FlattenInvariant {
        file: "menu_state.rs",
        src: include_str!("../../src/components/menu_state.rs"),
    },
    FlattenInvariant {
        file: "modal.rs",
        src: include_str!("../../src/components/modal.rs"),
    },
    FlattenInvariant {
        file: "number_input.rs",
        src: include_str!("../../src/components/number_input.rs"),
    },
    FlattenInvariant {
        file: "overlay.rs",
        src: include_str!("../../src/components/overlay.rs"),
    },
    FlattenInvariant {
        file: "pagination.rs",
        src: include_str!("../../src/components/pagination.rs"),
    },
    FlattenInvariant {
        file: "paper.rs",
        src: include_str!("../../src/components/paper.rs"),
    },
    FlattenInvariant {
        file: "popover.rs",
        src: include_str!("../../src/components/popover.rs"),
    },
    FlattenInvariant {
        file: "popup.rs",
        src: include_str!("../../src/components/popup.rs"),
    },
    FlattenInvariant {
        file: "popup_state.rs",
        src: include_str!("../../src/components/popup_state.rs"),
    },
    FlattenInvariant {
        file: "progress.rs",
        src: include_str!("../../src/components/progress.rs"),
    },
    FlattenInvariant {
        file: "radio.rs",
        src: include_str!("../../src/components/radio.rs"),
    },
    FlattenInvariant {
        file: "range_slider.rs",
        src: include_str!("../../src/components/range_slider.rs"),
    },
    FlattenInvariant {
        file: "rating.rs",
        src: include_str!("../../src/components/rating.rs"),
    },
    FlattenInvariant {
        file: "scroll_area.rs",
        src: include_str!("../../src/components/scroll_area.rs"),
    },
    FlattenInvariant {
        file: "segmented_control.rs",
        src: include_str!("../../src/components/segmented_control.rs"),
    },
    FlattenInvariant {
        file: "select.rs",
        src: include_str!("../../src/components/select.rs"),
    },
    FlattenInvariant {
        file: "select_state.rs",
        src: include_str!("../../src/components/select_state.rs"),
    },
    FlattenInvariant {
        file: "selection_state.rs",
        src: include_str!("../../src/components/selection_state.rs"),
    },
    FlattenInvariant {
        file: "slider.rs",
        src: include_str!("../../src/components/slider.rs"),
    },
    FlattenInvariant {
        file: "slider_axis.rs",
        src: include_str!("../../src/components/slider_axis.rs"),
    },
    FlattenInvariant {
        file: "stepper.rs",
        src: include_str!("../../src/components/stepper.rs"),
    },
    FlattenInvariant {
        file: "switch.rs",
        src: include_str!("../../src/components/switch.rs"),
    },
    FlattenInvariant {
        file: "table.rs",
        src: include_str!("../../src/components/table.rs"),
    },
    FlattenInvariant {
        file: "table_state.rs",
        src: include_str!("../../src/components/table_state.rs"),
    },
    FlattenInvariant {
        file: "tabs.rs",
        src: include_str!("../../src/components/tabs.rs"),
    },
    FlattenInvariant {
        file: "text.rs",
        src: include_str!("../../src/components/text.rs"),
    },
    FlattenInvariant {
        file: "text_input_actions.rs",
        src: include_str!("../../src/components/text_input_actions.rs"),
    },
    FlattenInvariant {
        file: "text_input_state.rs",
        src: include_str!("../../src/components/text_input_state.rs"),
    },
    FlattenInvariant {
        file: "textarea.rs",
        src: include_str!("../../src/components/textarea.rs"),
    },
    FlattenInvariant {
        file: "timeline.rs",
        src: include_str!("../../src/components/timeline.rs"),
    },
    FlattenInvariant {
        file: "title.rs",
        src: include_str!("../../src/components/title.rs"),
    },
    FlattenInvariant {
        file: "title_bar.rs",
        src: include_str!("../../src/components/title_bar.rs"),
    },
    FlattenInvariant {
        file: "toggle.rs",
        src: include_str!("../../src/components/toggle.rs"),
    },
    FlattenInvariant {
        file: "tooltip.rs",
        src: include_str!("../../src/components/tooltip.rs"),
    },
    FlattenInvariant {
        file: "transition.rs",
        src: include_str!("../../src/components/transition.rs"),
    },
    FlattenInvariant {
        file: "tree.rs",
        src: include_str!("../../src/components/tree.rs"),
    },
    FlattenInvariant {
        file: "tree_state.rs",
        src: include_str!("../../src/components/tree_state.rs"),
    },
    FlattenInvariant {
        file: "utils.rs",
        src: include_str!("../../src/components/utils.rs"),
    },
];

#[test]
fn no_placeholder_anyelement_fallbacks_in_component_sources() {
    for it in INVARIANTS {
        let contains_placeholder_fallback = it
            .src
            .contains("let mut close_action: AnyElement = div().into_any_element()");
        assert!(
            !contains_placeholder_fallback,
            "{} still uses empty AnyElement placeholder fallback",
            it.file
        );
    }
}

#[test]
fn label_block_helpers_use_sparse_option_shape() {
    for it in INVARIANTS {
        assert!(
            !it.src
                .contains("fn render_label_block(&self) -> AnyElement"),
            "{} still uses non-sparse render_label_block return type",
            it.file
        );
    }
}

#[test]
fn no_empty_placeholder_cell_or_grid_fillers() {
    let table_src = include_str!("../../src/components/table.rs");
    assert!(
        !table_src.contains("cell = cell.child(div().child(\"\"));"),
        "table.rs should not use empty div placeholder for missing cells",
    );

    let layout_src = include_str!("../../src/components/layout.rs");
    assert!(
        !layout_src.contains("current_row.push(div().w_full().h_full().into_any_element());"),
        "layout.rs should not push synthetic placeholder cells",
    );
}

#[test]
fn popup_trigger_defaults_do_not_wrap_text_in_extra_div() {
    let menu_src = include_str!("../../src/components/menu.rs");
    assert!(
        !menu_src.contains(".unwrap_or_else(|| div().child(\"Menu\").into_any_element())"),
        "menu.rs should not use wrapped div fallback for trigger text",
    );

    let popover_src = include_str!("../../src/components/popover.rs");
    assert!(
        !popover_src.contains(".unwrap_or_else(|| div().child(\"Open\").into_any_element())"),
        "popover.rs should not use wrapped div fallback for trigger text",
    );

    let tooltip_src = include_str!("../../src/components/tooltip.rs");
    assert!(
        !tooltip_src.contains(".unwrap_or_else(|| div().child(\"target\").into_any_element())"),
        "tooltip.rs should not use wrapped div fallback for trigger text",
    );

    let hovercard_src = include_str!("../../src/components/hovercard.rs");
    assert!(
        !hovercard_src.contains(".unwrap_or_else(|| div().child(\"target\").into_any_element())"),
        "hovercard.rs should not use wrapped div fallback for trigger text",
    );
}

#[test]
fn stepper_label_block_does_not_use_optional_wrapper_div() {
    let stepper_src = include_str!("../../src/components/stepper.rs");
    assert!(
        !stepper_src.contains("Stack::vertical().gap(tokens.text_gap).child(div().children("),
        "stepper.rs should not wrap optional label in an extra div().children(...) layer",
    );
}
