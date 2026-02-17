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
});

impl_option_overrides_methods!(InputOverrides {
    bg: Hsla,
    fg: Hsla,
    placeholder: Hsla,
    border: Hsla,
    border_focus: Hsla,
    border_error: Hsla,
    label: Hsla,
    description: Hsla,
    error: Hsla,
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
});

impl_option_overrides_methods!(SwitchOverrides {
    track_off_bg: Hsla,
    track_on_bg: Hsla,
    track_hover_border: Hsla,
    track_focus_border: Hsla,
    thumb_bg: Hsla,
    label: Hsla,
    description: Hsla,
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
});

impl_option_overrides_methods!(AccordionOverrides {
    item_bg: Hsla,
    item_border: Hsla,
    label: Hsla,
    description: Hsla,
    content: Hsla,
    chevron: Hsla,
});

impl_option_overrides_methods!(MenuOverrides {
    dropdown_bg: Hsla,
    dropdown_border: Hsla,
    item_fg: Hsla,
    item_hover_bg: Hsla,
    item_disabled_fg: Hsla,
    icon: Hsla,
});

impl_option_overrides_methods!(ProgressOverrides {
    track_bg: Hsla,
    fill_bg: Hsla,
    label: Hsla,
});

impl_option_overrides_methods!(SliderOverrides {
    track_bg: Hsla,
    fill_bg: Hsla,
    thumb_bg: Hsla,
    thumb_border: Hsla,
    label: Hsla,
    value: Hsla,
});

impl_option_overrides_methods!(OverlayOverrides { bg: Hsla });

impl_option_overrides_methods!(LoadingOverlayOverrides {
    bg: Hsla,
    loader_color: Hsla,
    label: Hsla,
});

impl_option_overrides_methods!(PopoverOverrides {
    bg: Hsla,
    border: Hsla,
    title: Hsla,
    body: Hsla,
});

impl_option_overrides_methods!(TooltipOverrides {
    bg: Hsla,
    fg: Hsla,
    border: Hsla,
});

impl_option_overrides_methods!(HoverCardOverrides {
    bg: Hsla,
    border: Hsla,
    title: Hsla,
    body: Hsla,
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
    description: Hsla,
    error: Hsla,
});

impl_option_overrides_methods!(ModalOverrides {
    panel_bg: Hsla,
    panel_border: Hsla,
    overlay_bg: Hsla,
    title: Hsla,
    body: Hsla,
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
});

impl_option_overrides_methods!(DividerOverrides {
    line: Hsla,
    label: Hsla,
});

impl_option_overrides_methods!(ScrollAreaOverrides {
    bg: Hsla,
    border: Hsla,
});

impl_option_overrides_methods!(DrawerOverrides {
    panel_bg: Hsla,
    panel_border: Hsla,
    overlay_bg: Hsla,
    title: Hsla,
    body: Hsla,
});

impl_option_overrides_methods!(AppShellOverrides { bg: Hsla });

impl_option_overrides_methods!(TitleBarOverrides {
    bg: Hsla,
    border: Hsla,
    fg: Hsla,
    controls_bg: Hsla,
});

impl_option_overrides_methods!(SidebarOverrides {
    bg: Hsla,
    border: Hsla,
    header_fg: Hsla,
    content_fg: Hsla,
    footer_fg: Hsla,
});

impl_option_overrides_methods!(TextOverrides {
    fg: Hsla,
    secondary: Hsla,
    muted: Hsla,
    accent: Hsla,
    success: Hsla,
    warning: Hsla,
    error: Hsla,
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
});

impl_option_overrides_methods!(SegmentedControlOverrides {
    bg: Hsla,
    border: Hsla,
    item_fg: Hsla,
    item_active_bg: Hsla,
    item_active_fg: Hsla,
    item_hover_bg: Hsla,
    item_disabled_fg: Hsla,
});

impl_option_overrides_methods!(TextareaOverrides {
    bg: Hsla,
    fg: Hsla,
    placeholder: Hsla,
    border: Hsla,
    border_focus: Hsla,
    border_error: Hsla,
    label: Hsla,
    description: Hsla,
    error: Hsla,
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
    description: Hsla,
    error: Hsla,
});

impl_option_overrides_methods!(RangeSliderOverrides {
    track_bg: Hsla,
    range_bg: Hsla,
    thumb_bg: Hsla,
    thumb_border: Hsla,
    label: Hsla,
    value: Hsla,
});

impl_option_overrides_methods!(RatingOverrides {
    active: Hsla,
    inactive: Hsla,
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
});

impl_option_overrides_methods!(BreadcrumbsOverrides {
    item_fg: Hsla,
    item_current_fg: Hsla,
    separator: Hsla,
    item_hover_bg: Hsla,
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
});

impl_option_overrides_methods!(TreeOverrides {
    row_fg: Hsla,
    row_selected_fg: Hsla,
    row_selected_bg: Hsla,
    row_hover_bg: Hsla,
    row_disabled_fg: Hsla,
    line: Hsla,
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
);

impl Theme {
    pub fn with_overrides(self, configure: impl FnOnce(ThemeOverrides) -> ThemeOverrides) -> Self {
        let overrides = configure(ThemeOverrides::default());
        self.merged(&overrides)
    }
}
