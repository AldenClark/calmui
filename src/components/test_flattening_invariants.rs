struct FlattenInvariant {
    file: &'static str,
    src: &'static str,
}

const INVARIANTS: &[FlattenInvariant] = &[
    FlattenInvariant {
        file: "accordion.rs",
        src: include_str!("accordion.rs"),
    },
    FlattenInvariant {
        file: "action_icon.rs",
        src: include_str!("action_icon.rs"),
    },
    FlattenInvariant {
        file: "alert.rs",
        src: include_str!("alert.rs"),
    },
    FlattenInvariant {
        file: "app_shell.rs",
        src: include_str!("app_shell.rs"),
    },
    FlattenInvariant {
        file: "badge.rs",
        src: include_str!("badge.rs"),
    },
    FlattenInvariant {
        file: "breadcrumbs.rs",
        src: include_str!("breadcrumbs.rs"),
    },
    FlattenInvariant {
        file: "button.rs",
        src: include_str!("button.rs"),
    },
    FlattenInvariant {
        file: "checkbox.rs",
        src: include_str!("checkbox.rs"),
    },
    FlattenInvariant {
        file: "chip.rs",
        src: include_str!("chip.rs"),
    },
    FlattenInvariant {
        file: "control.rs",
        src: include_str!("control.rs"),
    },
    FlattenInvariant {
        file: "divider.rs",
        src: include_str!("divider.rs"),
    },
    FlattenInvariant {
        file: "drawer.rs",
        src: include_str!("drawer.rs"),
    },
    FlattenInvariant {
        file: "field_variant.rs",
        src: include_str!("field_variant.rs"),
    },
    FlattenInvariant {
        file: "hovercard.rs",
        src: include_str!("hovercard.rs"),
    },
    FlattenInvariant {
        file: "icon.rs",
        src: include_str!("icon.rs"),
    },
    FlattenInvariant {
        file: "indicator.rs",
        src: include_str!("indicator.rs"),
    },
    FlattenInvariant {
        file: "input.rs",
        src: include_str!("input.rs"),
    },
    FlattenInvariant {
        file: "interaction_adapter.rs",
        src: include_str!("interaction_adapter.rs"),
    },
    FlattenInvariant {
        file: "layers.rs",
        src: include_str!("layers.rs"),
    },
    FlattenInvariant {
        file: "layout.rs",
        src: include_str!("layout.rs"),
    },
    FlattenInvariant {
        file: "loader.rs",
        src: include_str!("loader.rs"),
    },
    FlattenInvariant {
        file: "loading_overlay.rs",
        src: include_str!("loading_overlay.rs"),
    },
    FlattenInvariant {
        file: "markdown.rs",
        src: include_str!("markdown.rs"),
    },
    FlattenInvariant {
        file: "menu.rs",
        src: include_str!("menu.rs"),
    },
    FlattenInvariant {
        file: "menu_state.rs",
        src: include_str!("menu_state.rs"),
    },
    FlattenInvariant {
        file: "modal.rs",
        src: include_str!("modal.rs"),
    },
    FlattenInvariant {
        file: "number_input.rs",
        src: include_str!("number_input.rs"),
    },
    FlattenInvariant {
        file: "overlay.rs",
        src: include_str!("overlay.rs"),
    },
    FlattenInvariant {
        file: "pagination.rs",
        src: include_str!("pagination.rs"),
    },
    FlattenInvariant {
        file: "paper.rs",
        src: include_str!("paper.rs"),
    },
    FlattenInvariant {
        file: "popover.rs",
        src: include_str!("popover.rs"),
    },
    FlattenInvariant {
        file: "popup.rs",
        src: include_str!("popup.rs"),
    },
    FlattenInvariant {
        file: "popup_state.rs",
        src: include_str!("popup_state.rs"),
    },
    FlattenInvariant {
        file: "progress.rs",
        src: include_str!("progress.rs"),
    },
    FlattenInvariant {
        file: "radio.rs",
        src: include_str!("radio.rs"),
    },
    FlattenInvariant {
        file: "range_slider.rs",
        src: include_str!("range_slider.rs"),
    },
    FlattenInvariant {
        file: "rating.rs",
        src: include_str!("rating.rs"),
    },
    FlattenInvariant {
        file: "scroll_area.rs",
        src: include_str!("scroll_area.rs"),
    },
    FlattenInvariant {
        file: "segmented_control.rs",
        src: include_str!("segmented_control.rs"),
    },
    FlattenInvariant {
        file: "select.rs",
        src: include_str!("select.rs"),
    },
    FlattenInvariant {
        file: "select_state.rs",
        src: include_str!("select_state.rs"),
    },
    FlattenInvariant {
        file: "selection_state.rs",
        src: include_str!("selection_state.rs"),
    },
    FlattenInvariant {
        file: "slider.rs",
        src: include_str!("slider.rs"),
    },
    FlattenInvariant {
        file: "slider_axis.rs",
        src: include_str!("slider_axis.rs"),
    },
    FlattenInvariant {
        file: "stepper.rs",
        src: include_str!("stepper.rs"),
    },
    FlattenInvariant {
        file: "switch.rs",
        src: include_str!("switch.rs"),
    },
    FlattenInvariant {
        file: "table.rs",
        src: include_str!("table.rs"),
    },
    FlattenInvariant {
        file: "table_state.rs",
        src: include_str!("table_state.rs"),
    },
    FlattenInvariant {
        file: "tabs.rs",
        src: include_str!("tabs.rs"),
    },
    FlattenInvariant {
        file: "text.rs",
        src: include_str!("text.rs"),
    },
    FlattenInvariant {
        file: "text_input_actions.rs",
        src: include_str!("text_input_actions.rs"),
    },
    FlattenInvariant {
        file: "text_input_state.rs",
        src: include_str!("text_input_state.rs"),
    },
    FlattenInvariant {
        file: "textarea.rs",
        src: include_str!("textarea.rs"),
    },
    FlattenInvariant {
        file: "timeline.rs",
        src: include_str!("timeline.rs"),
    },
    FlattenInvariant {
        file: "title.rs",
        src: include_str!("title.rs"),
    },
    FlattenInvariant {
        file: "title_bar.rs",
        src: include_str!("title_bar.rs"),
    },
    FlattenInvariant {
        file: "toggle.rs",
        src: include_str!("toggle.rs"),
    },
    FlattenInvariant {
        file: "tooltip.rs",
        src: include_str!("tooltip.rs"),
    },
    FlattenInvariant {
        file: "transition.rs",
        src: include_str!("transition.rs"),
    },
    FlattenInvariant {
        file: "tree.rs",
        src: include_str!("tree.rs"),
    },
    FlattenInvariant {
        file: "tree_state.rs",
        src: include_str!("tree_state.rs"),
    },
    FlattenInvariant {
        file: "utils.rs",
        src: include_str!("utils.rs"),
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
    let table_src = include_str!("table.rs");
    assert!(
        !table_src.contains("cell = cell.child(div().child(\"\"));"),
        "table.rs should not use empty div placeholder for missing cells",
    );

    let layout_src = include_str!("layout.rs");
    assert!(
        !layout_src.contains("current_row.push(div().w_full().h_full().into_any_element());"),
        "layout.rs should not push synthetic placeholder cells",
    );
}

#[test]
fn popup_trigger_defaults_do_not_wrap_text_in_extra_div() {
    let menu_src = include_str!("menu.rs");
    assert!(
        !menu_src.contains(".unwrap_or_else(|| div().child(\"Menu\").into_any_element())"),
        "menu.rs should not use wrapped div fallback for trigger text",
    );

    let popover_src = include_str!("popover.rs");
    assert!(
        !popover_src.contains(".unwrap_or_else(|| div().child(\"Open\").into_any_element())"),
        "popover.rs should not use wrapped div fallback for trigger text",
    );

    let tooltip_src = include_str!("tooltip.rs");
    assert!(
        !tooltip_src.contains(".unwrap_or_else(|| div().child(\"target\").into_any_element())"),
        "tooltip.rs should not use wrapped div fallback for trigger text",
    );

    let hovercard_src = include_str!("hovercard.rs");
    assert!(
        !hovercard_src.contains(".unwrap_or_else(|| div().child(\"target\").into_any_element())"),
        "hovercard.rs should not use wrapped div fallback for trigger text",
    );
}
