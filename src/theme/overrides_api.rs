use super::*;
macro_rules! impl_option_overrides_methods {
    ($type:ty { $($field:ident : $value:ty),* $(,)? }) => {
        impl $type {
            $(
                pub fn $field(mut self, value: impl Into<$value>) -> Self {
                    self.$field = Some(value.into());
                    self
                }
            )*
        }
    };
}

macro_rules! impl_nested_overrides_methods {
    ($type:ty { $($field:ident : $value:ty),* $(,)? }) => {
        impl $type {
            $(
                pub fn $field(mut self, configure: impl FnOnce($value) -> $value) -> Self {
                    self.$field = configure(self.$field);
                    self
                }
            )*
        }
    };
}

impl_option_overrides_methods!(SemanticOverrides {
    text_primary: Hsla,
    text_secondary: Hsla,
    text_muted: Hsla,
    bg_canvas: Hsla,
    bg_surface: Hsla,
    bg_soft: Hsla,
    border_subtle: Hsla,
    border_strong: Hsla,
    focus_ring: Hsla,
    status_info: Hsla,
    status_success: Hsla,
    status_warning: Hsla,
    status_error: Hsla,
    overlay_mask: Hsla,
});

impl_option_overrides_methods!(RadiiOverrides {
    default: Pixels,
    xs: Pixels,
    sm: Pixels,
    md: Pixels,
    lg: Pixels,
    xl: Pixels,
    pill: Pixels,
});

impl_option_overrides_methods!(ButtonOverrides {
    filled_bg: Hsla,
    filled_fg: Hsla,
    light_bg: Hsla,
    light_fg: Hsla,
    subtle_bg: Hsla,
    subtle_fg: Hsla,
    outline_border: Hsla,
    outline_fg: Hsla,
    ghost_fg: Hsla,
    disabled_bg: Hsla,
    disabled_fg: Hsla,
    sizes: ButtonSizeScale,
});

impl_option_overrides_methods!(InputOverrides {
    bg: Hsla,
    fg: Hsla,
    caret: Hsla,
    selection_bg: Hsla,
    placeholder: Hsla,
    border: Hsla,
    border_focus: Hsla,
    border_error: Hsla,
    label: Hsla,
    label_size: Pixels,
    label_weight: FontWeight,
    description: Hsla,
    description_size: Pixels,
    error: Hsla,
    error_size: Pixels,
    label_block_gap: Pixels,
    label_row_gap: Pixels,
    slot_fg: Hsla,
    slot_gap: Pixels,
    slot_min_width: Pixels,
    layout_gap_vertical: Pixels,
    layout_gap_horizontal: Pixels,
    horizontal_label_width: Pixels,
    pin_cells_gap: Pixels,
    pin_error_gap: Pixels,
    sizes: FieldSizeScale,
});

impl_option_overrides_methods!(RadioOverrides {
    control_bg: Hsla,
    border: Hsla,
    border_hover: Hsla,
    border_focus: Hsla,
    border_checked: Hsla,
    indicator: Hsla,
    label: Hsla,
    description: Hsla,
    label_description_gap: Pixels,
    group_gap_horizontal: Pixels,
    group_gap_vertical: Pixels,
    sizes: ChoiceControlSizeScale,
});

impl_option_overrides_methods!(CheckboxOverrides {
    control_bg: Hsla,
    control_bg_checked: Hsla,
    border: Hsla,
    border_hover: Hsla,
    border_focus: Hsla,
    border_checked: Hsla,
    indicator: Hsla,
    label: Hsla,
    description: Hsla,
    label_description_gap: Pixels,
    group_gap_horizontal: Pixels,
    group_gap_vertical: Pixels,
    sizes: ChoiceControlSizeScale,
});

impl_option_overrides_methods!(SwitchOverrides {
    track_off_bg: Hsla,
    track_on_bg: Hsla,
    track_hover_border: Hsla,
    track_focus_border: Hsla,
    thumb_bg: Hsla,
    label: Hsla,
    description: Hsla,
    label_description_gap: Pixels,
    sizes: SwitchSizeScale,
});

impl_option_overrides_methods!(ChipOverrides {
    unchecked_bg: Hsla,
    unchecked_fg: Hsla,
    unchecked_border: Hsla,
    filled_bg: Hsla,
    filled_fg: Hsla,
    light_bg: Hsla,
    light_fg: Hsla,
    subtle_bg: Hsla,
    subtle_fg: Hsla,
    outline_border: Hsla,
    outline_fg: Hsla,
    ghost_fg: Hsla,
    default_bg: Hsla,
    default_fg: Hsla,
    default_border: Hsla,
    border_hover: Hsla,
    border_focus: Hsla,
    content_gap: Pixels,
    indicator_size: Pixels,
    group_gap_horizontal: Pixels,
    group_gap_vertical: Pixels,
    sizes: ButtonSizeScale,
});

impl_option_overrides_methods!(BadgeOverrides {
    filled_bg: Hsla,
    filled_fg: Hsla,
    light_bg: Hsla,
    light_fg: Hsla,
    subtle_bg: Hsla,
    subtle_fg: Hsla,
    outline_border: Hsla,
    outline_fg: Hsla,
    default_bg: Hsla,
    default_fg: Hsla,
    default_border: Hsla,
    sizes: BadgeSizeScale,
});

impl_option_overrides_methods!(AccordionOverrides {
    item_bg: Hsla,
    item_border: Hsla,
    label: Hsla,
    description: Hsla,
    content: Hsla,
    chevron: Hsla,
    stack_gap: Pixels,
    header_gap: Pixels,
    label_stack_gap: Pixels,
    panel_gap: Pixels,
    sizes: AccordionSizeScale,
});

impl_option_overrides_methods!(MenuOverrides {
    dropdown_bg: Hsla,
    dropdown_border: Hsla,
    item_fg: Hsla,
    item_hover_bg: Hsla,
    item_disabled_fg: Hsla,
    icon: Hsla,
    item_gap: Pixels,
    item_padding_x: Pixels,
    item_padding_y: Pixels,
    item_size: Pixels,
    item_icon_size: Pixels,
    item_radius: Pixels,
    dropdown_padding: Pixels,
    dropdown_gap: Pixels,
    dropdown_radius: Pixels,
    dropdown_width_fallback: Pixels,
    dropdown_min_width: Pixels,
});

impl_option_overrides_methods!(ProgressOverrides {
    track_bg: Hsla,
    fill_bg: Hsla,
    label: Hsla,
    default_width: Pixels,
    min_width: Pixels,
    root_gap: Pixels,
    sizes: ProgressSizeScale,
});

impl_option_overrides_methods!(SliderOverrides {
    track_bg: Hsla,
    fill_bg: Hsla,
    thumb_bg: Hsla,
    thumb_border: Hsla,
    label: Hsla,
    value: Hsla,
    label_size: Pixels,
    value_size: Pixels,
    header_gap_vertical: Pixels,
    header_gap_horizontal: Pixels,
    default_width: Pixels,
    min_width: Pixels,
    sizes: SliderSizeScale,
});

impl_option_overrides_methods!(OverlayOverrides { bg: Hsla });

impl_option_overrides_methods!(LoaderOverrides {
    color: Hsla,
    label: Hsla,
    sizes: LoaderSizeScale,
});

impl_option_overrides_methods!(LoadingOverlayOverrides {
    bg: Hsla,
    loader_color: Hsla,
    label: Hsla,
    content_gap: Pixels,
    label_size: Pixels,
});

impl_option_overrides_methods!(PopoverOverrides {
    bg: Hsla,
    border: Hsla,
    title: Hsla,
    body: Hsla,
    padding: Pixels,
    gap: Pixels,
    radius: Pixels,
});

impl_option_overrides_methods!(TooltipOverrides {
    bg: Hsla,
    fg: Hsla,
    border: Hsla,
    text_size: Pixels,
    padding_x: Pixels,
    padding_y: Pixels,
    radius: Pixels,
    max_width: Pixels,
});

impl_option_overrides_methods!(HoverCardOverrides {
    bg: Hsla,
    border: Hsla,
    title: Hsla,
    body: Hsla,
    title_size: Pixels,
    title_weight: FontWeight,
    body_size: Pixels,
    min_width: Pixels,
    max_width: Pixels,
    padding: Pixels,
    gap: Pixels,
    radius: Pixels,
});

impl_option_overrides_methods!(SelectOverrides {
    bg: Hsla,
    fg: Hsla,
    placeholder: Hsla,
    border: Hsla,
    border_focus: Hsla,
    border_error: Hsla,
    dropdown_bg: Hsla,
    dropdown_border: Hsla,
    option_fg: Hsla,
    option_hover_bg: Hsla,
    option_selected_bg: Hsla,
    tag_bg: Hsla,
    tag_fg: Hsla,
    tag_border: Hsla,
    icon: Hsla,
    label: Hsla,
    label_size: Pixels,
    label_weight: FontWeight,
    description: Hsla,
    description_size: Pixels,
    error: Hsla,
    error_size: Pixels,
    label_block_gap: Pixels,
    label_row_gap: Pixels,
    slot_gap: Pixels,
    slot_min_width: Pixels,
    layout_gap_vertical: Pixels,
    layout_gap_horizontal: Pixels,
    horizontal_label_width: Pixels,
    icon_size: Pixels,
    option_size: Pixels,
    option_padding_x: Pixels,
    option_padding_y: Pixels,
    option_content_gap: Pixels,
    option_check_size: Pixels,
    dropdown_padding: Pixels,
    dropdown_gap: Pixels,
    dropdown_max_height: Pixels,
    dropdown_width_fallback: Pixels,
    dropdown_open_preferred_height: Pixels,
    tag_size: Pixels,
    tag_padding_x: Pixels,
    tag_padding_y: Pixels,
    tag_gap: Pixels,
    tag_max_width: Pixels,
    dropdown_anchor_offset: Pixels,
    sizes: FieldSizeScale,
});

impl_option_overrides_methods!(ModalOverrides {
    panel_bg: Hsla,
    panel_border: Hsla,
    overlay_bg: Hsla,
    title: Hsla,
    body: Hsla,
    title_size: Pixels,
    title_weight: FontWeight,
    body_size: Pixels,
    kind_icon_size: Pixels,
    kind_icon_gap: Pixels,
    panel_radius: Pixels,
    panel_padding: Pixels,
    header_margin_bottom: Pixels,
    body_margin_bottom: Pixels,
    actions_margin_top: Pixels,
    actions_gap: Pixels,
    close_size: Pixels,
    close_icon_size: Pixels,
    default_width: Pixels,
    min_width: Pixels,
});

impl_option_overrides_methods!(ToastOverrides {
    info_bg: Hsla,
    info_fg: Hsla,
    success_bg: Hsla,
    success_fg: Hsla,
    warning_bg: Hsla,
    warning_fg: Hsla,
    error_bg: Hsla,
    error_fg: Hsla,
    card_width: Pixels,
    card_padding: Pixels,
    row_gap: Pixels,
    content_gap: Pixels,
    icon_box_size: Pixels,
    icon_size: Pixels,
    close_button_size: Pixels,
    close_icon_size: Pixels,
    title_size: Pixels,
    body_size: Pixels,
    stack_gap: Pixels,
    edge_offset: Pixels,
    top_offset_extra: Pixels,
});

impl_option_overrides_methods!(DividerOverrides {
    line: Hsla,
    line_width: Pixels,
    label: Hsla,
    label_size: Pixels,
    label_gap: Pixels,
    edge_span: Pixels,
});

impl_option_overrides_methods!(ScrollAreaOverrides {
    bg: Hsla,
    border: Hsla,
    padding: InsetSizeScale,
});

impl_option_overrides_methods!(DrawerOverrides {
    panel_bg: Hsla,
    panel_border: Hsla,
    overlay_bg: Hsla,
    title: Hsla,
    body: Hsla,
    title_size: Pixels,
    title_weight: FontWeight,
    body_size: Pixels,
    panel_padding: Pixels,
    panel_radius: Pixels,
    header_margin_bottom: Pixels,
    close_size: Pixels,
    close_icon_size: Pixels,
});

impl_option_overrides_methods!(AppShellOverrides {
    bg: Hsla,
    title_bar_bg: Hsla,
    sidebar_bg: Hsla,
    sidebar_overlay_bg: Hsla,
    content_bg: Hsla,
    bottom_panel_bg: Hsla,
    inspector_bg: Hsla,
    inspector_overlay_bg: Hsla,
    region_border: Hsla,
    title_bar_height: Pixels,
    sidebar_width: Pixels,
    sidebar_min_width: Pixels,
    inspector_width: Pixels,
    inspector_min_width: Pixels,
    bottom_panel_height: Pixels,
    bottom_panel_min_height: Pixels,
});

impl_option_overrides_methods!(TitleBarOverrides {
    bg: Hsla,
    border: Hsla,
    fg: Hsla,
    controls_bg: Hsla,
    height: Pixels,
    title_size: Pixels,
    title_weight: FontWeight,
    windows_button_width: Pixels,
    windows_icon_size: Pixels,
    linux_button_width: Pixels,
    linux_button_height: Pixels,
    linux_buttons_gap: Pixels,
    macos_controls_reserve: Pixels,
    title_padding_right: Pixels,
    title_max_width: Pixels,
    title_min_width: Pixels,
    platform_padding_left: Pixels,
    platform_padding_right: Pixels,
    controls_slot_gap: Pixels,
    control_button_radius: Pixels,
});

impl_option_overrides_methods!(SidebarOverrides {
    bg: Hsla,
    border: Hsla,
    header_fg: Hsla,
    content_fg: Hsla,
    footer_fg: Hsla,
    inline_radius: Pixels,
    overlay_radius: Pixels,
    min_width: Pixels,
    section_padding: Pixels,
    footer_size: Pixels,
    scroll_padding: Size,
});

impl_option_overrides_methods!(MarkdownOverrides {
    paragraph: Hsla,
    paragraph_muted: Hsla,
    heading: Hsla,
    heading2_border: Hsla,
    quote_bg: Hsla,
    quote_border: Hsla,
    quote_fg: Hsla,
    code_bg: Hsla,
    code_border: Hsla,
    code_fg: Hsla,
    code_lang_fg: Hsla,
    link: Hsla,
    link_hover: Hsla,
    strong: Hsla,
    em: Hsla,
    del: Hsla,
    inline_code_bg: Hsla,
    inline_code_border: Hsla,
    inline_code_fg: Hsla,
    kbd_bg: Hsla,
    kbd_border: Hsla,
    kbd_fg: Hsla,
    mark_bg: Hsla,
    mark_fg: Hsla,
    list_marker: Hsla,
    rule: Hsla,
    table_border: Hsla,
    table_header_bg: Hsla,
    table_header_fg: Hsla,
    table_row_alt_bg: Hsla,
    table_cell_fg: Hsla,
    task_border: Hsla,
    task_bg: Hsla,
    task_checked_bg: Hsla,
    task_checked_fg: Hsla,
    details_bg: Hsla,
    details_border: Hsla,
    details_summary_fg: Hsla,
    details_body_fg: Hsla,
    image_border: Hsla,
    image_bg: Hsla,
    image_caption_fg: Hsla,
    gap_regular: Pixels,
    gap_compact: Pixels,
    paragraph_size: Pixels,
    paragraph_line_height: Pixels,
    quote_size: Pixels,
    quote_line_height: Pixels,
    code_size: Pixels,
    code_line_height: Pixels,
    code_lang_size: Pixels,
    list_size: Pixels,
    list_line_height: Pixels,
    table_size: Pixels,
    image_caption_size: Pixels,
    quote_padding_x: Pixels,
    quote_padding_y: Pixels,
    quote_radius: Pixels,
    quote_gap: Pixels,
    code_padding: Pixels,
    code_radius: Pixels,
    code_gap: Pixels,
    inline_code_radius: Pixels,
    kbd_radius: Pixels,
    list_gap: Pixels,
    list_item_gap: Pixels,
    list_indent: Pixels,
    table_radius: Pixels,
    table_cell_padding_x: Pixels,
    table_cell_padding_y: Pixels,
    details_radius: Pixels,
    details_padding_x: Pixels,
    details_padding_y: Pixels,
    image_radius: Pixels,
    image_padding: Pixels,
    image_gap: Pixels,
    heading2_padding_top: Pixels,
});

impl_option_overrides_methods!(TextOverrides {
    fg: Hsla,
    secondary: Hsla,
    muted: Hsla,
    accent: Hsla,
    success: Hsla,
    warning: Hsla,
    error: Hsla,
    sizes: TextSizeScale,
});

impl_option_overrides_methods!(TitleOverrides {
    fg: Hsla,
    subtitle: Hsla,
    gap: Pixels,
    subtitle_size: Pixels,
    subtitle_line_height: Pixels,
    subtitle_weight: FontWeight,
});

impl_option_overrides_methods!(TitleLevelOverrides {
    font_size: Pixels,
    line_height: Pixels,
    weight: FontWeight,
});

impl_option_overrides_methods!(PaperOverrides {
    bg: Hsla,
    border: Hsla,
    padding: InsetSizeScale,
});

impl_option_overrides_methods!(ActionIconOverrides {
    filled_bg: Hsla,
    filled_fg: Hsla,
    light_bg: Hsla,
    light_fg: Hsla,
    subtle_bg: Hsla,
    subtle_fg: Hsla,
    outline_border: Hsla,
    outline_fg: Hsla,
    ghost_fg: Hsla,
    default_bg: Hsla,
    default_fg: Hsla,
    default_border: Hsla,
    disabled_bg: Hsla,
    disabled_fg: Hsla,
    disabled_border: Hsla,
    sizes: ActionIconSizeScale,
});

impl_option_overrides_methods!(SegmentedControlOverrides {
    bg: Hsla,
    border: Hsla,
    item_fg: Hsla,
    item_active_bg: Hsla,
    item_active_fg: Hsla,
    item_hover_bg: Hsla,
    item_disabled_fg: Hsla,
    track_padding: Pixels,
    item_gap: Pixels,
    sizes: SegmentedControlSizeScale,
});

impl_option_overrides_methods!(TextareaOverrides {
    bg: Hsla,
    fg: Hsla,
    caret: Hsla,
    selection_bg: Hsla,
    placeholder: Hsla,
    border: Hsla,
    border_focus: Hsla,
    border_error: Hsla,
    label: Hsla,
    label_size: Pixels,
    label_weight: FontWeight,
    description: Hsla,
    description_size: Pixels,
    error: Hsla,
    error_size: Pixels,
    label_block_gap: Pixels,
    label_row_gap: Pixels,
    layout_gap_vertical: Pixels,
    layout_gap_horizontal: Pixels,
    horizontal_label_width: Pixels,
    content_width_fallback: Pixels,
    sizes: FieldSizeScale,
});

impl_option_overrides_methods!(NumberInputOverrides {
    bg: Hsla,
    fg: Hsla,
    placeholder: Hsla,
    border: Hsla,
    border_focus: Hsla,
    border_error: Hsla,
    controls_bg: Hsla,
    controls_fg: Hsla,
    controls_border: Hsla,
    label: Hsla,
    label_size: Pixels,
    label_weight: FontWeight,
    description: Hsla,
    description_size: Pixels,
    error: Hsla,
    error_size: Pixels,
    controls_width: Pixels,
    controls_height: Pixels,
    controls_icon_size: Pixels,
    controls_gap: Pixels,
    sizes: FieldSizeScale,
});

impl_option_overrides_methods!(RangeSliderOverrides {
    track_bg: Hsla,
    range_bg: Hsla,
    thumb_bg: Hsla,
    thumb_border: Hsla,
    label: Hsla,
    value: Hsla,
    label_size: Pixels,
    value_size: Pixels,
    header_gap_vertical: Pixels,
    header_gap_horizontal: Pixels,
    default_width: Pixels,
    min_width: Pixels,
    sizes: SliderSizeScale,
});

impl_option_overrides_methods!(RatingOverrides {
    active: Hsla,
    inactive: Hsla,
    sizes: RatingSizeScale,
});

impl_option_overrides_methods!(TabsOverrides {
    list_bg: Hsla,
    list_border: Hsla,
    tab_fg: Hsla,
    tab_active_bg: Hsla,
    tab_active_fg: Hsla,
    tab_hover_bg: Hsla,
    tab_disabled_fg: Hsla,
    panel_bg: Hsla,
    panel_border: Hsla,
    panel_fg: Hsla,
    root_gap: Pixels,
    list_gap: Pixels,
    list_padding: Pixels,
    panel_padding: Pixels,
    sizes: TabsSizeScale,
});

impl_option_overrides_methods!(PaginationOverrides {
    item_bg: Hsla,
    item_border: Hsla,
    item_fg: Hsla,
    item_active_bg: Hsla,
    item_active_fg: Hsla,
    item_hover_bg: Hsla,
    item_disabled_fg: Hsla,
    dots_fg: Hsla,
    root_gap: Pixels,
    sizes: PaginationSizeScale,
});

impl_option_overrides_methods!(BreadcrumbsOverrides {
    item_fg: Hsla,
    item_current_fg: Hsla,
    separator: Hsla,
    item_hover_bg: Hsla,
    root_gap: Pixels,
    sizes: BreadcrumbsSizeScale,
});

impl_option_overrides_methods!(TableOverrides {
    header_bg: Hsla,
    header_fg: Hsla,
    row_bg: Hsla,
    row_alt_bg: Hsla,
    row_hover_bg: Hsla,
    row_border: Hsla,
    cell_fg: Hsla,
    caption: Hsla,
    caption_size: Pixels,
    row_gap: Pixels,
    pagination_summary_size: Pixels,
    page_chip_size: Pixels,
    page_chip_padding_x: Pixels,
    page_chip_padding_y: Pixels,
    page_chip_radius: Pixels,
    page_chip_gap: Pixels,
    pagination_items_gap: Pixels,
    pagination_padding_x: Pixels,
    pagination_padding_y: Pixels,
    pagination_gap: Pixels,
    virtualization_padding: Pixels,
    min_viewport_height: Pixels,
    sizes: TableSizeScale,
});

impl_option_overrides_methods!(StepperOverrides {
    step_bg: Hsla,
    step_border: Hsla,
    step_fg: Hsla,
    step_active_bg: Hsla,
    step_active_border: Hsla,
    step_active_fg: Hsla,
    step_completed_bg: Hsla,
    step_completed_border: Hsla,
    step_completed_fg: Hsla,
    connector: Hsla,
    label: Hsla,
    description: Hsla,
    panel_bg: Hsla,
    panel_border: Hsla,
    panel_fg: Hsla,
    root_gap: Pixels,
    steps_gap_vertical: Pixels,
    text_gap: Pixels,
    panel_margin_top: Pixels,
    sizes: StepperSizeScale,
});

impl_option_overrides_methods!(TimelineOverrides {
    bullet_bg: Hsla,
    bullet_border: Hsla,
    bullet_fg: Hsla,
    bullet_active_bg: Hsla,
    bullet_active_border: Hsla,
    bullet_active_fg: Hsla,
    line: Hsla,
    line_active: Hsla,
    title: Hsla,
    title_active: Hsla,
    body: Hsla,
    card_bg: Hsla,
    card_border: Hsla,
    root_gap: Pixels,
    row_gap: Pixels,
    content_gap: Pixels,
    card_margin_top: Pixels,
    row_padding_y: Pixels,
    line_min_height: Pixels,
    line_extra_height: Pixels,
    sizes: TimelineSizeScale,
});

impl_option_overrides_methods!(TreeOverrides {
    row_fg: Hsla,
    row_selected_fg: Hsla,
    row_selected_bg: Hsla,
    row_hover_bg: Hsla,
    row_disabled_fg: Hsla,
    line: Hsla,
    root_gap: Pixels,
    children_gap: Pixels,
    sizes: TreeSizeScale,
});

impl_option_overrides_methods!(LayoutOverrides {
    gap: GapSizeScale,
    space: GapSizeScale,
    popup_snap_margin: Pixels,
});

impl_nested_overrides_methods!(ComponentOverrides {
    button: ButtonOverrides,
    input: InputOverrides,
    radio: RadioOverrides,
    checkbox: CheckboxOverrides,
    switch: SwitchOverrides,
    chip: ChipOverrides,
    badge: BadgeOverrides,
    accordion: AccordionOverrides,
    menu: MenuOverrides,
    progress: ProgressOverrides,
    slider: SliderOverrides,
    overlay: OverlayOverrides,
    loader: LoaderOverrides,
    loading_overlay: LoadingOverlayOverrides,
    popover: PopoverOverrides,
    tooltip: TooltipOverrides,
    hover_card: HoverCardOverrides,
    select: SelectOverrides,
    modal: ModalOverrides,
    toast: ToastOverrides,
    divider: DividerOverrides,
    scroll_area: ScrollAreaOverrides,
    drawer: DrawerOverrides,
    app_shell: AppShellOverrides,
    title_bar: TitleBarOverrides,
    sidebar: SidebarOverrides,
    markdown: MarkdownOverrides,
    text: TextOverrides,
    title: TitleOverrides,
    paper: PaperOverrides,
    action_icon: ActionIconOverrides,
    segmented_control: SegmentedControlOverrides,
    textarea: TextareaOverrides,
    number_input: NumberInputOverrides,
    range_slider: RangeSliderOverrides,
    rating: RatingOverrides,
    tabs: TabsOverrides,
    pagination: PaginationOverrides,
    breadcrumbs: BreadcrumbsOverrides,
    table: TableOverrides,
    stepper: StepperOverrides,
    timeline: TimelineOverrides,
    tree: TreeOverrides,
    layout: LayoutOverrides,
});

impl ThemeOverrides {
    pub fn primary_color(mut self, value: impl Into<PaletteKey>) -> Self {
        self.primary_color = Some(value.into());
        self
    }

    pub fn primary_shade_light(mut self, value: impl Into<u8>) -> Self {
        self.primary_shade_light = Some(value.into());
        self
    }

    pub fn primary_shade_dark(mut self, value: impl Into<u8>) -> Self {
        self.primary_shade_dark = Some(value.into());
        self
    }

    pub fn color_scheme(mut self, value: impl Into<ColorScheme>) -> Self {
        self.color_scheme = Some(value.into());
        self
    }

    pub fn primary_shade(mut self, shade: u8) -> Self {
        let clamped = shade.min(9);
        self.primary_shade_light = Some(clamped);
        self.primary_shade_dark = Some(clamped);
        self
    }

    pub fn palette_override(mut self, key: PaletteKey, scale: ColorScale) -> Self {
        self.palette_overrides.insert(key, scale);
        self
    }

    pub fn radii(mut self, configure: impl FnOnce(RadiiOverrides) -> RadiiOverrides) -> Self {
        self.radii = configure(self.radii);
        self
    }

    pub fn semantic(
        mut self,
        configure: impl FnOnce(SemanticOverrides) -> SemanticOverrides,
    ) -> Self {
        self.semantic = configure(self.semantic);
        self
    }

    pub fn components(
        mut self,
        configure: impl FnOnce(ComponentOverrides) -> ComponentOverrides,
    ) -> Self {
        self.components = configure(self.components);
        self
    }
}

macro_rules! impl_theme_component_passthrough_methods {
    ($($field:ident : $value:ty),* $(,)?) => {
        impl ThemeOverrides {
            $(
                pub fn $field(mut self, configure: impl FnOnce($value) -> $value) -> Self {
                    self.components = self.components.$field(configure);
                    self
                }
            )*
        }
    };
}

impl_theme_component_passthrough_methods!(
    button: ButtonOverrides,
    input: InputOverrides,
    radio: RadioOverrides,
    checkbox: CheckboxOverrides,
    switch: SwitchOverrides,
    chip: ChipOverrides,
    badge: BadgeOverrides,
    accordion: AccordionOverrides,
    menu: MenuOverrides,
    progress: ProgressOverrides,
    slider: SliderOverrides,
    overlay: OverlayOverrides,
    loader: LoaderOverrides,
    loading_overlay: LoadingOverlayOverrides,
    popover: PopoverOverrides,
    tooltip: TooltipOverrides,
    hover_card: HoverCardOverrides,
    select: SelectOverrides,
    modal: ModalOverrides,
    toast: ToastOverrides,
    divider: DividerOverrides,
    scroll_area: ScrollAreaOverrides,
    drawer: DrawerOverrides,
    app_shell: AppShellOverrides,
    title_bar: TitleBarOverrides,
    sidebar: SidebarOverrides,
    markdown: MarkdownOverrides,
    text: TextOverrides,
    title: TitleOverrides,
    paper: PaperOverrides,
    action_icon: ActionIconOverrides,
    segmented_control: SegmentedControlOverrides,
    textarea: TextareaOverrides,
    number_input: NumberInputOverrides,
    range_slider: RangeSliderOverrides,
    rating: RatingOverrides,
    tabs: TabsOverrides,
    pagination: PaginationOverrides,
    breadcrumbs: BreadcrumbsOverrides,
    table: TableOverrides,
    stepper: StepperOverrides,
    timeline: TimelineOverrides,
    tree: TreeOverrides,
    layout: LayoutOverrides,
);

impl Theme {
    pub fn with_overrides(self, configure: impl FnOnce(ThemeOverrides) -> ThemeOverrides) -> Self {
        let overrides = configure(ThemeOverrides::default());
        self.merged(&overrides)
    }
}
