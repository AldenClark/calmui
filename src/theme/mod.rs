use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use crate::tokens::{ColorScale, PaletteCatalog, PaletteKey};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorScheme {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimaryShade {
    Uniform(u8),
    Split { light: u8, dark: u8 },
}

impl PrimaryShade {
    pub const fn shade_for(self, scheme: ColorScheme) -> u8 {
        match self {
            Self::Uniform(shade) => shade,
            Self::Split { light, dark } => match scheme {
                ColorScheme::Light => light,
                ColorScheme::Dark => dark,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ColorValue {
    Palette { key: PaletteKey, shade: u8 },
    White,
    Black,
    Custom(String),
}

impl ColorValue {
    pub const fn palette(key: PaletteKey, shade: u8) -> Self {
        Self::Palette { key, shade }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticColors {
    pub text_primary: ColorValue,
    pub text_secondary: ColorValue,
    pub text_muted: ColorValue,
    pub bg_canvas: ColorValue,
    pub bg_surface: ColorValue,
    pub bg_soft: ColorValue,
    pub border_subtle: ColorValue,
    pub border_strong: ColorValue,
    pub focus_ring: ColorValue,
    pub status_info: ColorValue,
    pub status_success: ColorValue,
    pub status_warning: ColorValue,
    pub status_error: ColorValue,
    pub overlay_mask: ColorValue,
}

impl SemanticColors {
    pub fn defaults(primary: PaletteKey) -> Self {
        Self::defaults_for(primary, ColorScheme::Light)
    }

    pub fn defaults_for(primary: PaletteKey, scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Light => Self {
                text_primary: ColorValue::palette(PaletteKey::Dark, 9),
                text_secondary: ColorValue::palette(PaletteKey::Gray, 7),
                text_muted: ColorValue::palette(PaletteKey::Gray, 6),
                bg_canvas: ColorValue::White,
                bg_surface: ColorValue::palette(PaletteKey::Gray, 0),
                bg_soft: ColorValue::palette(PaletteKey::Gray, 1),
                border_subtle: ColorValue::palette(PaletteKey::Gray, 3),
                border_strong: ColorValue::palette(PaletteKey::Gray, 5),
                focus_ring: ColorValue::palette(primary, 6),
                status_info: ColorValue::palette(PaletteKey::Blue, 6),
                status_success: ColorValue::palette(PaletteKey::Green, 6),
                status_warning: ColorValue::palette(PaletteKey::Yellow, 7),
                status_error: ColorValue::palette(PaletteKey::Red, 6),
                overlay_mask: ColorValue::Custom("#00000073".to_string()),
            },
            ColorScheme::Dark => Self {
                text_primary: ColorValue::palette(PaletteKey::Gray, 0),
                text_secondary: ColorValue::palette(PaletteKey::Gray, 3),
                text_muted: ColorValue::palette(PaletteKey::Gray, 5),
                bg_canvas: ColorValue::palette(PaletteKey::Dark, 9),
                bg_surface: ColorValue::palette(PaletteKey::Dark, 8),
                bg_soft: ColorValue::palette(PaletteKey::Dark, 7),
                border_subtle: ColorValue::palette(PaletteKey::Dark, 5),
                border_strong: ColorValue::palette(PaletteKey::Dark, 4),
                focus_ring: ColorValue::palette(primary, 5),
                status_info: ColorValue::palette(PaletteKey::Blue, 4),
                status_success: ColorValue::palette(PaletteKey::Green, 4),
                status_warning: ColorValue::palette(PaletteKey::Yellow, 4),
                status_error: ColorValue::palette(PaletteKey::Red, 4),
                overlay_mask: ColorValue::Custom("#000000CC".to_string()),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ButtonTokens {
    pub filled_bg: ColorValue,
    pub filled_fg: ColorValue,
    pub light_bg: ColorValue,
    pub light_fg: ColorValue,
    pub subtle_bg: ColorValue,
    pub subtle_fg: ColorValue,
    pub outline_border: ColorValue,
    pub outline_fg: ColorValue,
    pub ghost_fg: ColorValue,
    pub disabled_bg: ColorValue,
    pub disabled_fg: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputTokens {
    pub bg: ColorValue,
    pub fg: ColorValue,
    pub placeholder: ColorValue,
    pub border: ColorValue,
    pub border_focus: ColorValue,
    pub border_error: ColorValue,
    pub label: ColorValue,
    pub description: ColorValue,
    pub error: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RadioTokens {
    pub control_bg: ColorValue,
    pub border: ColorValue,
    pub border_checked: ColorValue,
    pub indicator: ColorValue,
    pub label: ColorValue,
    pub description: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckboxTokens {
    pub control_bg: ColorValue,
    pub control_bg_checked: ColorValue,
    pub border: ColorValue,
    pub border_checked: ColorValue,
    pub indicator: ColorValue,
    pub label: ColorValue,
    pub description: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwitchTokens {
    pub track_off_bg: ColorValue,
    pub track_on_bg: ColorValue,
    pub thumb_bg: ColorValue,
    pub label: ColorValue,
    pub description: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChipTokens {
    pub unchecked_bg: ColorValue,
    pub unchecked_fg: ColorValue,
    pub unchecked_border: ColorValue,
    pub filled_bg: ColorValue,
    pub filled_fg: ColorValue,
    pub light_bg: ColorValue,
    pub light_fg: ColorValue,
    pub subtle_bg: ColorValue,
    pub subtle_fg: ColorValue,
    pub outline_border: ColorValue,
    pub outline_fg: ColorValue,
    pub ghost_fg: ColorValue,
    pub default_bg: ColorValue,
    pub default_fg: ColorValue,
    pub default_border: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BadgeTokens {
    pub filled_bg: ColorValue,
    pub filled_fg: ColorValue,
    pub light_bg: ColorValue,
    pub light_fg: ColorValue,
    pub subtle_bg: ColorValue,
    pub subtle_fg: ColorValue,
    pub outline_border: ColorValue,
    pub outline_fg: ColorValue,
    pub default_bg: ColorValue,
    pub default_fg: ColorValue,
    pub default_border: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccordionTokens {
    pub item_bg: ColorValue,
    pub item_border: ColorValue,
    pub label: ColorValue,
    pub description: ColorValue,
    pub content: ColorValue,
    pub chevron: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuTokens {
    pub dropdown_bg: ColorValue,
    pub dropdown_border: ColorValue,
    pub item_fg: ColorValue,
    pub item_hover_bg: ColorValue,
    pub item_disabled_fg: ColorValue,
    pub icon: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgressTokens {
    pub track_bg: ColorValue,
    pub fill_bg: ColorValue,
    pub label: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliderTokens {
    pub track_bg: ColorValue,
    pub fill_bg: ColorValue,
    pub thumb_bg: ColorValue,
    pub thumb_border: ColorValue,
    pub label: ColorValue,
    pub value: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverlayTokens {
    pub bg: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadingOverlayTokens {
    pub bg: ColorValue,
    pub loader_color: ColorValue,
    pub label: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PopoverTokens {
    pub bg: ColorValue,
    pub border: ColorValue,
    pub title: ColorValue,
    pub body: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TooltipTokens {
    pub bg: ColorValue,
    pub fg: ColorValue,
    pub border: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HoverCardTokens {
    pub bg: ColorValue,
    pub border: ColorValue,
    pub title: ColorValue,
    pub body: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectTokens {
    pub bg: ColorValue,
    pub fg: ColorValue,
    pub placeholder: ColorValue,
    pub border: ColorValue,
    pub border_focus: ColorValue,
    pub border_error: ColorValue,
    pub dropdown_bg: ColorValue,
    pub dropdown_border: ColorValue,
    pub option_fg: ColorValue,
    pub option_hover_bg: ColorValue,
    pub option_selected_bg: ColorValue,
    pub tag_bg: ColorValue,
    pub tag_fg: ColorValue,
    pub tag_border: ColorValue,
    pub icon: ColorValue,
    pub label: ColorValue,
    pub description: ColorValue,
    pub error: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModalTokens {
    pub panel_bg: ColorValue,
    pub panel_border: ColorValue,
    pub overlay_bg: ColorValue,
    pub title: ColorValue,
    pub body: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToastTokens {
    pub info_bg: ColorValue,
    pub info_fg: ColorValue,
    pub success_bg: ColorValue,
    pub success_fg: ColorValue,
    pub warning_bg: ColorValue,
    pub warning_fg: ColorValue,
    pub error_bg: ColorValue,
    pub error_fg: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DividerTokens {
    pub line: ColorValue,
    pub label: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScrollAreaTokens {
    pub bg: ColorValue,
    pub border: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrawerTokens {
    pub panel_bg: ColorValue,
    pub panel_border: ColorValue,
    pub overlay_bg: ColorValue,
    pub title: ColorValue,
    pub body: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppShellTokens {
    pub bg: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TitleBarTokens {
    pub bg: ColorValue,
    pub border: ColorValue,
    pub fg: ColorValue,
    pub controls_bg: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarTokens {
    pub bg: ColorValue,
    pub border: ColorValue,
    pub header_fg: ColorValue,
    pub content_fg: ColorValue,
    pub footer_fg: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextTokens {
    pub fg: ColorValue,
    pub secondary: ColorValue,
    pub muted: ColorValue,
    pub accent: ColorValue,
    pub success: ColorValue,
    pub warning: ColorValue,
    pub error: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TitleTokens {
    pub fg: ColorValue,
    pub subtitle: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaperTokens {
    pub bg: ColorValue,
    pub border: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionIconTokens {
    pub filled_bg: ColorValue,
    pub filled_fg: ColorValue,
    pub light_bg: ColorValue,
    pub light_fg: ColorValue,
    pub subtle_bg: ColorValue,
    pub subtle_fg: ColorValue,
    pub outline_border: ColorValue,
    pub outline_fg: ColorValue,
    pub ghost_fg: ColorValue,
    pub default_bg: ColorValue,
    pub default_fg: ColorValue,
    pub default_border: ColorValue,
    pub disabled_bg: ColorValue,
    pub disabled_fg: ColorValue,
    pub disabled_border: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SegmentedControlTokens {
    pub bg: ColorValue,
    pub border: ColorValue,
    pub item_fg: ColorValue,
    pub item_active_bg: ColorValue,
    pub item_active_fg: ColorValue,
    pub item_hover_bg: ColorValue,
    pub item_disabled_fg: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextareaTokens {
    pub bg: ColorValue,
    pub fg: ColorValue,
    pub placeholder: ColorValue,
    pub border: ColorValue,
    pub border_focus: ColorValue,
    pub border_error: ColorValue,
    pub label: ColorValue,
    pub description: ColorValue,
    pub error: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NumberInputTokens {
    pub bg: ColorValue,
    pub fg: ColorValue,
    pub placeholder: ColorValue,
    pub border: ColorValue,
    pub border_focus: ColorValue,
    pub border_error: ColorValue,
    pub controls_bg: ColorValue,
    pub controls_fg: ColorValue,
    pub controls_border: ColorValue,
    pub label: ColorValue,
    pub description: ColorValue,
    pub error: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RangeSliderTokens {
    pub track_bg: ColorValue,
    pub range_bg: ColorValue,
    pub thumb_bg: ColorValue,
    pub thumb_border: ColorValue,
    pub label: ColorValue,
    pub value: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RatingTokens {
    pub active: ColorValue,
    pub inactive: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabsTokens {
    pub list_bg: ColorValue,
    pub list_border: ColorValue,
    pub tab_fg: ColorValue,
    pub tab_active_bg: ColorValue,
    pub tab_active_fg: ColorValue,
    pub tab_hover_bg: ColorValue,
    pub tab_disabled_fg: ColorValue,
    pub panel_bg: ColorValue,
    pub panel_border: ColorValue,
    pub panel_fg: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaginationTokens {
    pub item_bg: ColorValue,
    pub item_border: ColorValue,
    pub item_fg: ColorValue,
    pub item_active_bg: ColorValue,
    pub item_active_fg: ColorValue,
    pub item_hover_bg: ColorValue,
    pub item_disabled_fg: ColorValue,
    pub dots_fg: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreadcrumbsTokens {
    pub item_fg: ColorValue,
    pub item_current_fg: ColorValue,
    pub separator: ColorValue,
    pub item_hover_bg: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableTokens {
    pub header_bg: ColorValue,
    pub header_fg: ColorValue,
    pub row_bg: ColorValue,
    pub row_alt_bg: ColorValue,
    pub row_hover_bg: ColorValue,
    pub row_border: ColorValue,
    pub cell_fg: ColorValue,
    pub caption: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepperTokens {
    pub step_bg: ColorValue,
    pub step_border: ColorValue,
    pub step_fg: ColorValue,
    pub step_active_bg: ColorValue,
    pub step_active_border: ColorValue,
    pub step_active_fg: ColorValue,
    pub step_completed_bg: ColorValue,
    pub step_completed_border: ColorValue,
    pub step_completed_fg: ColorValue,
    pub connector: ColorValue,
    pub label: ColorValue,
    pub description: ColorValue,
    pub panel_bg: ColorValue,
    pub panel_border: ColorValue,
    pub panel_fg: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineTokens {
    pub bullet_bg: ColorValue,
    pub bullet_border: ColorValue,
    pub bullet_fg: ColorValue,
    pub bullet_active_bg: ColorValue,
    pub bullet_active_border: ColorValue,
    pub bullet_active_fg: ColorValue,
    pub line: ColorValue,
    pub line_active: ColorValue,
    pub title: ColorValue,
    pub title_active: ColorValue,
    pub body: ColorValue,
    pub card_bg: ColorValue,
    pub card_border: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeTokens {
    pub row_fg: ColorValue,
    pub row_selected_fg: ColorValue,
    pub row_selected_bg: ColorValue,
    pub row_hover_bg: ColorValue,
    pub row_disabled_fg: ColorValue,
    pub line: ColorValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentTokens {
    pub button: ButtonTokens,
    pub input: InputTokens,
    pub radio: RadioTokens,
    pub checkbox: CheckboxTokens,
    pub switch: SwitchTokens,
    pub chip: ChipTokens,
    pub badge: BadgeTokens,
    pub accordion: AccordionTokens,
    pub menu: MenuTokens,
    pub progress: ProgressTokens,
    pub slider: SliderTokens,
    pub overlay: OverlayTokens,
    pub loading_overlay: LoadingOverlayTokens,
    pub popover: PopoverTokens,
    pub tooltip: TooltipTokens,
    pub hover_card: HoverCardTokens,
    pub select: SelectTokens,
    pub modal: ModalTokens,
    pub toast: ToastTokens,
    pub divider: DividerTokens,
    pub scroll_area: ScrollAreaTokens,
    pub drawer: DrawerTokens,
    pub app_shell: AppShellTokens,
    pub title_bar: TitleBarTokens,
    pub sidebar: SidebarTokens,
    pub text: TextTokens,
    pub title: TitleTokens,
    pub paper: PaperTokens,
    pub action_icon: ActionIconTokens,
    pub segmented_control: SegmentedControlTokens,
    pub textarea: TextareaTokens,
    pub number_input: NumberInputTokens,
    pub range_slider: RangeSliderTokens,
    pub rating: RatingTokens,
    pub tabs: TabsTokens,
    pub pagination: PaginationTokens,
    pub breadcrumbs: BreadcrumbsTokens,
    pub table: TableTokens,
    pub stepper: StepperTokens,
    pub timeline: TimelineTokens,
    pub tree: TreeTokens,
}

impl ComponentTokens {
    pub fn defaults(primary: PaletteKey) -> Self {
        Self::defaults_for(primary, ColorScheme::Light)
    }

    pub fn defaults_for(primary: PaletteKey, scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Light => Self {
                button: ButtonTokens {
                    filled_bg: ColorValue::palette(primary, 6),
                    filled_fg: ColorValue::White,
                    light_bg: ColorValue::palette(primary, 0),
                    light_fg: ColorValue::palette(primary, 6),
                    subtle_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    subtle_fg: ColorValue::palette(PaletteKey::Gray, 8),
                    outline_border: ColorValue::palette(primary, 4),
                    outline_fg: ColorValue::palette(primary, 7),
                    ghost_fg: ColorValue::palette(primary, 6),
                    disabled_bg: ColorValue::palette(PaletteKey::Gray, 2),
                    disabled_fg: ColorValue::palette(PaletteKey::Gray, 5),
                },
                input: InputTokens {
                    bg: ColorValue::White,
                    fg: ColorValue::palette(PaletteKey::Dark, 9),
                    placeholder: ColorValue::palette(PaletteKey::Gray, 5),
                    border: ColorValue::palette(PaletteKey::Gray, 4),
                    border_focus: ColorValue::palette(primary, 6),
                    border_error: ColorValue::palette(PaletteKey::Red, 6),
                    label: ColorValue::palette(PaletteKey::Dark, 8),
                    description: ColorValue::palette(PaletteKey::Gray, 7),
                    error: ColorValue::palette(PaletteKey::Red, 6),
                },
                radio: RadioTokens {
                    control_bg: ColorValue::White,
                    border: ColorValue::palette(PaletteKey::Gray, 4),
                    border_checked: ColorValue::palette(primary, 6),
                    indicator: ColorValue::palette(primary, 6),
                    label: ColorValue::palette(PaletteKey::Dark, 9),
                    description: ColorValue::palette(PaletteKey::Gray, 7),
                },
                checkbox: CheckboxTokens {
                    control_bg: ColorValue::White,
                    control_bg_checked: ColorValue::palette(primary, 6),
                    border: ColorValue::palette(PaletteKey::Gray, 4),
                    border_checked: ColorValue::palette(primary, 6),
                    indicator: ColorValue::White,
                    label: ColorValue::palette(PaletteKey::Dark, 9),
                    description: ColorValue::palette(PaletteKey::Gray, 7),
                },
                switch: SwitchTokens {
                    track_off_bg: ColorValue::palette(PaletteKey::Gray, 4),
                    track_on_bg: ColorValue::palette(primary, 6),
                    thumb_bg: ColorValue::White,
                    label: ColorValue::palette(PaletteKey::Dark, 9),
                    description: ColorValue::palette(PaletteKey::Gray, 7),
                },
                chip: ChipTokens {
                    unchecked_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    unchecked_fg: ColorValue::palette(PaletteKey::Gray, 8),
                    unchecked_border: ColorValue::palette(PaletteKey::Gray, 3),
                    filled_bg: ColorValue::palette(primary, 6),
                    filled_fg: ColorValue::White,
                    light_bg: ColorValue::palette(primary, 0),
                    light_fg: ColorValue::palette(primary, 7),
                    subtle_bg: ColorValue::palette(PaletteKey::Gray, 1),
                    subtle_fg: ColorValue::palette(PaletteKey::Gray, 8),
                    outline_border: ColorValue::palette(primary, 4),
                    outline_fg: ColorValue::palette(primary, 7),
                    ghost_fg: ColorValue::palette(primary, 6),
                    default_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    default_fg: ColorValue::palette(PaletteKey::Dark, 8),
                    default_border: ColorValue::palette(PaletteKey::Gray, 3),
                },
                badge: BadgeTokens {
                    filled_bg: ColorValue::palette(primary, 6),
                    filled_fg: ColorValue::White,
                    light_bg: ColorValue::palette(primary, 0),
                    light_fg: ColorValue::palette(primary, 7),
                    subtle_bg: ColorValue::palette(PaletteKey::Gray, 1),
                    subtle_fg: ColorValue::palette(PaletteKey::Gray, 8),
                    outline_border: ColorValue::palette(primary, 4),
                    outline_fg: ColorValue::palette(primary, 7),
                    default_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    default_fg: ColorValue::palette(PaletteKey::Dark, 8),
                    default_border: ColorValue::palette(PaletteKey::Gray, 3),
                },
                accordion: AccordionTokens {
                    item_bg: ColorValue::White,
                    item_border: ColorValue::palette(PaletteKey::Gray, 3),
                    label: ColorValue::palette(PaletteKey::Dark, 9),
                    description: ColorValue::palette(PaletteKey::Gray, 7),
                    content: ColorValue::palette(PaletteKey::Dark, 8),
                    chevron: ColorValue::palette(PaletteKey::Gray, 7),
                },
                menu: MenuTokens {
                    dropdown_bg: ColorValue::White,
                    dropdown_border: ColorValue::palette(PaletteKey::Gray, 3),
                    item_fg: ColorValue::palette(PaletteKey::Dark, 9),
                    item_hover_bg: ColorValue::palette(PaletteKey::Gray, 1),
                    item_disabled_fg: ColorValue::palette(PaletteKey::Gray, 5),
                    icon: ColorValue::palette(PaletteKey::Gray, 7),
                },
                progress: ProgressTokens {
                    track_bg: ColorValue::palette(PaletteKey::Gray, 2),
                    fill_bg: ColorValue::palette(primary, 6),
                    label: ColorValue::palette(PaletteKey::Dark, 8),
                },
                slider: SliderTokens {
                    track_bg: ColorValue::palette(PaletteKey::Gray, 2),
                    fill_bg: ColorValue::palette(primary, 6),
                    thumb_bg: ColorValue::White,
                    thumb_border: ColorValue::palette(primary, 6),
                    label: ColorValue::palette(PaletteKey::Dark, 8),
                    value: ColorValue::palette(PaletteKey::Gray, 7),
                },
                overlay: OverlayTokens {
                    bg: ColorValue::Custom("#000000E6".to_string()),
                },
                loading_overlay: LoadingOverlayTokens {
                    bg: ColorValue::Custom("#000000E6".to_string()),
                    loader_color: ColorValue::palette(primary, 6),
                    label: ColorValue::White,
                },
                popover: PopoverTokens {
                    bg: ColorValue::White,
                    border: ColorValue::palette(PaletteKey::Gray, 3),
                    title: ColorValue::palette(PaletteKey::Dark, 9),
                    body: ColorValue::palette(PaletteKey::Gray, 8),
                },
                tooltip: TooltipTokens {
                    bg: ColorValue::palette(PaletteKey::Dark, 8),
                    fg: ColorValue::White,
                    border: ColorValue::palette(PaletteKey::Dark, 6),
                },
                hover_card: HoverCardTokens {
                    bg: ColorValue::White,
                    border: ColorValue::palette(PaletteKey::Gray, 3),
                    title: ColorValue::palette(PaletteKey::Dark, 9),
                    body: ColorValue::palette(PaletteKey::Gray, 8),
                },
                select: SelectTokens {
                    bg: ColorValue::White,
                    fg: ColorValue::palette(PaletteKey::Dark, 9),
                    placeholder: ColorValue::palette(PaletteKey::Gray, 5),
                    border: ColorValue::palette(PaletteKey::Gray, 4),
                    border_focus: ColorValue::palette(primary, 6),
                    border_error: ColorValue::palette(PaletteKey::Red, 6),
                    dropdown_bg: ColorValue::White,
                    dropdown_border: ColorValue::palette(PaletteKey::Gray, 3),
                    option_fg: ColorValue::palette(PaletteKey::Dark, 9),
                    option_hover_bg: ColorValue::palette(PaletteKey::Gray, 1),
                    option_selected_bg: ColorValue::palette(primary, 0),
                    tag_bg: ColorValue::palette(primary, 0),
                    tag_fg: ColorValue::palette(primary, 7),
                    tag_border: ColorValue::palette(primary, 3),
                    icon: ColorValue::palette(PaletteKey::Gray, 7),
                    label: ColorValue::palette(PaletteKey::Dark, 8),
                    description: ColorValue::palette(PaletteKey::Gray, 7),
                    error: ColorValue::palette(PaletteKey::Red, 6),
                },
                modal: ModalTokens {
                    panel_bg: ColorValue::White,
                    panel_border: ColorValue::palette(PaletteKey::Gray, 3),
                    overlay_bg: ColorValue::Custom("#000000E6".to_string()),
                    title: ColorValue::palette(PaletteKey::Dark, 9),
                    body: ColorValue::palette(PaletteKey::Gray, 8),
                },
                toast: ToastTokens {
                    info_bg: ColorValue::palette(PaletteKey::Blue, 0),
                    info_fg: ColorValue::palette(PaletteKey::Blue, 8),
                    success_bg: ColorValue::palette(PaletteKey::Green, 0),
                    success_fg: ColorValue::palette(PaletteKey::Green, 8),
                    warning_bg: ColorValue::palette(PaletteKey::Yellow, 0),
                    warning_fg: ColorValue::palette(PaletteKey::Yellow, 9),
                    error_bg: ColorValue::palette(PaletteKey::Red, 0),
                    error_fg: ColorValue::palette(PaletteKey::Red, 8),
                },
                divider: DividerTokens {
                    line: ColorValue::palette(PaletteKey::Gray, 3),
                    label: ColorValue::palette(PaletteKey::Gray, 6),
                },
                scroll_area: ScrollAreaTokens {
                    bg: ColorValue::palette(PaletteKey::Gray, 0),
                    border: ColorValue::palette(PaletteKey::Gray, 3),
                },
                drawer: DrawerTokens {
                    panel_bg: ColorValue::White,
                    panel_border: ColorValue::palette(PaletteKey::Gray, 3),
                    overlay_bg: ColorValue::Custom("#000000E6".to_string()),
                    title: ColorValue::palette(PaletteKey::Dark, 9),
                    body: ColorValue::palette(PaletteKey::Gray, 8),
                },
                app_shell: AppShellTokens {
                    bg: ColorValue::palette(PaletteKey::Gray, 0),
                },
                title_bar: TitleBarTokens {
                    bg: ColorValue::Custom("#00000000".to_string()),
                    border: ColorValue::palette(PaletteKey::Gray, 3),
                    fg: ColorValue::palette(PaletteKey::Dark, 8),
                    controls_bg: ColorValue::palette(PaletteKey::Gray, 1),
                },
                sidebar: SidebarTokens {
                    bg: ColorValue::White,
                    border: ColorValue::palette(PaletteKey::Gray, 3),
                    header_fg: ColorValue::palette(PaletteKey::Dark, 8),
                    content_fg: ColorValue::palette(PaletteKey::Dark, 8),
                    footer_fg: ColorValue::palette(PaletteKey::Gray, 7),
                },
                text: TextTokens {
                    fg: ColorValue::palette(PaletteKey::Dark, 9),
                    secondary: ColorValue::palette(PaletteKey::Gray, 7),
                    muted: ColorValue::palette(PaletteKey::Gray, 6),
                    accent: ColorValue::palette(primary, 6),
                    success: ColorValue::palette(PaletteKey::Green, 7),
                    warning: ColorValue::palette(PaletteKey::Yellow, 8),
                    error: ColorValue::palette(PaletteKey::Red, 6),
                },
                title: TitleTokens {
                    fg: ColorValue::palette(PaletteKey::Dark, 9),
                    subtitle: ColorValue::palette(PaletteKey::Gray, 6),
                },
                paper: PaperTokens {
                    bg: ColorValue::White,
                    border: ColorValue::palette(PaletteKey::Gray, 3),
                },
                action_icon: ActionIconTokens {
                    filled_bg: ColorValue::palette(primary, 6),
                    filled_fg: ColorValue::White,
                    light_bg: ColorValue::palette(primary, 0),
                    light_fg: ColorValue::palette(primary, 6),
                    subtle_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    subtle_fg: ColorValue::palette(PaletteKey::Gray, 8),
                    outline_border: ColorValue::palette(primary, 4),
                    outline_fg: ColorValue::palette(primary, 7),
                    ghost_fg: ColorValue::palette(primary, 6),
                    default_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    default_fg: ColorValue::palette(PaletteKey::Dark, 8),
                    default_border: ColorValue::palette(PaletteKey::Gray, 3),
                    disabled_bg: ColorValue::palette(PaletteKey::Gray, 1),
                    disabled_fg: ColorValue::palette(PaletteKey::Gray, 5),
                    disabled_border: ColorValue::palette(PaletteKey::Gray, 3),
                },
                segmented_control: SegmentedControlTokens {
                    bg: ColorValue::palette(PaletteKey::Gray, 0),
                    border: ColorValue::palette(PaletteKey::Gray, 3),
                    item_fg: ColorValue::palette(PaletteKey::Gray, 8),
                    item_active_bg: ColorValue::White,
                    item_active_fg: ColorValue::palette(PaletteKey::Dark, 9),
                    item_hover_bg: ColorValue::palette(PaletteKey::Gray, 1),
                    item_disabled_fg: ColorValue::palette(PaletteKey::Gray, 5),
                },
                textarea: TextareaTokens {
                    bg: ColorValue::White,
                    fg: ColorValue::palette(PaletteKey::Dark, 9),
                    placeholder: ColorValue::palette(PaletteKey::Gray, 5),
                    border: ColorValue::palette(PaletteKey::Gray, 4),
                    border_focus: ColorValue::palette(primary, 6),
                    border_error: ColorValue::palette(PaletteKey::Red, 6),
                    label: ColorValue::palette(PaletteKey::Dark, 8),
                    description: ColorValue::palette(PaletteKey::Gray, 7),
                    error: ColorValue::palette(PaletteKey::Red, 6),
                },
                number_input: NumberInputTokens {
                    bg: ColorValue::White,
                    fg: ColorValue::palette(PaletteKey::Dark, 9),
                    placeholder: ColorValue::palette(PaletteKey::Gray, 5),
                    border: ColorValue::palette(PaletteKey::Gray, 4),
                    border_focus: ColorValue::palette(primary, 6),
                    border_error: ColorValue::palette(PaletteKey::Red, 6),
                    controls_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    controls_fg: ColorValue::palette(PaletteKey::Gray, 7),
                    controls_border: ColorValue::palette(PaletteKey::Gray, 3),
                    label: ColorValue::palette(PaletteKey::Dark, 8),
                    description: ColorValue::palette(PaletteKey::Gray, 7),
                    error: ColorValue::palette(PaletteKey::Red, 6),
                },
                range_slider: RangeSliderTokens {
                    track_bg: ColorValue::palette(PaletteKey::Gray, 2),
                    range_bg: ColorValue::palette(primary, 6),
                    thumb_bg: ColorValue::White,
                    thumb_border: ColorValue::palette(primary, 6),
                    label: ColorValue::palette(PaletteKey::Dark, 8),
                    value: ColorValue::palette(PaletteKey::Gray, 7),
                },
                rating: RatingTokens {
                    active: ColorValue::palette(PaletteKey::Yellow, 6),
                    inactive: ColorValue::palette(PaletteKey::Gray, 4),
                },
                tabs: TabsTokens {
                    list_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    list_border: ColorValue::palette(PaletteKey::Gray, 3),
                    tab_fg: ColorValue::palette(PaletteKey::Gray, 7),
                    tab_active_bg: ColorValue::White,
                    tab_active_fg: ColorValue::palette(PaletteKey::Dark, 9),
                    tab_hover_bg: ColorValue::palette(PaletteKey::Gray, 1),
                    tab_disabled_fg: ColorValue::palette(PaletteKey::Gray, 5),
                    panel_bg: ColorValue::White,
                    panel_border: ColorValue::palette(PaletteKey::Gray, 3),
                    panel_fg: ColorValue::palette(PaletteKey::Dark, 8),
                },
                pagination: PaginationTokens {
                    item_bg: ColorValue::White,
                    item_border: ColorValue::palette(PaletteKey::Gray, 3),
                    item_fg: ColorValue::palette(PaletteKey::Dark, 8),
                    item_active_bg: ColorValue::palette(primary, 6),
                    item_active_fg: ColorValue::White,
                    item_hover_bg: ColorValue::palette(PaletteKey::Gray, 1),
                    item_disabled_fg: ColorValue::palette(PaletteKey::Gray, 5),
                    dots_fg: ColorValue::palette(PaletteKey::Gray, 6),
                },
                breadcrumbs: BreadcrumbsTokens {
                    item_fg: ColorValue::palette(PaletteKey::Gray, 7),
                    item_current_fg: ColorValue::palette(PaletteKey::Dark, 9),
                    separator: ColorValue::palette(PaletteKey::Gray, 5),
                    item_hover_bg: ColorValue::palette(PaletteKey::Gray, 1),
                },
                table: TableTokens {
                    header_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    header_fg: ColorValue::palette(PaletteKey::Dark, 8),
                    row_bg: ColorValue::White,
                    row_alt_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    row_hover_bg: ColorValue::palette(PaletteKey::Gray, 1),
                    row_border: ColorValue::palette(PaletteKey::Gray, 3),
                    cell_fg: ColorValue::palette(PaletteKey::Dark, 8),
                    caption: ColorValue::palette(PaletteKey::Gray, 6),
                },
                stepper: StepperTokens {
                    step_bg: ColorValue::White,
                    step_border: ColorValue::palette(PaletteKey::Gray, 4),
                    step_fg: ColorValue::palette(PaletteKey::Gray, 7),
                    step_active_bg: ColorValue::palette(primary, 6),
                    step_active_border: ColorValue::palette(primary, 6),
                    step_active_fg: ColorValue::White,
                    step_completed_bg: ColorValue::palette(primary, 5),
                    step_completed_border: ColorValue::palette(primary, 5),
                    step_completed_fg: ColorValue::White,
                    connector: ColorValue::palette(PaletteKey::Gray, 3),
                    label: ColorValue::palette(PaletteKey::Dark, 8),
                    description: ColorValue::palette(PaletteKey::Gray, 6),
                    panel_bg: ColorValue::White,
                    panel_border: ColorValue::palette(PaletteKey::Gray, 3),
                    panel_fg: ColorValue::palette(PaletteKey::Dark, 8),
                },
                timeline: TimelineTokens {
                    bullet_bg: ColorValue::White,
                    bullet_border: ColorValue::palette(PaletteKey::Gray, 4),
                    bullet_fg: ColorValue::palette(PaletteKey::Gray, 7),
                    bullet_active_bg: ColorValue::palette(primary, 6),
                    bullet_active_border: ColorValue::palette(primary, 6),
                    bullet_active_fg: ColorValue::White,
                    line: ColorValue::palette(PaletteKey::Gray, 3),
                    line_active: ColorValue::palette(primary, 5),
                    title: ColorValue::palette(PaletteKey::Dark, 8),
                    title_active: ColorValue::palette(PaletteKey::Dark, 9),
                    body: ColorValue::palette(PaletteKey::Gray, 6),
                    card_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    card_border: ColorValue::palette(PaletteKey::Gray, 3),
                },
                tree: TreeTokens {
                    row_fg: ColorValue::palette(PaletteKey::Dark, 8),
                    row_selected_fg: ColorValue::palette(primary, 7),
                    row_selected_bg: ColorValue::palette(primary, 0),
                    row_hover_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    row_disabled_fg: ColorValue::palette(PaletteKey::Gray, 5),
                    line: ColorValue::palette(PaletteKey::Gray, 3),
                },
            },
            ColorScheme::Dark => Self {
                button: ButtonTokens {
                    filled_bg: ColorValue::palette(primary, 5),
                    filled_fg: ColorValue::White,
                    light_bg: ColorValue::palette(PaletteKey::Dark, 6),
                    light_fg: ColorValue::palette(primary, 2),
                    subtle_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    subtle_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    outline_border: ColorValue::palette(PaletteKey::Dark, 4),
                    outline_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    ghost_fg: ColorValue::palette(PaletteKey::Gray, 2),
                    disabled_bg: ColorValue::palette(PaletteKey::Dark, 6),
                    disabled_fg: ColorValue::palette(PaletteKey::Dark, 3),
                },
                input: InputTokens {
                    bg: ColorValue::palette(PaletteKey::Dark, 8),
                    fg: ColorValue::palette(PaletteKey::Gray, 0),
                    placeholder: ColorValue::palette(PaletteKey::Dark, 2),
                    border: ColorValue::palette(PaletteKey::Dark, 4),
                    border_focus: ColorValue::palette(primary, 5),
                    border_error: ColorValue::palette(PaletteKey::Red, 5),
                    label: ColorValue::palette(PaletteKey::Gray, 1),
                    description: ColorValue::palette(PaletteKey::Gray, 4),
                    error: ColorValue::palette(PaletteKey::Red, 4),
                },
                radio: RadioTokens {
                    control_bg: ColorValue::palette(PaletteKey::Dark, 8),
                    border: ColorValue::palette(PaletteKey::Dark, 4),
                    border_checked: ColorValue::palette(primary, 5),
                    indicator: ColorValue::palette(primary, 4),
                    label: ColorValue::palette(PaletteKey::Gray, 0),
                    description: ColorValue::palette(PaletteKey::Gray, 4),
                },
                checkbox: CheckboxTokens {
                    control_bg: ColorValue::palette(PaletteKey::Dark, 8),
                    control_bg_checked: ColorValue::palette(primary, 5),
                    border: ColorValue::palette(PaletteKey::Dark, 4),
                    border_checked: ColorValue::palette(primary, 5),
                    indicator: ColorValue::White,
                    label: ColorValue::palette(PaletteKey::Gray, 0),
                    description: ColorValue::palette(PaletteKey::Gray, 4),
                },
                switch: SwitchTokens {
                    track_off_bg: ColorValue::palette(PaletteKey::Dark, 3),
                    track_on_bg: ColorValue::palette(primary, 5),
                    thumb_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    label: ColorValue::palette(PaletteKey::Gray, 0),
                    description: ColorValue::palette(PaletteKey::Gray, 4),
                },
                chip: ChipTokens {
                    unchecked_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    unchecked_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    unchecked_border: ColorValue::palette(PaletteKey::Dark, 4),
                    filled_bg: ColorValue::palette(primary, 5),
                    filled_fg: ColorValue::White,
                    light_bg: ColorValue::palette(PaletteKey::Dark, 6),
                    light_fg: ColorValue::palette(primary, 2),
                    subtle_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    subtle_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    outline_border: ColorValue::palette(PaletteKey::Dark, 4),
                    outline_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    ghost_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    default_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    default_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    default_border: ColorValue::palette(PaletteKey::Dark, 4),
                },
                badge: BadgeTokens {
                    filled_bg: ColorValue::palette(primary, 5),
                    filled_fg: ColorValue::White,
                    light_bg: ColorValue::palette(PaletteKey::Dark, 6),
                    light_fg: ColorValue::palette(primary, 2),
                    subtle_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    subtle_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    outline_border: ColorValue::palette(PaletteKey::Dark, 4),
                    outline_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    default_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    default_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    default_border: ColorValue::palette(PaletteKey::Dark, 4),
                },
                accordion: AccordionTokens {
                    item_bg: ColorValue::palette(PaletteKey::Dark, 8),
                    item_border: ColorValue::palette(PaletteKey::Dark, 4),
                    label: ColorValue::palette(PaletteKey::Gray, 0),
                    description: ColorValue::palette(PaletteKey::Gray, 4),
                    content: ColorValue::palette(PaletteKey::Gray, 2),
                    chevron: ColorValue::palette(PaletteKey::Gray, 4),
                },
                menu: MenuTokens {
                    dropdown_bg: ColorValue::palette(PaletteKey::Dark, 8),
                    dropdown_border: ColorValue::palette(PaletteKey::Dark, 4),
                    item_fg: ColorValue::palette(PaletteKey::Gray, 0),
                    item_hover_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    item_disabled_fg: ColorValue::palette(PaletteKey::Dark, 2),
                    icon: ColorValue::palette(PaletteKey::Gray, 4),
                },
                progress: ProgressTokens {
                    track_bg: ColorValue::palette(PaletteKey::Dark, 5),
                    fill_bg: ColorValue::palette(primary, 5),
                    label: ColorValue::palette(PaletteKey::Gray, 2),
                },
                slider: SliderTokens {
                    track_bg: ColorValue::palette(PaletteKey::Dark, 5),
                    fill_bg: ColorValue::palette(primary, 5),
                    thumb_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    thumb_border: ColorValue::palette(primary, 5),
                    label: ColorValue::palette(PaletteKey::Gray, 2),
                    value: ColorValue::palette(PaletteKey::Gray, 4),
                },
                overlay: OverlayTokens {
                    bg: ColorValue::Custom("#000000E6".to_string()),
                },
                loading_overlay: LoadingOverlayTokens {
                    bg: ColorValue::Custom("#000000E6".to_string()),
                    loader_color: ColorValue::palette(primary, 4),
                    label: ColorValue::palette(PaletteKey::Gray, 0),
                },
                popover: PopoverTokens {
                    bg: ColorValue::palette(PaletteKey::Dark, 8),
                    border: ColorValue::palette(PaletteKey::Dark, 4),
                    title: ColorValue::palette(PaletteKey::Gray, 0),
                    body: ColorValue::palette(PaletteKey::Gray, 3),
                },
                tooltip: TooltipTokens {
                    bg: ColorValue::palette(PaletteKey::Dark, 9),
                    fg: ColorValue::palette(PaletteKey::Gray, 0),
                    border: ColorValue::palette(PaletteKey::Dark, 4),
                },
                hover_card: HoverCardTokens {
                    bg: ColorValue::palette(PaletteKey::Dark, 8),
                    border: ColorValue::palette(PaletteKey::Dark, 4),
                    title: ColorValue::palette(PaletteKey::Gray, 0),
                    body: ColorValue::palette(PaletteKey::Gray, 3),
                },
                select: SelectTokens {
                    bg: ColorValue::palette(PaletteKey::Dark, 8),
                    fg: ColorValue::palette(PaletteKey::Gray, 0),
                    placeholder: ColorValue::palette(PaletteKey::Dark, 2),
                    border: ColorValue::palette(PaletteKey::Dark, 4),
                    border_focus: ColorValue::palette(primary, 5),
                    border_error: ColorValue::palette(PaletteKey::Red, 5),
                    dropdown_bg: ColorValue::palette(PaletteKey::Dark, 8),
                    dropdown_border: ColorValue::palette(PaletteKey::Dark, 4),
                    option_fg: ColorValue::palette(PaletteKey::Gray, 0),
                    option_hover_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    option_selected_bg: ColorValue::palette(primary, 9),
                    tag_bg: ColorValue::palette(primary, 9),
                    tag_fg: ColorValue::palette(primary, 2),
                    tag_border: ColorValue::palette(primary, 7),
                    icon: ColorValue::palette(PaletteKey::Gray, 4),
                    label: ColorValue::palette(PaletteKey::Gray, 1),
                    description: ColorValue::palette(PaletteKey::Gray, 4),
                    error: ColorValue::palette(PaletteKey::Red, 4),
                },
                modal: ModalTokens {
                    panel_bg: ColorValue::palette(PaletteKey::Dark, 8),
                    panel_border: ColorValue::palette(PaletteKey::Dark, 4),
                    overlay_bg: ColorValue::Custom("#000000E6".to_string()),
                    title: ColorValue::palette(PaletteKey::Gray, 0),
                    body: ColorValue::palette(PaletteKey::Gray, 3),
                },
                toast: ToastTokens {
                    info_bg: ColorValue::palette(PaletteKey::Blue, 9),
                    info_fg: ColorValue::palette(PaletteKey::Blue, 2),
                    success_bg: ColorValue::palette(PaletteKey::Green, 9),
                    success_fg: ColorValue::palette(PaletteKey::Green, 2),
                    warning_bg: ColorValue::palette(PaletteKey::Yellow, 9),
                    warning_fg: ColorValue::palette(PaletteKey::Yellow, 3),
                    error_bg: ColorValue::palette(PaletteKey::Red, 9),
                    error_fg: ColorValue::palette(PaletteKey::Red, 2),
                },
                divider: DividerTokens {
                    line: ColorValue::palette(PaletteKey::Dark, 4),
                    label: ColorValue::palette(PaletteKey::Gray, 5),
                },
                scroll_area: ScrollAreaTokens {
                    bg: ColorValue::palette(PaletteKey::Dark, 8),
                    border: ColorValue::palette(PaletteKey::Dark, 5),
                },
                drawer: DrawerTokens {
                    panel_bg: ColorValue::palette(PaletteKey::Dark, 8),
                    panel_border: ColorValue::palette(PaletteKey::Dark, 4),
                    overlay_bg: ColorValue::Custom("#000000E6".to_string()),
                    title: ColorValue::palette(PaletteKey::Gray, 0),
                    body: ColorValue::palette(PaletteKey::Gray, 3),
                },
                app_shell: AppShellTokens {
                    bg: ColorValue::palette(PaletteKey::Dark, 9),
                },
                title_bar: TitleBarTokens {
                    bg: ColorValue::Custom("#00000000".to_string()),
                    border: ColorValue::palette(PaletteKey::Dark, 4),
                    fg: ColorValue::palette(PaletteKey::Gray, 1),
                    controls_bg: ColorValue::palette(PaletteKey::Dark, 6),
                },
                sidebar: SidebarTokens {
                    bg: ColorValue::palette(PaletteKey::Dark, 8),
                    border: ColorValue::palette(PaletteKey::Dark, 4),
                    header_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    content_fg: ColorValue::palette(PaletteKey::Gray, 2),
                    footer_fg: ColorValue::palette(PaletteKey::Gray, 4),
                },
                text: TextTokens {
                    fg: ColorValue::palette(PaletteKey::Gray, 0),
                    secondary: ColorValue::palette(PaletteKey::Gray, 3),
                    muted: ColorValue::palette(PaletteKey::Gray, 5),
                    accent: ColorValue::palette(primary, 3),
                    success: ColorValue::palette(PaletteKey::Green, 4),
                    warning: ColorValue::palette(PaletteKey::Yellow, 4),
                    error: ColorValue::palette(PaletteKey::Red, 4),
                },
                title: TitleTokens {
                    fg: ColorValue::palette(PaletteKey::Gray, 0),
                    subtitle: ColorValue::palette(PaletteKey::Gray, 4),
                },
                paper: PaperTokens {
                    bg: ColorValue::palette(PaletteKey::Dark, 8),
                    border: ColorValue::palette(PaletteKey::Dark, 4),
                },
                action_icon: ActionIconTokens {
                    filled_bg: ColorValue::palette(primary, 5),
                    filled_fg: ColorValue::White,
                    light_bg: ColorValue::palette(PaletteKey::Dark, 6),
                    light_fg: ColorValue::palette(primary, 2),
                    subtle_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    subtle_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    outline_border: ColorValue::palette(PaletteKey::Dark, 4),
                    outline_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    ghost_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    default_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    default_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    default_border: ColorValue::palette(PaletteKey::Dark, 4),
                    disabled_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    disabled_fg: ColorValue::palette(PaletteKey::Dark, 3),
                    disabled_border: ColorValue::palette(PaletteKey::Dark, 5),
                },
                segmented_control: SegmentedControlTokens {
                    bg: ColorValue::palette(PaletteKey::Dark, 8),
                    border: ColorValue::palette(PaletteKey::Dark, 4),
                    item_fg: ColorValue::palette(PaletteKey::Gray, 2),
                    item_active_bg: ColorValue::palette(PaletteKey::Dark, 6),
                    item_active_fg: ColorValue::palette(PaletteKey::Gray, 0),
                    item_hover_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    item_disabled_fg: ColorValue::palette(PaletteKey::Dark, 3),
                },
                textarea: TextareaTokens {
                    bg: ColorValue::palette(PaletteKey::Dark, 8),
                    fg: ColorValue::palette(PaletteKey::Gray, 0),
                    placeholder: ColorValue::palette(PaletteKey::Dark, 2),
                    border: ColorValue::palette(PaletteKey::Dark, 4),
                    border_focus: ColorValue::palette(primary, 5),
                    border_error: ColorValue::palette(PaletteKey::Red, 5),
                    label: ColorValue::palette(PaletteKey::Gray, 1),
                    description: ColorValue::palette(PaletteKey::Gray, 4),
                    error: ColorValue::palette(PaletteKey::Red, 4),
                },
                number_input: NumberInputTokens {
                    bg: ColorValue::palette(PaletteKey::Dark, 8),
                    fg: ColorValue::palette(PaletteKey::Gray, 0),
                    placeholder: ColorValue::palette(PaletteKey::Dark, 2),
                    border: ColorValue::palette(PaletteKey::Dark, 4),
                    border_focus: ColorValue::palette(primary, 5),
                    border_error: ColorValue::palette(PaletteKey::Red, 5),
                    controls_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    controls_fg: ColorValue::palette(PaletteKey::Gray, 3),
                    controls_border: ColorValue::palette(PaletteKey::Dark, 4),
                    label: ColorValue::palette(PaletteKey::Gray, 1),
                    description: ColorValue::palette(PaletteKey::Gray, 4),
                    error: ColorValue::palette(PaletteKey::Red, 4),
                },
                range_slider: RangeSliderTokens {
                    track_bg: ColorValue::palette(PaletteKey::Dark, 5),
                    range_bg: ColorValue::palette(primary, 5),
                    thumb_bg: ColorValue::palette(PaletteKey::Gray, 0),
                    thumb_border: ColorValue::palette(primary, 5),
                    label: ColorValue::palette(PaletteKey::Gray, 2),
                    value: ColorValue::palette(PaletteKey::Gray, 4),
                },
                rating: RatingTokens {
                    active: ColorValue::palette(PaletteKey::Yellow, 4),
                    inactive: ColorValue::palette(PaletteKey::Dark, 3),
                },
                tabs: TabsTokens {
                    list_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    list_border: ColorValue::palette(PaletteKey::Dark, 4),
                    tab_fg: ColorValue::palette(PaletteKey::Gray, 2),
                    tab_active_bg: ColorValue::palette(PaletteKey::Dark, 5),
                    tab_active_fg: ColorValue::palette(PaletteKey::Gray, 0),
                    tab_hover_bg: ColorValue::palette(PaletteKey::Dark, 6),
                    tab_disabled_fg: ColorValue::palette(PaletteKey::Dark, 3),
                    panel_bg: ColorValue::palette(PaletteKey::Dark, 8),
                    panel_border: ColorValue::palette(PaletteKey::Dark, 4),
                    panel_fg: ColorValue::palette(PaletteKey::Gray, 2),
                },
                pagination: PaginationTokens {
                    item_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    item_border: ColorValue::palette(PaletteKey::Dark, 4),
                    item_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    item_active_bg: ColorValue::palette(primary, 5),
                    item_active_fg: ColorValue::White,
                    item_hover_bg: ColorValue::palette(PaletteKey::Dark, 6),
                    item_disabled_fg: ColorValue::palette(PaletteKey::Dark, 3),
                    dots_fg: ColorValue::palette(PaletteKey::Gray, 5),
                },
                breadcrumbs: BreadcrumbsTokens {
                    item_fg: ColorValue::palette(PaletteKey::Gray, 3),
                    item_current_fg: ColorValue::palette(PaletteKey::Gray, 0),
                    separator: ColorValue::palette(PaletteKey::Dark, 2),
                    item_hover_bg: ColorValue::palette(PaletteKey::Dark, 6),
                },
                table: TableTokens {
                    header_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    header_fg: ColorValue::palette(PaletteKey::Gray, 1),
                    row_bg: ColorValue::palette(PaletteKey::Dark, 8),
                    row_alt_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    row_hover_bg: ColorValue::palette(PaletteKey::Dark, 6),
                    row_border: ColorValue::palette(PaletteKey::Dark, 4),
                    cell_fg: ColorValue::palette(PaletteKey::Gray, 2),
                    caption: ColorValue::palette(PaletteKey::Gray, 5),
                },
                stepper: StepperTokens {
                    step_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    step_border: ColorValue::palette(PaletteKey::Dark, 4),
                    step_fg: ColorValue::palette(PaletteKey::Gray, 3),
                    step_active_bg: ColorValue::palette(primary, 5),
                    step_active_border: ColorValue::palette(primary, 5),
                    step_active_fg: ColorValue::White,
                    step_completed_bg: ColorValue::palette(primary, 4),
                    step_completed_border: ColorValue::palette(primary, 4),
                    step_completed_fg: ColorValue::White,
                    connector: ColorValue::palette(PaletteKey::Dark, 4),
                    label: ColorValue::palette(PaletteKey::Gray, 1),
                    description: ColorValue::palette(PaletteKey::Gray, 5),
                    panel_bg: ColorValue::palette(PaletteKey::Dark, 8),
                    panel_border: ColorValue::palette(PaletteKey::Dark, 4),
                    panel_fg: ColorValue::palette(PaletteKey::Gray, 2),
                },
                timeline: TimelineTokens {
                    bullet_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    bullet_border: ColorValue::palette(PaletteKey::Dark, 4),
                    bullet_fg: ColorValue::palette(PaletteKey::Gray, 4),
                    bullet_active_bg: ColorValue::palette(primary, 5),
                    bullet_active_border: ColorValue::palette(primary, 5),
                    bullet_active_fg: ColorValue::White,
                    line: ColorValue::palette(PaletteKey::Dark, 4),
                    line_active: ColorValue::palette(primary, 4),
                    title: ColorValue::palette(PaletteKey::Gray, 2),
                    title_active: ColorValue::palette(PaletteKey::Gray, 0),
                    body: ColorValue::palette(PaletteKey::Gray, 5),
                    card_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    card_border: ColorValue::palette(PaletteKey::Dark, 4),
                },
                tree: TreeTokens {
                    row_fg: ColorValue::palette(PaletteKey::Gray, 2),
                    row_selected_fg: ColorValue::palette(primary, 1),
                    row_selected_bg: ColorValue::palette(primary, 9),
                    row_hover_bg: ColorValue::palette(PaletteKey::Dark, 7),
                    row_disabled_fg: ColorValue::palette(PaletteKey::Dark, 3),
                    line: ColorValue::palette(PaletteKey::Dark, 4),
                },
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Theme {
    pub white: &'static str,
    pub black: &'static str,
    pub radius_default: &'static str,
    pub primary_color: PaletteKey,
    pub primary_shade: PrimaryShade,
    pub color_scheme: ColorScheme,
    pub palette: BTreeMap<PaletteKey, ColorScale>,
    pub semantic: SemanticColors,
    pub components: ComponentTokens,
}

impl Default for Theme {
    fn default() -> Self {
        let primary = PaletteKey::Blue;
        Self {
            white: "#fff",
            black: "#000",
            radius_default: "0.25rem",
            primary_color: primary,
            primary_shade: PrimaryShade::Split { light: 6, dark: 8 },
            color_scheme: ColorScheme::Light,
            palette: PaletteCatalog::store(),
            semantic: SemanticColors::defaults_for(primary, ColorScheme::Light),
            components: ComponentTokens::defaults_for(primary, ColorScheme::Light),
        }
    }
}

impl Theme {
    pub fn with_primary_color(mut self, primary: PaletteKey) -> Self {
        self.primary_color = primary;
        self.semantic = SemanticColors::defaults_for(primary, self.color_scheme);
        self.components = ComponentTokens::defaults_for(primary, self.color_scheme);
        self
    }

    pub fn with_primary_shade(mut self, primary_shade: PrimaryShade) -> Self {
        self.primary_shade = primary_shade;
        self
    }

    pub fn with_color_scheme(mut self, scheme: ColorScheme) -> Self {
        self.color_scheme = scheme;
        self.semantic = SemanticColors::defaults_for(self.primary_color, scheme);
        self.components = ComponentTokens::defaults_for(self.primary_color, scheme);
        self
    }

    pub fn with_palette_override(mut self, key: PaletteKey, scale: ColorScale) -> Self {
        self.palette.insert(key, scale);
        self
    }

    pub fn with_accent_color(self, accent: PaletteKey) -> Self {
        self.with_primary_color(accent)
    }

    pub fn resolve_color(&self, token: &ColorValue) -> String {
        match token {
            ColorValue::Palette { key, shade } => self
                .palette
                .get(key)
                .and_then(|scale| scale.get(*shade as usize))
                .unwrap_or(&self.black)
                .to_string(),
            ColorValue::White => self.white.to_string(),
            ColorValue::Black => self.black.to_string(),
            ColorValue::Custom(value) => value.clone(),
        }
    }

    pub fn resolve_hsla(&self, token: &ColorValue) -> gpui::Hsla {
        let raw = self.resolve_color(token);
        gpui::Rgba::try_from(raw.as_str())
            .map(Into::into)
            .unwrap_or_else(|_| gpui::black())
    }

    pub fn merged(&self, patch: &ThemePatch) -> Self {
        let mut next = self.clone();
        if let Some(primary) = patch.primary_color {
            next = next.with_primary_color(primary);
        }
        if let Some(primary_shade) = patch.primary_shade {
            next.primary_shade = primary_shade;
        }
        if let Some(color_scheme) = patch.color_scheme {
            next.color_scheme = color_scheme;
        }
        for (key, value) in &patch.palette_overrides {
            next.palette.insert(*key, *value);
        }
        next.semantic = patch.semantic.apply(next.semantic);
        next.components = patch.components.apply(next.components);
        next
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticPatch {
    pub text_primary: Option<ColorValue>,
    pub text_secondary: Option<ColorValue>,
    pub text_muted: Option<ColorValue>,
    pub bg_canvas: Option<ColorValue>,
    pub bg_surface: Option<ColorValue>,
    pub bg_soft: Option<ColorValue>,
    pub border_subtle: Option<ColorValue>,
    pub border_strong: Option<ColorValue>,
    pub focus_ring: Option<ColorValue>,
    pub status_info: Option<ColorValue>,
    pub status_success: Option<ColorValue>,
    pub status_warning: Option<ColorValue>,
    pub status_error: Option<ColorValue>,
    pub overlay_mask: Option<ColorValue>,
}

impl SemanticPatch {
    fn apply(&self, mut current: SemanticColors) -> SemanticColors {
        if let Some(value) = &self.text_primary {
            current.text_primary = value.clone();
        }
        if let Some(value) = &self.text_secondary {
            current.text_secondary = value.clone();
        }
        if let Some(value) = &self.text_muted {
            current.text_muted = value.clone();
        }
        if let Some(value) = &self.bg_canvas {
            current.bg_canvas = value.clone();
        }
        if let Some(value) = &self.bg_surface {
            current.bg_surface = value.clone();
        }
        if let Some(value) = &self.bg_soft {
            current.bg_soft = value.clone();
        }
        if let Some(value) = &self.border_subtle {
            current.border_subtle = value.clone();
        }
        if let Some(value) = &self.border_strong {
            current.border_strong = value.clone();
        }
        if let Some(value) = &self.focus_ring {
            current.focus_ring = value.clone();
        }
        if let Some(value) = &self.status_info {
            current.status_info = value.clone();
        }
        if let Some(value) = &self.status_success {
            current.status_success = value.clone();
        }
        if let Some(value) = &self.status_warning {
            current.status_warning = value.clone();
        }
        if let Some(value) = &self.status_error {
            current.status_error = value.clone();
        }
        if let Some(value) = &self.overlay_mask {
            current.overlay_mask = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ButtonPatch {
    pub filled_bg: Option<ColorValue>,
    pub filled_fg: Option<ColorValue>,
    pub light_bg: Option<ColorValue>,
    pub light_fg: Option<ColorValue>,
    pub subtle_bg: Option<ColorValue>,
    pub subtle_fg: Option<ColorValue>,
    pub outline_border: Option<ColorValue>,
    pub outline_fg: Option<ColorValue>,
    pub ghost_fg: Option<ColorValue>,
    pub disabled_bg: Option<ColorValue>,
    pub disabled_fg: Option<ColorValue>,
}

impl ButtonPatch {
    fn apply(&self, mut current: ButtonTokens) -> ButtonTokens {
        if let Some(value) = &self.filled_bg {
            current.filled_bg = value.clone();
        }
        if let Some(value) = &self.filled_fg {
            current.filled_fg = value.clone();
        }
        if let Some(value) = &self.light_bg {
            current.light_bg = value.clone();
        }
        if let Some(value) = &self.light_fg {
            current.light_fg = value.clone();
        }
        if let Some(value) = &self.subtle_bg {
            current.subtle_bg = value.clone();
        }
        if let Some(value) = &self.subtle_fg {
            current.subtle_fg = value.clone();
        }
        if let Some(value) = &self.outline_border {
            current.outline_border = value.clone();
        }
        if let Some(value) = &self.outline_fg {
            current.outline_fg = value.clone();
        }
        if let Some(value) = &self.ghost_fg {
            current.ghost_fg = value.clone();
        }
        if let Some(value) = &self.disabled_bg {
            current.disabled_bg = value.clone();
        }
        if let Some(value) = &self.disabled_fg {
            current.disabled_fg = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InputPatch {
    pub bg: Option<ColorValue>,
    pub fg: Option<ColorValue>,
    pub placeholder: Option<ColorValue>,
    pub border: Option<ColorValue>,
    pub border_focus: Option<ColorValue>,
    pub border_error: Option<ColorValue>,
    pub label: Option<ColorValue>,
    pub description: Option<ColorValue>,
    pub error: Option<ColorValue>,
}

impl InputPatch {
    fn apply(&self, mut current: InputTokens) -> InputTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.fg {
            current.fg = value.clone();
        }
        if let Some(value) = &self.placeholder {
            current.placeholder = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = &self.border_focus {
            current.border_focus = value.clone();
        }
        if let Some(value) = &self.border_error {
            current.border_error = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        if let Some(value) = &self.description {
            current.description = value.clone();
        }
        if let Some(value) = &self.error {
            current.error = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RadioPatch {
    pub control_bg: Option<ColorValue>,
    pub border: Option<ColorValue>,
    pub border_checked: Option<ColorValue>,
    pub indicator: Option<ColorValue>,
    pub label: Option<ColorValue>,
    pub description: Option<ColorValue>,
}

impl RadioPatch {
    fn apply(&self, mut current: RadioTokens) -> RadioTokens {
        if let Some(value) = &self.control_bg {
            current.control_bg = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = &self.border_checked {
            current.border_checked = value.clone();
        }
        if let Some(value) = &self.indicator {
            current.indicator = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        if let Some(value) = &self.description {
            current.description = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckboxPatch {
    pub control_bg: Option<ColorValue>,
    pub control_bg_checked: Option<ColorValue>,
    pub border: Option<ColorValue>,
    pub border_checked: Option<ColorValue>,
    pub indicator: Option<ColorValue>,
    pub label: Option<ColorValue>,
    pub description: Option<ColorValue>,
}

impl CheckboxPatch {
    fn apply(&self, mut current: CheckboxTokens) -> CheckboxTokens {
        if let Some(value) = &self.control_bg {
            current.control_bg = value.clone();
        }
        if let Some(value) = &self.control_bg_checked {
            current.control_bg_checked = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = &self.border_checked {
            current.border_checked = value.clone();
        }
        if let Some(value) = &self.indicator {
            current.indicator = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        if let Some(value) = &self.description {
            current.description = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SwitchPatch {
    pub track_off_bg: Option<ColorValue>,
    pub track_on_bg: Option<ColorValue>,
    pub thumb_bg: Option<ColorValue>,
    pub label: Option<ColorValue>,
    pub description: Option<ColorValue>,
}

impl SwitchPatch {
    fn apply(&self, mut current: SwitchTokens) -> SwitchTokens {
        if let Some(value) = &self.track_off_bg {
            current.track_off_bg = value.clone();
        }
        if let Some(value) = &self.track_on_bg {
            current.track_on_bg = value.clone();
        }
        if let Some(value) = &self.thumb_bg {
            current.thumb_bg = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        if let Some(value) = &self.description {
            current.description = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ChipPatch {
    pub unchecked_bg: Option<ColorValue>,
    pub unchecked_fg: Option<ColorValue>,
    pub unchecked_border: Option<ColorValue>,
    pub filled_bg: Option<ColorValue>,
    pub filled_fg: Option<ColorValue>,
    pub light_bg: Option<ColorValue>,
    pub light_fg: Option<ColorValue>,
    pub subtle_bg: Option<ColorValue>,
    pub subtle_fg: Option<ColorValue>,
    pub outline_border: Option<ColorValue>,
    pub outline_fg: Option<ColorValue>,
    pub ghost_fg: Option<ColorValue>,
    pub default_bg: Option<ColorValue>,
    pub default_fg: Option<ColorValue>,
    pub default_border: Option<ColorValue>,
}

impl ChipPatch {
    fn apply(&self, mut current: ChipTokens) -> ChipTokens {
        if let Some(value) = &self.unchecked_bg {
            current.unchecked_bg = value.clone();
        }
        if let Some(value) = &self.unchecked_fg {
            current.unchecked_fg = value.clone();
        }
        if let Some(value) = &self.unchecked_border {
            current.unchecked_border = value.clone();
        }
        if let Some(value) = &self.filled_bg {
            current.filled_bg = value.clone();
        }
        if let Some(value) = &self.filled_fg {
            current.filled_fg = value.clone();
        }
        if let Some(value) = &self.light_bg {
            current.light_bg = value.clone();
        }
        if let Some(value) = &self.light_fg {
            current.light_fg = value.clone();
        }
        if let Some(value) = &self.subtle_bg {
            current.subtle_bg = value.clone();
        }
        if let Some(value) = &self.subtle_fg {
            current.subtle_fg = value.clone();
        }
        if let Some(value) = &self.outline_border {
            current.outline_border = value.clone();
        }
        if let Some(value) = &self.outline_fg {
            current.outline_fg = value.clone();
        }
        if let Some(value) = &self.ghost_fg {
            current.ghost_fg = value.clone();
        }
        if let Some(value) = &self.default_bg {
            current.default_bg = value.clone();
        }
        if let Some(value) = &self.default_fg {
            current.default_fg = value.clone();
        }
        if let Some(value) = &self.default_border {
            current.default_border = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BadgePatch {
    pub filled_bg: Option<ColorValue>,
    pub filled_fg: Option<ColorValue>,
    pub light_bg: Option<ColorValue>,
    pub light_fg: Option<ColorValue>,
    pub subtle_bg: Option<ColorValue>,
    pub subtle_fg: Option<ColorValue>,
    pub outline_border: Option<ColorValue>,
    pub outline_fg: Option<ColorValue>,
    pub default_bg: Option<ColorValue>,
    pub default_fg: Option<ColorValue>,
    pub default_border: Option<ColorValue>,
}

impl BadgePatch {
    fn apply(&self, mut current: BadgeTokens) -> BadgeTokens {
        if let Some(value) = &self.filled_bg {
            current.filled_bg = value.clone();
        }
        if let Some(value) = &self.filled_fg {
            current.filled_fg = value.clone();
        }
        if let Some(value) = &self.light_bg {
            current.light_bg = value.clone();
        }
        if let Some(value) = &self.light_fg {
            current.light_fg = value.clone();
        }
        if let Some(value) = &self.subtle_bg {
            current.subtle_bg = value.clone();
        }
        if let Some(value) = &self.subtle_fg {
            current.subtle_fg = value.clone();
        }
        if let Some(value) = &self.outline_border {
            current.outline_border = value.clone();
        }
        if let Some(value) = &self.outline_fg {
            current.outline_fg = value.clone();
        }
        if let Some(value) = &self.default_bg {
            current.default_bg = value.clone();
        }
        if let Some(value) = &self.default_fg {
            current.default_fg = value.clone();
        }
        if let Some(value) = &self.default_border {
            current.default_border = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AccordionPatch {
    pub item_bg: Option<ColorValue>,
    pub item_border: Option<ColorValue>,
    pub label: Option<ColorValue>,
    pub description: Option<ColorValue>,
    pub content: Option<ColorValue>,
    pub chevron: Option<ColorValue>,
}

impl AccordionPatch {
    fn apply(&self, mut current: AccordionTokens) -> AccordionTokens {
        if let Some(value) = &self.item_bg {
            current.item_bg = value.clone();
        }
        if let Some(value) = &self.item_border {
            current.item_border = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        if let Some(value) = &self.description {
            current.description = value.clone();
        }
        if let Some(value) = &self.content {
            current.content = value.clone();
        }
        if let Some(value) = &self.chevron {
            current.chevron = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MenuPatch {
    pub dropdown_bg: Option<ColorValue>,
    pub dropdown_border: Option<ColorValue>,
    pub item_fg: Option<ColorValue>,
    pub item_hover_bg: Option<ColorValue>,
    pub item_disabled_fg: Option<ColorValue>,
    pub icon: Option<ColorValue>,
}

impl MenuPatch {
    fn apply(&self, mut current: MenuTokens) -> MenuTokens {
        if let Some(value) = &self.dropdown_bg {
            current.dropdown_bg = value.clone();
        }
        if let Some(value) = &self.dropdown_border {
            current.dropdown_border = value.clone();
        }
        if let Some(value) = &self.item_fg {
            current.item_fg = value.clone();
        }
        if let Some(value) = &self.item_hover_bg {
            current.item_hover_bg = value.clone();
        }
        if let Some(value) = &self.item_disabled_fg {
            current.item_disabled_fg = value.clone();
        }
        if let Some(value) = &self.icon {
            current.icon = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProgressPatch {
    pub track_bg: Option<ColorValue>,
    pub fill_bg: Option<ColorValue>,
    pub label: Option<ColorValue>,
}

impl ProgressPatch {
    fn apply(&self, mut current: ProgressTokens) -> ProgressTokens {
        if let Some(value) = &self.track_bg {
            current.track_bg = value.clone();
        }
        if let Some(value) = &self.fill_bg {
            current.fill_bg = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SliderPatch {
    pub track_bg: Option<ColorValue>,
    pub fill_bg: Option<ColorValue>,
    pub thumb_bg: Option<ColorValue>,
    pub thumb_border: Option<ColorValue>,
    pub label: Option<ColorValue>,
    pub value: Option<ColorValue>,
}

impl SliderPatch {
    fn apply(&self, mut current: SliderTokens) -> SliderTokens {
        if let Some(value) = &self.track_bg {
            current.track_bg = value.clone();
        }
        if let Some(value) = &self.fill_bg {
            current.fill_bg = value.clone();
        }
        if let Some(value) = &self.thumb_bg {
            current.thumb_bg = value.clone();
        }
        if let Some(value) = &self.thumb_border {
            current.thumb_border = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        if let Some(value) = &self.value {
            current.value = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OverlayPatch {
    pub bg: Option<ColorValue>,
}

impl OverlayPatch {
    fn apply(&self, mut current: OverlayTokens) -> OverlayTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LoadingOverlayPatch {
    pub bg: Option<ColorValue>,
    pub loader_color: Option<ColorValue>,
    pub label: Option<ColorValue>,
}

impl LoadingOverlayPatch {
    fn apply(&self, mut current: LoadingOverlayTokens) -> LoadingOverlayTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.loader_color {
            current.loader_color = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PopoverPatch {
    pub bg: Option<ColorValue>,
    pub border: Option<ColorValue>,
    pub title: Option<ColorValue>,
    pub body: Option<ColorValue>,
}

impl PopoverPatch {
    fn apply(&self, mut current: PopoverTokens) -> PopoverTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = &self.title {
            current.title = value.clone();
        }
        if let Some(value) = &self.body {
            current.body = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TooltipPatch {
    pub bg: Option<ColorValue>,
    pub fg: Option<ColorValue>,
    pub border: Option<ColorValue>,
}

impl TooltipPatch {
    fn apply(&self, mut current: TooltipTokens) -> TooltipTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.fg {
            current.fg = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HoverCardPatch {
    pub bg: Option<ColorValue>,
    pub border: Option<ColorValue>,
    pub title: Option<ColorValue>,
    pub body: Option<ColorValue>,
}

impl HoverCardPatch {
    fn apply(&self, mut current: HoverCardTokens) -> HoverCardTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = &self.title {
            current.title = value.clone();
        }
        if let Some(value) = &self.body {
            current.body = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SelectPatch {
    pub bg: Option<ColorValue>,
    pub fg: Option<ColorValue>,
    pub placeholder: Option<ColorValue>,
    pub border: Option<ColorValue>,
    pub border_focus: Option<ColorValue>,
    pub border_error: Option<ColorValue>,
    pub dropdown_bg: Option<ColorValue>,
    pub dropdown_border: Option<ColorValue>,
    pub option_fg: Option<ColorValue>,
    pub option_hover_bg: Option<ColorValue>,
    pub option_selected_bg: Option<ColorValue>,
    pub tag_bg: Option<ColorValue>,
    pub tag_fg: Option<ColorValue>,
    pub tag_border: Option<ColorValue>,
    pub icon: Option<ColorValue>,
    pub label: Option<ColorValue>,
    pub description: Option<ColorValue>,
    pub error: Option<ColorValue>,
}

impl SelectPatch {
    fn apply(&self, mut current: SelectTokens) -> SelectTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.fg {
            current.fg = value.clone();
        }
        if let Some(value) = &self.placeholder {
            current.placeholder = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = &self.border_focus {
            current.border_focus = value.clone();
        }
        if let Some(value) = &self.border_error {
            current.border_error = value.clone();
        }
        if let Some(value) = &self.dropdown_bg {
            current.dropdown_bg = value.clone();
        }
        if let Some(value) = &self.dropdown_border {
            current.dropdown_border = value.clone();
        }
        if let Some(value) = &self.option_fg {
            current.option_fg = value.clone();
        }
        if let Some(value) = &self.option_hover_bg {
            current.option_hover_bg = value.clone();
        }
        if let Some(value) = &self.option_selected_bg {
            current.option_selected_bg = value.clone();
        }
        if let Some(value) = &self.tag_bg {
            current.tag_bg = value.clone();
        }
        if let Some(value) = &self.tag_fg {
            current.tag_fg = value.clone();
        }
        if let Some(value) = &self.tag_border {
            current.tag_border = value.clone();
        }
        if let Some(value) = &self.icon {
            current.icon = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        if let Some(value) = &self.description {
            current.description = value.clone();
        }
        if let Some(value) = &self.error {
            current.error = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModalPatch {
    pub panel_bg: Option<ColorValue>,
    pub panel_border: Option<ColorValue>,
    pub overlay_bg: Option<ColorValue>,
    pub title: Option<ColorValue>,
    pub body: Option<ColorValue>,
}

impl ModalPatch {
    fn apply(&self, mut current: ModalTokens) -> ModalTokens {
        if let Some(value) = &self.panel_bg {
            current.panel_bg = value.clone();
        }
        if let Some(value) = &self.panel_border {
            current.panel_border = value.clone();
        }
        if let Some(value) = &self.overlay_bg {
            current.overlay_bg = value.clone();
        }
        if let Some(value) = &self.title {
            current.title = value.clone();
        }
        if let Some(value) = &self.body {
            current.body = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ToastPatch {
    pub info_bg: Option<ColorValue>,
    pub info_fg: Option<ColorValue>,
    pub success_bg: Option<ColorValue>,
    pub success_fg: Option<ColorValue>,
    pub warning_bg: Option<ColorValue>,
    pub warning_fg: Option<ColorValue>,
    pub error_bg: Option<ColorValue>,
    pub error_fg: Option<ColorValue>,
}

impl ToastPatch {
    fn apply(&self, mut current: ToastTokens) -> ToastTokens {
        if let Some(value) = &self.info_bg {
            current.info_bg = value.clone();
        }
        if let Some(value) = &self.info_fg {
            current.info_fg = value.clone();
        }
        if let Some(value) = &self.success_bg {
            current.success_bg = value.clone();
        }
        if let Some(value) = &self.success_fg {
            current.success_fg = value.clone();
        }
        if let Some(value) = &self.warning_bg {
            current.warning_bg = value.clone();
        }
        if let Some(value) = &self.warning_fg {
            current.warning_fg = value.clone();
        }
        if let Some(value) = &self.error_bg {
            current.error_bg = value.clone();
        }
        if let Some(value) = &self.error_fg {
            current.error_fg = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DividerPatch {
    pub line: Option<ColorValue>,
    pub label: Option<ColorValue>,
}

impl DividerPatch {
    fn apply(&self, mut current: DividerTokens) -> DividerTokens {
        if let Some(value) = &self.line {
            current.line = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScrollAreaPatch {
    pub bg: Option<ColorValue>,
    pub border: Option<ColorValue>,
}

impl ScrollAreaPatch {
    fn apply(&self, mut current: ScrollAreaTokens) -> ScrollAreaTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DrawerPatch {
    pub panel_bg: Option<ColorValue>,
    pub panel_border: Option<ColorValue>,
    pub overlay_bg: Option<ColorValue>,
    pub title: Option<ColorValue>,
    pub body: Option<ColorValue>,
}

impl DrawerPatch {
    fn apply(&self, mut current: DrawerTokens) -> DrawerTokens {
        if let Some(value) = &self.panel_bg {
            current.panel_bg = value.clone();
        }
        if let Some(value) = &self.panel_border {
            current.panel_border = value.clone();
        }
        if let Some(value) = &self.overlay_bg {
            current.overlay_bg = value.clone();
        }
        if let Some(value) = &self.title {
            current.title = value.clone();
        }
        if let Some(value) = &self.body {
            current.body = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AppShellPatch {
    pub bg: Option<ColorValue>,
}

impl AppShellPatch {
    fn apply(&self, mut current: AppShellTokens) -> AppShellTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TitleBarPatch {
    pub bg: Option<ColorValue>,
    pub border: Option<ColorValue>,
    pub fg: Option<ColorValue>,
    pub controls_bg: Option<ColorValue>,
}

impl TitleBarPatch {
    fn apply(&self, mut current: TitleBarTokens) -> TitleBarTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = &self.fg {
            current.fg = value.clone();
        }
        if let Some(value) = &self.controls_bg {
            current.controls_bg = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SidebarPatch {
    pub bg: Option<ColorValue>,
    pub border: Option<ColorValue>,
    pub header_fg: Option<ColorValue>,
    pub content_fg: Option<ColorValue>,
    pub footer_fg: Option<ColorValue>,
}

impl SidebarPatch {
    fn apply(&self, mut current: SidebarTokens) -> SidebarTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = &self.header_fg {
            current.header_fg = value.clone();
        }
        if let Some(value) = &self.content_fg {
            current.content_fg = value.clone();
        }
        if let Some(value) = &self.footer_fg {
            current.footer_fg = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TextPatch {
    pub fg: Option<ColorValue>,
    pub secondary: Option<ColorValue>,
    pub muted: Option<ColorValue>,
    pub accent: Option<ColorValue>,
    pub success: Option<ColorValue>,
    pub warning: Option<ColorValue>,
    pub error: Option<ColorValue>,
}

impl TextPatch {
    fn apply(&self, mut current: TextTokens) -> TextTokens {
        if let Some(value) = &self.fg {
            current.fg = value.clone();
        }
        if let Some(value) = &self.secondary {
            current.secondary = value.clone();
        }
        if let Some(value) = &self.muted {
            current.muted = value.clone();
        }
        if let Some(value) = &self.accent {
            current.accent = value.clone();
        }
        if let Some(value) = &self.success {
            current.success = value.clone();
        }
        if let Some(value) = &self.warning {
            current.warning = value.clone();
        }
        if let Some(value) = &self.error {
            current.error = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TitlePatch {
    pub fg: Option<ColorValue>,
    pub subtitle: Option<ColorValue>,
}

impl TitlePatch {
    fn apply(&self, mut current: TitleTokens) -> TitleTokens {
        if let Some(value) = &self.fg {
            current.fg = value.clone();
        }
        if let Some(value) = &self.subtitle {
            current.subtitle = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PaperPatch {
    pub bg: Option<ColorValue>,
    pub border: Option<ColorValue>,
}

impl PaperPatch {
    fn apply(&self, mut current: PaperTokens) -> PaperTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActionIconPatch {
    pub filled_bg: Option<ColorValue>,
    pub filled_fg: Option<ColorValue>,
    pub light_bg: Option<ColorValue>,
    pub light_fg: Option<ColorValue>,
    pub subtle_bg: Option<ColorValue>,
    pub subtle_fg: Option<ColorValue>,
    pub outline_border: Option<ColorValue>,
    pub outline_fg: Option<ColorValue>,
    pub ghost_fg: Option<ColorValue>,
    pub default_bg: Option<ColorValue>,
    pub default_fg: Option<ColorValue>,
    pub default_border: Option<ColorValue>,
    pub disabled_bg: Option<ColorValue>,
    pub disabled_fg: Option<ColorValue>,
    pub disabled_border: Option<ColorValue>,
}

impl ActionIconPatch {
    fn apply(&self, mut current: ActionIconTokens) -> ActionIconTokens {
        if let Some(value) = &self.filled_bg {
            current.filled_bg = value.clone();
        }
        if let Some(value) = &self.filled_fg {
            current.filled_fg = value.clone();
        }
        if let Some(value) = &self.light_bg {
            current.light_bg = value.clone();
        }
        if let Some(value) = &self.light_fg {
            current.light_fg = value.clone();
        }
        if let Some(value) = &self.subtle_bg {
            current.subtle_bg = value.clone();
        }
        if let Some(value) = &self.subtle_fg {
            current.subtle_fg = value.clone();
        }
        if let Some(value) = &self.outline_border {
            current.outline_border = value.clone();
        }
        if let Some(value) = &self.outline_fg {
            current.outline_fg = value.clone();
        }
        if let Some(value) = &self.ghost_fg {
            current.ghost_fg = value.clone();
        }
        if let Some(value) = &self.default_bg {
            current.default_bg = value.clone();
        }
        if let Some(value) = &self.default_fg {
            current.default_fg = value.clone();
        }
        if let Some(value) = &self.default_border {
            current.default_border = value.clone();
        }
        if let Some(value) = &self.disabled_bg {
            current.disabled_bg = value.clone();
        }
        if let Some(value) = &self.disabled_fg {
            current.disabled_fg = value.clone();
        }
        if let Some(value) = &self.disabled_border {
            current.disabled_border = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SegmentedControlPatch {
    pub bg: Option<ColorValue>,
    pub border: Option<ColorValue>,
    pub item_fg: Option<ColorValue>,
    pub item_active_bg: Option<ColorValue>,
    pub item_active_fg: Option<ColorValue>,
    pub item_hover_bg: Option<ColorValue>,
    pub item_disabled_fg: Option<ColorValue>,
}

impl SegmentedControlPatch {
    fn apply(&self, mut current: SegmentedControlTokens) -> SegmentedControlTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = &self.item_fg {
            current.item_fg = value.clone();
        }
        if let Some(value) = &self.item_active_bg {
            current.item_active_bg = value.clone();
        }
        if let Some(value) = &self.item_active_fg {
            current.item_active_fg = value.clone();
        }
        if let Some(value) = &self.item_hover_bg {
            current.item_hover_bg = value.clone();
        }
        if let Some(value) = &self.item_disabled_fg {
            current.item_disabled_fg = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TextareaPatch {
    pub bg: Option<ColorValue>,
    pub fg: Option<ColorValue>,
    pub placeholder: Option<ColorValue>,
    pub border: Option<ColorValue>,
    pub border_focus: Option<ColorValue>,
    pub border_error: Option<ColorValue>,
    pub label: Option<ColorValue>,
    pub description: Option<ColorValue>,
    pub error: Option<ColorValue>,
}

impl TextareaPatch {
    fn apply(&self, mut current: TextareaTokens) -> TextareaTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.fg {
            current.fg = value.clone();
        }
        if let Some(value) = &self.placeholder {
            current.placeholder = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = &self.border_focus {
            current.border_focus = value.clone();
        }
        if let Some(value) = &self.border_error {
            current.border_error = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        if let Some(value) = &self.description {
            current.description = value.clone();
        }
        if let Some(value) = &self.error {
            current.error = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NumberInputPatch {
    pub bg: Option<ColorValue>,
    pub fg: Option<ColorValue>,
    pub placeholder: Option<ColorValue>,
    pub border: Option<ColorValue>,
    pub border_focus: Option<ColorValue>,
    pub border_error: Option<ColorValue>,
    pub controls_bg: Option<ColorValue>,
    pub controls_fg: Option<ColorValue>,
    pub controls_border: Option<ColorValue>,
    pub label: Option<ColorValue>,
    pub description: Option<ColorValue>,
    pub error: Option<ColorValue>,
}

impl NumberInputPatch {
    fn apply(&self, mut current: NumberInputTokens) -> NumberInputTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.fg {
            current.fg = value.clone();
        }
        if let Some(value) = &self.placeholder {
            current.placeholder = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = &self.border_focus {
            current.border_focus = value.clone();
        }
        if let Some(value) = &self.border_error {
            current.border_error = value.clone();
        }
        if let Some(value) = &self.controls_bg {
            current.controls_bg = value.clone();
        }
        if let Some(value) = &self.controls_fg {
            current.controls_fg = value.clone();
        }
        if let Some(value) = &self.controls_border {
            current.controls_border = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        if let Some(value) = &self.description {
            current.description = value.clone();
        }
        if let Some(value) = &self.error {
            current.error = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RangeSliderPatch {
    pub track_bg: Option<ColorValue>,
    pub range_bg: Option<ColorValue>,
    pub thumb_bg: Option<ColorValue>,
    pub thumb_border: Option<ColorValue>,
    pub label: Option<ColorValue>,
    pub value: Option<ColorValue>,
}

impl RangeSliderPatch {
    fn apply(&self, mut current: RangeSliderTokens) -> RangeSliderTokens {
        if let Some(value) = &self.track_bg {
            current.track_bg = value.clone();
        }
        if let Some(value) = &self.range_bg {
            current.range_bg = value.clone();
        }
        if let Some(value) = &self.thumb_bg {
            current.thumb_bg = value.clone();
        }
        if let Some(value) = &self.thumb_border {
            current.thumb_border = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        if let Some(value) = &self.value {
            current.value = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RatingPatch {
    pub active: Option<ColorValue>,
    pub inactive: Option<ColorValue>,
}

impl RatingPatch {
    fn apply(&self, mut current: RatingTokens) -> RatingTokens {
        if let Some(value) = &self.active {
            current.active = value.clone();
        }
        if let Some(value) = &self.inactive {
            current.inactive = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TabsPatch {
    pub list_bg: Option<ColorValue>,
    pub list_border: Option<ColorValue>,
    pub tab_fg: Option<ColorValue>,
    pub tab_active_bg: Option<ColorValue>,
    pub tab_active_fg: Option<ColorValue>,
    pub tab_hover_bg: Option<ColorValue>,
    pub tab_disabled_fg: Option<ColorValue>,
    pub panel_bg: Option<ColorValue>,
    pub panel_border: Option<ColorValue>,
    pub panel_fg: Option<ColorValue>,
}

impl TabsPatch {
    fn apply(&self, mut current: TabsTokens) -> TabsTokens {
        if let Some(value) = &self.list_bg {
            current.list_bg = value.clone();
        }
        if let Some(value) = &self.list_border {
            current.list_border = value.clone();
        }
        if let Some(value) = &self.tab_fg {
            current.tab_fg = value.clone();
        }
        if let Some(value) = &self.tab_active_bg {
            current.tab_active_bg = value.clone();
        }
        if let Some(value) = &self.tab_active_fg {
            current.tab_active_fg = value.clone();
        }
        if let Some(value) = &self.tab_hover_bg {
            current.tab_hover_bg = value.clone();
        }
        if let Some(value) = &self.tab_disabled_fg {
            current.tab_disabled_fg = value.clone();
        }
        if let Some(value) = &self.panel_bg {
            current.panel_bg = value.clone();
        }
        if let Some(value) = &self.panel_border {
            current.panel_border = value.clone();
        }
        if let Some(value) = &self.panel_fg {
            current.panel_fg = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PaginationPatch {
    pub item_bg: Option<ColorValue>,
    pub item_border: Option<ColorValue>,
    pub item_fg: Option<ColorValue>,
    pub item_active_bg: Option<ColorValue>,
    pub item_active_fg: Option<ColorValue>,
    pub item_hover_bg: Option<ColorValue>,
    pub item_disabled_fg: Option<ColorValue>,
    pub dots_fg: Option<ColorValue>,
}

impl PaginationPatch {
    fn apply(&self, mut current: PaginationTokens) -> PaginationTokens {
        if let Some(value) = &self.item_bg {
            current.item_bg = value.clone();
        }
        if let Some(value) = &self.item_border {
            current.item_border = value.clone();
        }
        if let Some(value) = &self.item_fg {
            current.item_fg = value.clone();
        }
        if let Some(value) = &self.item_active_bg {
            current.item_active_bg = value.clone();
        }
        if let Some(value) = &self.item_active_fg {
            current.item_active_fg = value.clone();
        }
        if let Some(value) = &self.item_hover_bg {
            current.item_hover_bg = value.clone();
        }
        if let Some(value) = &self.item_disabled_fg {
            current.item_disabled_fg = value.clone();
        }
        if let Some(value) = &self.dots_fg {
            current.dots_fg = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BreadcrumbsPatch {
    pub item_fg: Option<ColorValue>,
    pub item_current_fg: Option<ColorValue>,
    pub separator: Option<ColorValue>,
    pub item_hover_bg: Option<ColorValue>,
}

impl BreadcrumbsPatch {
    fn apply(&self, mut current: BreadcrumbsTokens) -> BreadcrumbsTokens {
        if let Some(value) = &self.item_fg {
            current.item_fg = value.clone();
        }
        if let Some(value) = &self.item_current_fg {
            current.item_current_fg = value.clone();
        }
        if let Some(value) = &self.separator {
            current.separator = value.clone();
        }
        if let Some(value) = &self.item_hover_bg {
            current.item_hover_bg = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TablePatch {
    pub header_bg: Option<ColorValue>,
    pub header_fg: Option<ColorValue>,
    pub row_bg: Option<ColorValue>,
    pub row_alt_bg: Option<ColorValue>,
    pub row_hover_bg: Option<ColorValue>,
    pub row_border: Option<ColorValue>,
    pub cell_fg: Option<ColorValue>,
    pub caption: Option<ColorValue>,
}

impl TablePatch {
    fn apply(&self, mut current: TableTokens) -> TableTokens {
        if let Some(value) = &self.header_bg {
            current.header_bg = value.clone();
        }
        if let Some(value) = &self.header_fg {
            current.header_fg = value.clone();
        }
        if let Some(value) = &self.row_bg {
            current.row_bg = value.clone();
        }
        if let Some(value) = &self.row_alt_bg {
            current.row_alt_bg = value.clone();
        }
        if let Some(value) = &self.row_hover_bg {
            current.row_hover_bg = value.clone();
        }
        if let Some(value) = &self.row_border {
            current.row_border = value.clone();
        }
        if let Some(value) = &self.cell_fg {
            current.cell_fg = value.clone();
        }
        if let Some(value) = &self.caption {
            current.caption = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StepperPatch {
    pub step_bg: Option<ColorValue>,
    pub step_border: Option<ColorValue>,
    pub step_fg: Option<ColorValue>,
    pub step_active_bg: Option<ColorValue>,
    pub step_active_border: Option<ColorValue>,
    pub step_active_fg: Option<ColorValue>,
    pub step_completed_bg: Option<ColorValue>,
    pub step_completed_border: Option<ColorValue>,
    pub step_completed_fg: Option<ColorValue>,
    pub connector: Option<ColorValue>,
    pub label: Option<ColorValue>,
    pub description: Option<ColorValue>,
    pub panel_bg: Option<ColorValue>,
    pub panel_border: Option<ColorValue>,
    pub panel_fg: Option<ColorValue>,
}

impl StepperPatch {
    fn apply(&self, mut current: StepperTokens) -> StepperTokens {
        if let Some(value) = &self.step_bg {
            current.step_bg = value.clone();
        }
        if let Some(value) = &self.step_border {
            current.step_border = value.clone();
        }
        if let Some(value) = &self.step_fg {
            current.step_fg = value.clone();
        }
        if let Some(value) = &self.step_active_bg {
            current.step_active_bg = value.clone();
        }
        if let Some(value) = &self.step_active_border {
            current.step_active_border = value.clone();
        }
        if let Some(value) = &self.step_active_fg {
            current.step_active_fg = value.clone();
        }
        if let Some(value) = &self.step_completed_bg {
            current.step_completed_bg = value.clone();
        }
        if let Some(value) = &self.step_completed_border {
            current.step_completed_border = value.clone();
        }
        if let Some(value) = &self.step_completed_fg {
            current.step_completed_fg = value.clone();
        }
        if let Some(value) = &self.connector {
            current.connector = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        if let Some(value) = &self.description {
            current.description = value.clone();
        }
        if let Some(value) = &self.panel_bg {
            current.panel_bg = value.clone();
        }
        if let Some(value) = &self.panel_border {
            current.panel_border = value.clone();
        }
        if let Some(value) = &self.panel_fg {
            current.panel_fg = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TimelinePatch {
    pub bullet_bg: Option<ColorValue>,
    pub bullet_border: Option<ColorValue>,
    pub bullet_fg: Option<ColorValue>,
    pub bullet_active_bg: Option<ColorValue>,
    pub bullet_active_border: Option<ColorValue>,
    pub bullet_active_fg: Option<ColorValue>,
    pub line: Option<ColorValue>,
    pub line_active: Option<ColorValue>,
    pub title: Option<ColorValue>,
    pub title_active: Option<ColorValue>,
    pub body: Option<ColorValue>,
    pub card_bg: Option<ColorValue>,
    pub card_border: Option<ColorValue>,
}

impl TimelinePatch {
    fn apply(&self, mut current: TimelineTokens) -> TimelineTokens {
        if let Some(value) = &self.bullet_bg {
            current.bullet_bg = value.clone();
        }
        if let Some(value) = &self.bullet_border {
            current.bullet_border = value.clone();
        }
        if let Some(value) = &self.bullet_fg {
            current.bullet_fg = value.clone();
        }
        if let Some(value) = &self.bullet_active_bg {
            current.bullet_active_bg = value.clone();
        }
        if let Some(value) = &self.bullet_active_border {
            current.bullet_active_border = value.clone();
        }
        if let Some(value) = &self.bullet_active_fg {
            current.bullet_active_fg = value.clone();
        }
        if let Some(value) = &self.line {
            current.line = value.clone();
        }
        if let Some(value) = &self.line_active {
            current.line_active = value.clone();
        }
        if let Some(value) = &self.title {
            current.title = value.clone();
        }
        if let Some(value) = &self.title_active {
            current.title_active = value.clone();
        }
        if let Some(value) = &self.body {
            current.body = value.clone();
        }
        if let Some(value) = &self.card_bg {
            current.card_bg = value.clone();
        }
        if let Some(value) = &self.card_border {
            current.card_border = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TreePatch {
    pub row_fg: Option<ColorValue>,
    pub row_selected_fg: Option<ColorValue>,
    pub row_selected_bg: Option<ColorValue>,
    pub row_hover_bg: Option<ColorValue>,
    pub row_disabled_fg: Option<ColorValue>,
    pub line: Option<ColorValue>,
}

impl TreePatch {
    fn apply(&self, mut current: TreeTokens) -> TreeTokens {
        if let Some(value) = &self.row_fg {
            current.row_fg = value.clone();
        }
        if let Some(value) = &self.row_selected_fg {
            current.row_selected_fg = value.clone();
        }
        if let Some(value) = &self.row_selected_bg {
            current.row_selected_bg = value.clone();
        }
        if let Some(value) = &self.row_hover_bg {
            current.row_hover_bg = value.clone();
        }
        if let Some(value) = &self.row_disabled_fg {
            current.row_disabled_fg = value.clone();
        }
        if let Some(value) = &self.line {
            current.line = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ComponentPatch {
    pub button: ButtonPatch,
    pub input: InputPatch,
    pub radio: RadioPatch,
    pub checkbox: CheckboxPatch,
    pub switch: SwitchPatch,
    pub chip: ChipPatch,
    pub badge: BadgePatch,
    pub accordion: AccordionPatch,
    pub menu: MenuPatch,
    pub progress: ProgressPatch,
    pub slider: SliderPatch,
    pub overlay: OverlayPatch,
    pub loading_overlay: LoadingOverlayPatch,
    pub popover: PopoverPatch,
    pub tooltip: TooltipPatch,
    pub hover_card: HoverCardPatch,
    pub select: SelectPatch,
    pub modal: ModalPatch,
    pub toast: ToastPatch,
    pub divider: DividerPatch,
    pub scroll_area: ScrollAreaPatch,
    pub drawer: DrawerPatch,
    pub app_shell: AppShellPatch,
    pub title_bar: TitleBarPatch,
    pub sidebar: SidebarPatch,
    pub text: TextPatch,
    pub title: TitlePatch,
    pub paper: PaperPatch,
    pub action_icon: ActionIconPatch,
    pub segmented_control: SegmentedControlPatch,
    pub textarea: TextareaPatch,
    pub number_input: NumberInputPatch,
    pub range_slider: RangeSliderPatch,
    pub rating: RatingPatch,
    pub tabs: TabsPatch,
    pub pagination: PaginationPatch,
    pub breadcrumbs: BreadcrumbsPatch,
    pub table: TablePatch,
    pub stepper: StepperPatch,
    pub timeline: TimelinePatch,
    pub tree: TreePatch,
}

impl ComponentPatch {
    fn apply(&self, current: ComponentTokens) -> ComponentTokens {
        ComponentTokens {
            button: self.button.apply(current.button),
            input: self.input.apply(current.input),
            radio: self.radio.apply(current.radio),
            checkbox: self.checkbox.apply(current.checkbox),
            switch: self.switch.apply(current.switch),
            chip: self.chip.apply(current.chip),
            badge: self.badge.apply(current.badge),
            accordion: self.accordion.apply(current.accordion),
            menu: self.menu.apply(current.menu),
            progress: self.progress.apply(current.progress),
            slider: self.slider.apply(current.slider),
            overlay: self.overlay.apply(current.overlay),
            loading_overlay: self.loading_overlay.apply(current.loading_overlay),
            popover: self.popover.apply(current.popover),
            tooltip: self.tooltip.apply(current.tooltip),
            hover_card: self.hover_card.apply(current.hover_card),
            select: self.select.apply(current.select),
            modal: self.modal.apply(current.modal),
            toast: self.toast.apply(current.toast),
            divider: self.divider.apply(current.divider),
            scroll_area: self.scroll_area.apply(current.scroll_area),
            drawer: self.drawer.apply(current.drawer),
            app_shell: self.app_shell.apply(current.app_shell),
            title_bar: self.title_bar.apply(current.title_bar),
            sidebar: self.sidebar.apply(current.sidebar),
            text: self.text.apply(current.text),
            title: self.title.apply(current.title),
            paper: self.paper.apply(current.paper),
            action_icon: self.action_icon.apply(current.action_icon),
            segmented_control: self.segmented_control.apply(current.segmented_control),
            textarea: self.textarea.apply(current.textarea),
            number_input: self.number_input.apply(current.number_input),
            range_slider: self.range_slider.apply(current.range_slider),
            rating: self.rating.apply(current.rating),
            tabs: self.tabs.apply(current.tabs),
            pagination: self.pagination.apply(current.pagination),
            breadcrumbs: self.breadcrumbs.apply(current.breadcrumbs),
            table: self.table.apply(current.table),
            stepper: self.stepper.apply(current.stepper),
            timeline: self.timeline.apply(current.timeline),
            tree: self.tree.apply(current.tree),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ThemePatch {
    pub primary_color: Option<PaletteKey>,
    pub primary_shade: Option<PrimaryShade>,
    pub color_scheme: Option<ColorScheme>,
    pub palette_overrides: BTreeMap<PaletteKey, ColorScale>,
    pub semantic: SemanticPatch,
    pub components: ComponentPatch,
}

#[derive(Clone, Default)]
pub struct LocalTheme {
    resolved: Option<Arc<Theme>>,
    component_patch: Option<ComponentPatch>,
}

impl LocalTheme {
    pub fn with_component_patch(mut self, patch: ComponentPatch) -> Self {
        self.component_patch = Some(patch);
        self
    }

    pub fn set_component_patch(&mut self, patch: Option<ComponentPatch>) {
        self.component_patch = patch;
        self.resolved = None;
    }

    pub fn sync_from_provider(&mut self, cx: &gpui::App) {
        let base = crate::provider::CalmProvider::theme_arc_or_default(cx);
        if let Some(component_patch) = &self.component_patch {
            let mut merged = base.as_ref().clone();
            merged.components = component_patch.apply(merged.components);
            self.resolved = Some(Arc::new(merged));
        } else {
            self.resolved = Some(base);
        }
    }

    fn fallback_theme() -> &'static Theme {
        static FALLBACK: OnceLock<Theme> = OnceLock::new();
        FALLBACK.get_or_init(Theme::default)
    }
}

impl std::ops::Deref for LocalTheme {
    type Target = Theme;

    fn deref(&self) -> &Self::Target {
        if let Some(resolved) = self.resolved.as_deref() {
            resolved
        } else {
            Self::fallback_theme()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::COLOR_STOPS;

    #[test]
    fn default_theme_uses_blue_as_primary_color() {
        let theme = Theme::default();
        assert_eq!(theme.primary_color, PaletteKey::Blue);
        assert_eq!(
            theme.primary_shade,
            PrimaryShade::Split { light: 6, dark: 8 }
        );
    }

    #[test]
    fn default_palette_is_complete() {
        let theme = Theme::default();
        assert_eq!(theme.palette.len(), 14);
        assert_eq!(theme.palette[&PaletteKey::Blue].len(), COLOR_STOPS);
    }

    #[test]
    fn nested_theme_patch_overrides_only_target_fields() {
        let base = Theme::default();
        let patch = ThemePatch {
            semantic: SemanticPatch {
                text_primary: Some(ColorValue::palette(PaletteKey::Orange, 8)),
                ..SemanticPatch::default()
            },
            ..ThemePatch::default()
        };
        let next = base.merged(&patch);
        assert_eq!(
            next.resolve_color(&next.semantic.text_primary),
            base.palette[&PaletteKey::Orange][8]
        );
        assert_eq!(
            next.resolve_color(&next.semantic.text_secondary),
            base.resolve_color(&base.semantic.text_secondary)
        );
    }

    #[test]
    fn color_scheme_switch_recomputes_semantic_and_component_tokens() {
        let light = Theme::default().with_color_scheme(ColorScheme::Light);
        let dark = Theme::default().with_color_scheme(ColorScheme::Dark);

        assert_ne!(light.semantic.bg_canvas, dark.semantic.bg_canvas);
        assert_ne!(
            light.components.checkbox.label,
            dark.components.checkbox.label
        );
        assert_ne!(
            light.components.radio.description,
            dark.components.radio.description
        );
        assert_ne!(light.components.switch.label, dark.components.switch.label);
    }

    #[test]
    fn dark_theme_uses_resolved_dark_text_for_selection_controls() {
        let dark = Theme::default().with_color_scheme(ColorScheme::Dark);
        let light = Theme::default().with_color_scheme(ColorScheme::Light);

        let dark_checkbox = dark.resolve_color(&dark.components.checkbox.label);
        let light_checkbox = light.resolve_color(&light.components.checkbox.label);
        let dark_radio = dark.resolve_color(&dark.components.radio.label);
        let light_radio = light.resolve_color(&light.components.radio.label);
        let dark_switch = dark.resolve_color(&dark.components.switch.label);
        let light_switch = light.resolve_color(&light.components.switch.label);

        assert_ne!(dark_checkbox, light_checkbox);
        assert_ne!(dark_radio, light_radio);
        assert_ne!(dark_switch, light_switch);
    }
}
