use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use crate::style::Radius;
use crate::tokens::{ColorScale, PaletteCatalog, PaletteKey};
use gpui::{
    Background, Corners, Fill, FontWeight, Hsla, Pixels, Rgba, black, px, transparent_black, white,
};

mod overrides_api;
mod themable_impls;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorScheme {
    Light,
    Dark,
}

pub const PRIMARY_SHADE_LIGHT_DEFAULT: u8 = 6;
pub const PRIMARY_SHADE_DARK_DEFAULT: u8 = 8;
pub const BUILTIN_TRANSPARENT_HEX: &str = "#00000000";
pub const BUILTIN_BLACK_HEX: &str = "#000000";
pub const BUILTIN_WHITE_HEX: &str = "#FFFFFF";
pub const COLOR_TOKEN_TRANSPARENT: ColorToken = ColorToken::Hex(BUILTIN_TRANSPARENT_HEX);
pub const COLOR_TOKEN_BLACK: ColorToken = ColorToken::Hex(BUILTIN_BLACK_HEX);
pub const COLOR_TOKEN_WHITE: ColorToken = ColorToken::Hex(BUILTIN_WHITE_HEX);

fn resolve_palette_hsla(key: PaletteKey, shade: u8) -> Hsla {
    Rgba::try_from(PaletteCatalog::scale(key)[shade.min(9) as usize])
        .map(Into::into)
        .unwrap_or_else(|_| black())
}

fn resolve_hex_hsla(hex: &'static str) -> Hsla {
    Rgba::try_from(hex)
        .map(Into::into)
        .unwrap_or_else(|_| black())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticColorToken {
    TextPrimary,
    TextSecondary,
    TextMuted,
    BgCanvas,
    BgSurface,
    BgSoft,
    BorderSubtle,
    BorderStrong,
    FocusRing,
    StatusInfo,
    StatusSuccess,
    StatusWarning,
    StatusError,
    OverlayMask,
}

pub trait ResolveWithTheme<T> {
    fn resolve(self, theme: &Theme) -> T;
}

impl ResolveWithTheme<Hsla> for Hsla {
    fn resolve(self, _theme: &Theme) -> Hsla {
        self
    }
}

impl ResolveWithTheme<Hsla> for &Hsla {
    fn resolve(self, _theme: &Theme) -> Hsla {
        *self
    }
}

impl ResolveWithTheme<Hsla> for SemanticColorToken {
    fn resolve(self, theme: &Theme) -> Hsla {
        match self {
            SemanticColorToken::TextPrimary => theme.semantic.text_primary,
            SemanticColorToken::TextSecondary => theme.semantic.text_secondary,
            SemanticColorToken::TextMuted => theme.semantic.text_muted,
            SemanticColorToken::BgCanvas => theme.semantic.bg_canvas,
            SemanticColorToken::BgSurface => theme.semantic.bg_surface,
            SemanticColorToken::BgSoft => theme.semantic.bg_soft,
            SemanticColorToken::BorderSubtle => theme.semantic.border_subtle,
            SemanticColorToken::BorderStrong => theme.semantic.border_strong,
            SemanticColorToken::FocusRing => theme.semantic.focus_ring,
            SemanticColorToken::StatusInfo => theme.semantic.status_info,
            SemanticColorToken::StatusSuccess => theme.semantic.status_success,
            SemanticColorToken::StatusWarning => theme.semantic.status_warning,
            SemanticColorToken::StatusError => theme.semantic.status_error,
            SemanticColorToken::OverlayMask => theme.semantic.overlay_mask,
        }
    }
}

impl ResolveWithTheme<Background> for SemanticColorToken {
    fn resolve(self, theme: &Theme) -> Background {
        ResolveWithTheme::<Hsla>::resolve(self, theme).into()
    }
}

impl ResolveWithTheme<Fill> for SemanticColorToken {
    fn resolve(self, theme: &Theme) -> Fill {
        ResolveWithTheme::<Hsla>::resolve(self, theme).into()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorToken {
    Raw(Hsla),
    Hex(&'static str),
    Palette { key: PaletteKey, shade: u8 },
    Semantic(SemanticColorToken),
}

impl ColorToken {
    pub const fn builtin_transparent() -> Self {
        Self::Hex(BUILTIN_TRANSPARENT_HEX)
    }

    pub const fn builtin_black() -> Self {
        Self::Hex(BUILTIN_BLACK_HEX)
    }

    pub const fn builtin_white() -> Self {
        Self::Hex(BUILTIN_WHITE_HEX)
    }

    pub const fn palette(key: PaletteKey, shade: u8) -> Self {
        Self::Palette { key, shade }
    }

    pub fn resolve(self, theme: &Theme) -> Hsla {
        match self {
            ColorToken::Raw(value) => value,
            ColorToken::Hex(hex) => resolve_hex_hsla(hex),
            ColorToken::Palette { key, shade } => resolve_palette_hsla(key, shade),
            ColorToken::Semantic(value) => value.resolve(theme),
        }
    }
}

impl ResolveWithTheme<Hsla> for ColorToken {
    fn resolve(self, theme: &Theme) -> Hsla {
        self.resolve(theme)
    }
}

impl ResolveWithTheme<Hsla> for &ColorToken {
    fn resolve(self, theme: &Theme) -> Hsla {
        (*self).resolve(theme)
    }
}

impl ResolveWithTheme<Background> for ColorToken {
    fn resolve(self, theme: &Theme) -> Background {
        ResolveWithTheme::<Hsla>::resolve(self, theme).into()
    }
}

impl ResolveWithTheme<Background> for &ColorToken {
    fn resolve(self, theme: &Theme) -> Background {
        ResolveWithTheme::<Hsla>::resolve(*self, theme).into()
    }
}

impl ResolveWithTheme<Fill> for ColorToken {
    fn resolve(self, theme: &Theme) -> Fill {
        ResolveWithTheme::<Hsla>::resolve(self, theme).into()
    }
}

impl ResolveWithTheme<Fill> for &ColorToken {
    fn resolve(self, theme: &Theme) -> Fill {
        ResolveWithTheme::<Hsla>::resolve(*self, theme).into()
    }
}

impl From<Hsla> for ColorToken {
    fn from(value: Hsla) -> Self {
        Self::Raw(value)
    }
}

impl From<&Hsla> for ColorToken {
    fn from(value: &Hsla) -> Self {
        Self::Raw(*value)
    }
}

impl From<SemanticColorToken> for ColorToken {
    fn from(value: SemanticColorToken) -> Self {
        Self::Semantic(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuiltinRadius {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Pill,
}

impl BuiltinRadius {
    pub const fn pixels(self) -> Pixels {
        match self {
            BuiltinRadius::Xs => px(2.0),
            BuiltinRadius::Sm => px(4.0),
            BuiltinRadius::Md => px(8.0),
            BuiltinRadius::Lg => px(16.0),
            BuiltinRadius::Xl => px(24.0),
            BuiltinRadius::Pill => px(999.0),
        }
    }
}

impl From<BuiltinRadius> for Pixels {
    fn from(value: BuiltinRadius) -> Self {
        value.pixels()
    }
}

impl From<BuiltinRadius> for Corners<Pixels> {
    fn from(value: BuiltinRadius) -> Self {
        Corners::all(value.pixels())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticRadiusToken {
    Default,
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Pill,
}

impl From<Radius> for SemanticRadiusToken {
    fn from(value: Radius) -> Self {
        match value {
            Radius::Xs => SemanticRadiusToken::Xs,
            Radius::Sm => SemanticRadiusToken::Sm,
            Radius::Md => SemanticRadiusToken::Md,
            Radius::Lg => SemanticRadiusToken::Lg,
            Radius::Xl => SemanticRadiusToken::Xl,
            Radius::Pill => SemanticRadiusToken::Pill,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RadiusToken {
    Raw(Pixels),
    Builtin(BuiltinRadius),
    Semantic(SemanticRadiusToken),
}

impl From<Pixels> for RadiusToken {
    fn from(value: Pixels) -> Self {
        Self::Raw(value)
    }
}

impl From<BuiltinRadius> for RadiusToken {
    fn from(value: BuiltinRadius) -> Self {
        Self::Builtin(value)
    }
}

impl From<SemanticRadiusToken> for RadiusToken {
    fn from(value: SemanticRadiusToken) -> Self {
        Self::Semantic(value)
    }
}

impl ResolveWithTheme<Pixels> for SemanticRadiusToken {
    fn resolve(self, theme: &Theme) -> Pixels {
        match self {
            SemanticRadiusToken::Default => theme.radii.default,
            SemanticRadiusToken::Xs => theme.radii.xs,
            SemanticRadiusToken::Sm => theme.radii.sm,
            SemanticRadiusToken::Md => theme.radii.md,
            SemanticRadiusToken::Lg => theme.radii.lg,
            SemanticRadiusToken::Xl => theme.radii.xl,
            SemanticRadiusToken::Pill => theme.radii.pill,
        }
    }
}

impl ResolveWithTheme<Pixels> for RadiusToken {
    fn resolve(self, theme: &Theme) -> Pixels {
        match self {
            RadiusToken::Raw(value) => value,
            RadiusToken::Builtin(value) => value.into(),
            RadiusToken::Semantic(value) => value.resolve(theme),
        }
    }
}

impl ResolveWithTheme<Pixels> for &RadiusToken {
    fn resolve(self, theme: &Theme) -> Pixels {
        (*self).resolve(theme)
    }
}

impl ResolveWithTheme<Corners<Pixels>> for RadiusToken {
    fn resolve(self, theme: &Theme) -> Corners<Pixels> {
        Corners::all(ResolveWithTheme::<Pixels>::resolve(self, theme))
    }
}

impl ResolveWithTheme<Corners<Pixels>> for &RadiusToken {
    fn resolve(self, theme: &Theme) -> Corners<Pixels> {
        Corners::all(ResolveWithTheme::<Pixels>::resolve(*self, theme))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThemeRadii {
    pub default: Pixels,
    pub xs: Pixels,
    pub sm: Pixels,
    pub md: Pixels,
    pub lg: Pixels,
    pub xl: Pixels,
    pub pill: Pixels,
}

impl Default for ThemeRadii {
    fn default() -> Self {
        Self {
            default: px(4.0),
            xs: px(2.0),
            sm: px(4.0),
            md: px(8.0),
            lg: px(16.0),
            xl: px(24.0),
            pill: px(999.0),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticColors {
    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub text_muted: Hsla,
    pub bg_canvas: Hsla,
    pub bg_surface: Hsla,
    pub bg_soft: Hsla,
    pub border_subtle: Hsla,
    pub border_strong: Hsla,
    pub focus_ring: Hsla,
    pub status_info: Hsla,
    pub status_success: Hsla,
    pub status_warning: Hsla,
    pub status_error: Hsla,
    pub overlay_mask: Hsla,
}

impl SemanticColors {
    pub fn defaults(primary: PaletteKey) -> Self {
        Self::defaults_for(primary, ColorScheme::Light)
    }

    pub fn defaults_for(primary: PaletteKey, scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Light => Self {
                text_primary: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                text_secondary: (Rgba::try_from(
                    PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                )
                .map(Into::into)
                .unwrap_or_else(|_| black())),
                text_muted: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[6 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                bg_canvas: white(),
                bg_surface: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                bg_soft: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[1 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                border_subtle: (Rgba::try_from(
                    PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                )
                .map(Into::into)
                .unwrap_or_else(|_| black())),
                border_strong: (Rgba::try_from(
                    PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                )
                .map(Into::into)
                .unwrap_or_else(|_| black())),
                focus_ring: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                status_info: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Blue)[6 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                status_success: (Rgba::try_from(
                    PaletteCatalog::scale(PaletteKey::Green)[6 as usize],
                )
                .map(Into::into)
                .unwrap_or_else(|_| black())),
                status_warning: (Rgba::try_from(
                    PaletteCatalog::scale(PaletteKey::Yellow)[7 as usize],
                )
                .map(Into::into)
                .unwrap_or_else(|_| black())),
                status_error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[6 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                overlay_mask: (Rgba::try_from("#00000073")
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
            },
            ColorScheme::Dark => Self {
                text_primary: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                text_secondary: (Rgba::try_from(
                    PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                )
                .map(Into::into)
                .unwrap_or_else(|_| black())),
                text_muted: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[5 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                bg_canvas: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                bg_surface: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                bg_soft: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[7 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                border_subtle: (Rgba::try_from(
                    PaletteCatalog::scale(PaletteKey::Dark)[5 as usize],
                )
                .map(Into::into)
                .unwrap_or_else(|_| black())),
                border_strong: (Rgba::try_from(
                    PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                )
                .map(Into::into)
                .unwrap_or_else(|_| black())),
                focus_ring: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                status_info: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Blue)[4 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                status_success: (Rgba::try_from(
                    PaletteCatalog::scale(PaletteKey::Green)[4 as usize],
                )
                .map(Into::into)
                .unwrap_or_else(|_| black())),
                status_warning: (Rgba::try_from(
                    PaletteCatalog::scale(PaletteKey::Yellow)[4 as usize],
                )
                .map(Into::into)
                .unwrap_or_else(|_| black())),
                status_error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[4 as usize])
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                overlay_mask: (Rgba::try_from("#000000CC")
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ButtonTokens {
    pub filled_bg: Hsla,
    pub filled_fg: Hsla,
    pub light_bg: Hsla,
    pub light_fg: Hsla,
    pub subtle_bg: Hsla,
    pub subtle_fg: Hsla,
    pub outline_border: Hsla,
    pub outline_fg: Hsla,
    pub ghost_fg: Hsla,
    pub disabled_bg: Hsla,
    pub disabled_fg: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputTokens {
    pub bg: Hsla,
    pub fg: Hsla,
    pub placeholder: Hsla,
    pub border: Hsla,
    pub border_focus: Hsla,
    pub border_error: Hsla,
    pub label: Hsla,
    pub description: Hsla,
    pub error: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RadioTokens {
    pub control_bg: Hsla,
    pub border: Hsla,
    pub border_checked: Hsla,
    pub indicator: Hsla,
    pub label: Hsla,
    pub description: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckboxTokens {
    pub control_bg: Hsla,
    pub control_bg_checked: Hsla,
    pub border: Hsla,
    pub border_checked: Hsla,
    pub indicator: Hsla,
    pub label: Hsla,
    pub description: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwitchTokens {
    pub track_off_bg: Hsla,
    pub track_on_bg: Hsla,
    pub thumb_bg: Hsla,
    pub label: Hsla,
    pub description: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChipTokens {
    pub unchecked_bg: Hsla,
    pub unchecked_fg: Hsla,
    pub unchecked_border: Hsla,
    pub filled_bg: Hsla,
    pub filled_fg: Hsla,
    pub light_bg: Hsla,
    pub light_fg: Hsla,
    pub subtle_bg: Hsla,
    pub subtle_fg: Hsla,
    pub outline_border: Hsla,
    pub outline_fg: Hsla,
    pub ghost_fg: Hsla,
    pub default_bg: Hsla,
    pub default_fg: Hsla,
    pub default_border: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BadgeTokens {
    pub filled_bg: Hsla,
    pub filled_fg: Hsla,
    pub light_bg: Hsla,
    pub light_fg: Hsla,
    pub subtle_bg: Hsla,
    pub subtle_fg: Hsla,
    pub outline_border: Hsla,
    pub outline_fg: Hsla,
    pub default_bg: Hsla,
    pub default_fg: Hsla,
    pub default_border: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccordionTokens {
    pub item_bg: Hsla,
    pub item_border: Hsla,
    pub label: Hsla,
    pub description: Hsla,
    pub content: Hsla,
    pub chevron: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuTokens {
    pub dropdown_bg: Hsla,
    pub dropdown_border: Hsla,
    pub item_fg: Hsla,
    pub item_hover_bg: Hsla,
    pub item_disabled_fg: Hsla,
    pub icon: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgressTokens {
    pub track_bg: Hsla,
    pub fill_bg: Hsla,
    pub label: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliderTokens {
    pub track_bg: Hsla,
    pub fill_bg: Hsla,
    pub thumb_bg: Hsla,
    pub thumb_border: Hsla,
    pub label: Hsla,
    pub value: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverlayTokens {
    pub bg: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadingOverlayTokens {
    pub bg: Hsla,
    pub loader_color: Hsla,
    pub label: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PopoverTokens {
    pub bg: Hsla,
    pub border: Hsla,
    pub title: Hsla,
    pub body: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TooltipTokens {
    pub bg: Hsla,
    pub fg: Hsla,
    pub border: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HoverCardTokens {
    pub bg: Hsla,
    pub border: Hsla,
    pub title: Hsla,
    pub body: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectTokens {
    pub bg: Hsla,
    pub fg: Hsla,
    pub placeholder: Hsla,
    pub border: Hsla,
    pub border_focus: Hsla,
    pub border_error: Hsla,
    pub dropdown_bg: Hsla,
    pub dropdown_border: Hsla,
    pub option_fg: Hsla,
    pub option_hover_bg: Hsla,
    pub option_selected_bg: Hsla,
    pub tag_bg: Hsla,
    pub tag_fg: Hsla,
    pub tag_border: Hsla,
    pub icon: Hsla,
    pub label: Hsla,
    pub description: Hsla,
    pub error: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModalTokens {
    pub panel_bg: Hsla,
    pub panel_border: Hsla,
    pub overlay_bg: Hsla,
    pub title: Hsla,
    pub body: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToastTokens {
    pub info_bg: Hsla,
    pub info_fg: Hsla,
    pub success_bg: Hsla,
    pub success_fg: Hsla,
    pub warning_bg: Hsla,
    pub warning_fg: Hsla,
    pub error_bg: Hsla,
    pub error_fg: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DividerTokens {
    pub line: Hsla,
    pub label: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScrollAreaTokens {
    pub bg: Hsla,
    pub border: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrawerTokens {
    pub panel_bg: Hsla,
    pub panel_border: Hsla,
    pub overlay_bg: Hsla,
    pub title: Hsla,
    pub body: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppShellTokens {
    pub bg: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TitleBarTokens {
    pub bg: Hsla,
    pub border: Hsla,
    pub fg: Hsla,
    pub controls_bg: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarTokens {
    pub bg: Hsla,
    pub border: Hsla,
    pub header_fg: Hsla,
    pub content_fg: Hsla,
    pub footer_fg: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextTokens {
    pub fg: Hsla,
    pub secondary: Hsla,
    pub muted: Hsla,
    pub accent: Hsla,
    pub success: Hsla,
    pub warning: Hsla,
    pub error: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TitleTokens {
    pub fg: Hsla,
    pub subtitle: Hsla,
    pub gap: Pixels,
    pub subtitle_size: Pixels,
    pub subtitle_line_height: Pixels,
    pub subtitle_weight: FontWeight,
    pub h1: TitleLevelTokens,
    pub h2: TitleLevelTokens,
    pub h3: TitleLevelTokens,
    pub h4: TitleLevelTokens,
    pub h5: TitleLevelTokens,
    pub h6: TitleLevelTokens,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TitleLevelTokens {
    pub font_size: Pixels,
    pub line_height: Pixels,
    pub weight: FontWeight,
}

impl TitleTokens {
    pub fn level(&self, order: u8) -> TitleLevelTokens {
        match order.clamp(1, 6) {
            1 => self.h1,
            2 => self.h2,
            3 => self.h3,
            4 => self.h4,
            5 => self.h5,
            _ => self.h6,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaperTokens {
    pub bg: Hsla,
    pub border: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionIconTokens {
    pub filled_bg: Hsla,
    pub filled_fg: Hsla,
    pub light_bg: Hsla,
    pub light_fg: Hsla,
    pub subtle_bg: Hsla,
    pub subtle_fg: Hsla,
    pub outline_border: Hsla,
    pub outline_fg: Hsla,
    pub ghost_fg: Hsla,
    pub default_bg: Hsla,
    pub default_fg: Hsla,
    pub default_border: Hsla,
    pub disabled_bg: Hsla,
    pub disabled_fg: Hsla,
    pub disabled_border: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SegmentedControlTokens {
    pub bg: Hsla,
    pub border: Hsla,
    pub item_fg: Hsla,
    pub item_active_bg: Hsla,
    pub item_active_fg: Hsla,
    pub item_hover_bg: Hsla,
    pub item_disabled_fg: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextareaTokens {
    pub bg: Hsla,
    pub fg: Hsla,
    pub placeholder: Hsla,
    pub border: Hsla,
    pub border_focus: Hsla,
    pub border_error: Hsla,
    pub label: Hsla,
    pub description: Hsla,
    pub error: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NumberInputTokens {
    pub bg: Hsla,
    pub fg: Hsla,
    pub placeholder: Hsla,
    pub border: Hsla,
    pub border_focus: Hsla,
    pub border_error: Hsla,
    pub controls_bg: Hsla,
    pub controls_fg: Hsla,
    pub controls_border: Hsla,
    pub label: Hsla,
    pub description: Hsla,
    pub error: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RangeSliderTokens {
    pub track_bg: Hsla,
    pub range_bg: Hsla,
    pub thumb_bg: Hsla,
    pub thumb_border: Hsla,
    pub label: Hsla,
    pub value: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RatingTokens {
    pub active: Hsla,
    pub inactive: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TabsTokens {
    pub list_bg: Hsla,
    pub list_border: Hsla,
    pub tab_fg: Hsla,
    pub tab_active_bg: Hsla,
    pub tab_active_fg: Hsla,
    pub tab_hover_bg: Hsla,
    pub tab_disabled_fg: Hsla,
    pub panel_bg: Hsla,
    pub panel_border: Hsla,
    pub panel_fg: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaginationTokens {
    pub item_bg: Hsla,
    pub item_border: Hsla,
    pub item_fg: Hsla,
    pub item_active_bg: Hsla,
    pub item_active_fg: Hsla,
    pub item_hover_bg: Hsla,
    pub item_disabled_fg: Hsla,
    pub dots_fg: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreadcrumbsTokens {
    pub item_fg: Hsla,
    pub item_current_fg: Hsla,
    pub separator: Hsla,
    pub item_hover_bg: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableTokens {
    pub header_bg: Hsla,
    pub header_fg: Hsla,
    pub row_bg: Hsla,
    pub row_alt_bg: Hsla,
    pub row_hover_bg: Hsla,
    pub row_border: Hsla,
    pub cell_fg: Hsla,
    pub caption: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepperTokens {
    pub step_bg: Hsla,
    pub step_border: Hsla,
    pub step_fg: Hsla,
    pub step_active_bg: Hsla,
    pub step_active_border: Hsla,
    pub step_active_fg: Hsla,
    pub step_completed_bg: Hsla,
    pub step_completed_border: Hsla,
    pub step_completed_fg: Hsla,
    pub connector: Hsla,
    pub label: Hsla,
    pub description: Hsla,
    pub panel_bg: Hsla,
    pub panel_border: Hsla,
    pub panel_fg: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimelineTokens {
    pub bullet_bg: Hsla,
    pub bullet_border: Hsla,
    pub bullet_fg: Hsla,
    pub bullet_active_bg: Hsla,
    pub bullet_active_border: Hsla,
    pub bullet_active_fg: Hsla,
    pub line: Hsla,
    pub line_active: Hsla,
    pub title: Hsla,
    pub title_active: Hsla,
    pub body: Hsla,
    pub card_bg: Hsla,
    pub card_border: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeTokens {
    pub row_fg: Hsla,
    pub row_selected_fg: Hsla,
    pub row_selected_bg: Hsla,
    pub row_hover_bg: Hsla,
    pub row_disabled_fg: Hsla,
    pub line: Hsla,
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
                    filled_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    filled_fg: white(),
                    light_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    light_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    subtle_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    subtle_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    outline_border: (Rgba::try_from(PaletteCatalog::scale(primary)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    outline_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    ghost_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    disabled_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[2 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                input: InputTokens {
                    bg: white(),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    placeholder: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_error: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Red)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                radio: RadioTokens {
                    control_bg: white(),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_checked: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    indicator: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                checkbox: CheckboxTokens {
                    control_bg: white(),
                    control_bg_checked: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_checked: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    indicator: white(),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                switch: SwitchTokens {
                    track_off_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    track_on_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    thumb_bg: white(),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                chip: ChipTokens {
                    unchecked_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    unchecked_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    unchecked_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    filled_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    filled_fg: white(),
                    light_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    light_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    subtle_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    subtle_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    outline_border: (Rgba::try_from(PaletteCatalog::scale(primary)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    outline_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    ghost_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    default_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    default_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    default_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                badge: BadgeTokens {
                    filled_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    filled_fg: white(),
                    light_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    light_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    subtle_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    subtle_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    outline_border: (Rgba::try_from(PaletteCatalog::scale(primary)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    outline_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    default_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    default_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    default_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                accordion: AccordionTokens {
                    item_bg: white(),
                    item_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    content: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    chevron: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                menu: MenuTokens {
                    dropdown_bg: white(),
                    dropdown_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    icon: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                progress: ProgressTokens {
                    track_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fill_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                slider: SliderTokens {
                    track_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fill_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    thumb_bg: white(),
                    thumb_border: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    value: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                overlay: OverlayTokens {
                    bg: (Rgba::try_from("#000000E6")
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                loading_overlay: LoadingOverlayTokens {
                    bg: (Rgba::try_from("#000000E6")
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    loader_color: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: white(),
                },
                popover: PopoverTokens {
                    bg: white(),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    title: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    body: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                tooltip: TooltipTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fg: white(),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                hover_card: HoverCardTokens {
                    bg: white(),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    title: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    body: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                select: SelectTokens {
                    bg: white(),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    placeholder: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_error: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Red)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    dropdown_bg: white(),
                    dropdown_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    option_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[9 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    option_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    option_selected_bg: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    tag_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    tag_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    tag_border: (Rgba::try_from(PaletteCatalog::scale(primary)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    icon: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                modal: ModalTokens {
                    panel_bg: white(),
                    panel_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    overlay_bg: (Rgba::try_from("#000000E6")
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    title: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    body: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                toast: ToastTokens {
                    info_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Blue)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    info_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Blue)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    success_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Green)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    success_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Green)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    warning_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Yellow)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    warning_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Yellow)[9 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    error_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    error_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                divider: DividerTokens {
                    line: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                scroll_area: ScrollAreaTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                drawer: DrawerTokens {
                    panel_bg: white(),
                    panel_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    overlay_bg: (Rgba::try_from("#000000E6")
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    title: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    body: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                app_shell: AppShellTokens { bg: white() },
                title_bar: TitleBarTokens {
                    bg: transparent_black(),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    controls_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                sidebar: SidebarTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    header_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    content_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    footer_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                text: TextTokens {
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    secondary: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    muted: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    accent: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    success: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Green)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    warning: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Yellow)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                title: TitleTokens {
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    subtitle: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    gap: px(4.0),
                    subtitle_size: px(15.0),
                    subtitle_line_height: px(22.0),
                    subtitle_weight: FontWeight::NORMAL,
                    h1: TitleLevelTokens {
                        font_size: px(34.0),
                        line_height: px(44.0),
                        weight: FontWeight::BOLD,
                    },
                    h2: TitleLevelTokens {
                        font_size: px(28.0),
                        line_height: px(38.0),
                        weight: FontWeight::BOLD,
                    },
                    h3: TitleLevelTokens {
                        font_size: px(24.0),
                        line_height: px(34.0),
                        weight: FontWeight::SEMIBOLD,
                    },
                    h4: TitleLevelTokens {
                        font_size: px(20.0),
                        line_height: px(30.0),
                        weight: FontWeight::SEMIBOLD,
                    },
                    h5: TitleLevelTokens {
                        font_size: px(17.0),
                        line_height: px(26.0),
                        weight: FontWeight::SEMIBOLD,
                    },
                    h6: TitleLevelTokens {
                        font_size: px(15.0),
                        line_height: px(23.0),
                        weight: FontWeight::MEDIUM,
                    },
                },
                paper: PaperTokens {
                    bg: white(),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                action_icon: ActionIconTokens {
                    filled_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    filled_fg: white(),
                    light_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    light_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    subtle_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    subtle_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    outline_border: (Rgba::try_from(PaletteCatalog::scale(primary)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    outline_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    ghost_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    default_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    default_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    default_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    disabled_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    disabled_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                segmented_control: SegmentedControlTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_active_bg: white(),
                    item_active_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[9 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                textarea: TextareaTokens {
                    bg: white(),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    placeholder: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_error: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Red)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                number_input: NumberInputTokens {
                    bg: white(),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    placeholder: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_error: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Red)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    controls_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    controls_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    controls_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                range_slider: RangeSliderTokens {
                    track_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    range_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    thumb_bg: white(),
                    thumb_border: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    value: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                rating: RatingTokens {
                    active: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Yellow)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    inactive: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                tabs: TabsTokens {
                    list_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    list_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    tab_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    tab_active_bg: white(),
                    tab_active_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[9 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    tab_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    tab_disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    panel_bg: white(),
                    panel_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    panel_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                pagination: PaginationTokens {
                    item_bg: white(),
                    item_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_active_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_active_fg: white(),
                    item_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    dots_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                breadcrumbs: BreadcrumbsTokens {
                    item_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_current_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[9 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    separator: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                table: TableTokens {
                    header_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    header_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    row_bg: white(),
                    row_alt_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    row_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    row_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    cell_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    caption: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                stepper: StepperTokens {
                    step_bg: white(),
                    step_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    step_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    step_active_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    step_active_border: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    step_active_fg: white(),
                    step_completed_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    step_completed_border: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    step_completed_fg: white(),
                    connector: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    panel_bg: white(),
                    panel_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    panel_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                timeline: TimelineTokens {
                    bullet_bg: white(),
                    bullet_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    bullet_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    bullet_active_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    bullet_active_border: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    bullet_active_fg: white(),
                    line: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    line_active: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    title: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    title_active: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[9 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    body: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    card_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    card_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                tree: TreeTokens {
                    row_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    row_selected_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    row_selected_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    row_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    row_disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    line: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
            },
            ColorScheme::Dark => Self {
                button: ButtonTokens {
                    filled_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    filled_fg: white(),
                    light_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    light_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    subtle_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    subtle_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    outline_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    outline_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    ghost_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    disabled_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                input: InputTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    placeholder: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[2 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_error: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Red)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[1 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                radio: RadioTokens {
                    control_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_checked: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    indicator: (Rgba::try_from(PaletteCatalog::scale(primary)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                checkbox: CheckboxTokens {
                    control_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    control_bg_checked: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_checked: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    indicator: white(),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                switch: SwitchTokens {
                    track_off_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    track_on_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    thumb_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                chip: ChipTokens {
                    unchecked_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    unchecked_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    unchecked_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    filled_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    filled_fg: white(),
                    light_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    light_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    subtle_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    subtle_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    outline_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    outline_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    ghost_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[1 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    default_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    default_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    default_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                badge: BadgeTokens {
                    filled_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    filled_fg: white(),
                    light_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    light_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    subtle_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    subtle_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    outline_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    outline_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    default_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    default_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    default_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                accordion: AccordionTokens {
                    item_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    content: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    chevron: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                menu: MenuTokens {
                    dropdown_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    dropdown_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[2 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    icon: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                progress: ProgressTokens {
                    track_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fill_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                slider: SliderTokens {
                    track_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fill_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    thumb_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    thumb_border: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    value: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                overlay: OverlayTokens {
                    bg: (Rgba::try_from("#000000E6")
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                loading_overlay: LoadingOverlayTokens {
                    bg: (Rgba::try_from("#000000E6")
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    loader_color: (Rgba::try_from(PaletteCatalog::scale(primary)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                popover: PopoverTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    title: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    body: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                tooltip: TooltipTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                hover_card: HoverCardTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    title: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    body: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                select: SelectTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    placeholder: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[2 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_error: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Red)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    dropdown_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    dropdown_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    option_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    option_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    option_selected_bg: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[9 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    tag_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    tag_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    tag_border: (Rgba::try_from(PaletteCatalog::scale(primary)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    icon: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[1 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                modal: ModalTokens {
                    panel_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    panel_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    overlay_bg: (Rgba::try_from("#000000E6")
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    title: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    body: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                toast: ToastTokens {
                    info_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Blue)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    info_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Blue)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    success_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Green)[9 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    success_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Green)[2 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    warning_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Yellow)[9 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    warning_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Yellow)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    error_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    error_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                divider: DividerTokens {
                    line: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                scroll_area: ScrollAreaTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                drawer: DrawerTokens {
                    panel_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    panel_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    overlay_bg: (Rgba::try_from("#000000E6")
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    title: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    body: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                app_shell: AppShellTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                title_bar: TitleBarTokens {
                    bg: transparent_black(),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[1 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    controls_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                sidebar: SidebarTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    header_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    content_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[2 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    footer_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                text: TextTokens {
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    secondary: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    muted: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    accent: (Rgba::try_from(PaletteCatalog::scale(primary)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    success: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Green)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    warning: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Yellow)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                title: TitleTokens {
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    subtitle: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    gap: px(4.0),
                    subtitle_size: px(15.0),
                    subtitle_line_height: px(22.0),
                    subtitle_weight: FontWeight::NORMAL,
                    h1: TitleLevelTokens {
                        font_size: px(34.0),
                        line_height: px(44.0),
                        weight: FontWeight::BOLD,
                    },
                    h2: TitleLevelTokens {
                        font_size: px(28.0),
                        line_height: px(38.0),
                        weight: FontWeight::BOLD,
                    },
                    h3: TitleLevelTokens {
                        font_size: px(24.0),
                        line_height: px(34.0),
                        weight: FontWeight::SEMIBOLD,
                    },
                    h4: TitleLevelTokens {
                        font_size: px(20.0),
                        line_height: px(30.0),
                        weight: FontWeight::SEMIBOLD,
                    },
                    h5: TitleLevelTokens {
                        font_size: px(17.0),
                        line_height: px(26.0),
                        weight: FontWeight::SEMIBOLD,
                    },
                    h6: TitleLevelTokens {
                        font_size: px(15.0),
                        line_height: px(23.0),
                        weight: FontWeight::MEDIUM,
                    },
                },
                paper: PaperTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                action_icon: ActionIconTokens {
                    filled_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    filled_fg: white(),
                    light_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    light_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    subtle_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    subtle_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    outline_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    outline_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    ghost_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[1 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    default_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    default_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    default_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    disabled_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    disabled_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                segmented_control: SegmentedControlTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_active_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_active_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                textarea: TextareaTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    placeholder: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[2 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_error: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Red)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[1 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                number_input: NumberInputTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    placeholder: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[2 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_error: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Red)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    controls_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    controls_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    controls_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[1 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                range_slider: RangeSliderTokens {
                    track_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    range_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    thumb_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    thumb_border: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    value: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                rating: RatingTokens {
                    active: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Yellow)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    inactive: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                tabs: TabsTokens {
                    list_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    list_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    tab_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    tab_active_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    tab_active_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    tab_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    tab_disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    panel_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    panel_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    panel_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                pagination: PaginationTokens {
                    item_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[1 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_active_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_active_fg: white(),
                    item_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    dots_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                breadcrumbs: BreadcrumbsTokens {
                    item_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_current_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    separator: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[2 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                table: TableTokens {
                    header_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    header_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[1 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    row_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    row_alt_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    row_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    row_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    cell_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    caption: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                stepper: StepperTokens {
                    step_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    step_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    step_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    step_active_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    step_active_border: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    step_active_fg: white(),
                    step_completed_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    step_completed_border: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    step_completed_fg: white(),
                    connector: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[1 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    panel_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    panel_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    panel_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                timeline: TimelineTokens {
                    bullet_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    bullet_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    bullet_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    bullet_active_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    bullet_active_border: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    bullet_active_fg: white(),
                    line: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    line_active: (Rgba::try_from(PaletteCatalog::scale(primary)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    title: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    title_active: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    body: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    card_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    card_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                },
                tree: TreeTokens {
                    row_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    row_selected_fg: (Rgba::try_from(PaletteCatalog::scale(primary)[1 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    row_selected_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    row_hover_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    row_disabled_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    line: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Theme {
    pub radii: ThemeRadii,
    pub primary_color: PaletteKey,
    pub primary_shade_light: u8,
    pub primary_shade_dark: u8,
    pub color_scheme: ColorScheme,
    pub palette: BTreeMap<PaletteKey, ColorScale>,
    pub semantic: SemanticColors,
    pub components: ComponentTokens,
}

impl Default for Theme {
    fn default() -> Self {
        let primary = PaletteKey::Blue;
        Self {
            radii: ThemeRadii::default(),
            primary_color: primary,
            primary_shade_light: PRIMARY_SHADE_LIGHT_DEFAULT,
            primary_shade_dark: PRIMARY_SHADE_DARK_DEFAULT,
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

    pub fn with_primary_shades(mut self, light: u8, dark: u8) -> Self {
        self.primary_shade_light = light.min(9);
        self.primary_shade_dark = dark.min(9);
        self
    }

    pub fn with_primary_shade(mut self, shade: u8) -> Self {
        let clamped = shade.min(9);
        self.primary_shade_light = clamped;
        self.primary_shade_dark = clamped;
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

    pub fn with_radii(mut self, radii: ThemeRadii) -> Self {
        self.radii = radii;
        self
    }

    pub fn resolve_color<T>(&self, token: T) -> String
    where
        T: ResolveWithTheme<Hsla>,
    {
        format!("{:?}", Rgba::from(token.resolve(self)))
    }

    pub fn resolve_hsla<T>(&self, token: T) -> Hsla
    where
        T: ResolveWithTheme<Hsla>,
    {
        token.resolve(self)
    }

    pub fn resolve_radius<T>(&self, token: T) -> Pixels
    where
        T: ResolveWithTheme<Pixels>,
    {
        token.resolve(self)
    }

    pub fn merged(&self, patch: &ThemeOverrides) -> Self {
        let mut next = self.clone();
        if let Some(primary) = patch.primary_color {
            next = next.with_primary_color(primary);
        }
        if let Some(primary_shade_light) = patch.primary_shade_light {
            next.primary_shade_light = primary_shade_light.min(9);
        }
        if let Some(primary_shade_dark) = patch.primary_shade_dark {
            next.primary_shade_dark = primary_shade_dark.min(9);
        }
        if let Some(color_scheme) = patch.color_scheme {
            next.color_scheme = color_scheme;
        }
        for (key, value) in &patch.palette_overrides {
            next.palette.insert(*key, *value);
        }
        next.radii = patch.radii.apply(next.radii);
        next.semantic = patch.semantic.apply(next.semantic);
        next.components = patch.components.apply(next.components);
        next
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SemanticOverrides {
    pub text_primary: Option<Hsla>,
    pub text_secondary: Option<Hsla>,
    pub text_muted: Option<Hsla>,
    pub bg_canvas: Option<Hsla>,
    pub bg_surface: Option<Hsla>,
    pub bg_soft: Option<Hsla>,
    pub border_subtle: Option<Hsla>,
    pub border_strong: Option<Hsla>,
    pub focus_ring: Option<Hsla>,
    pub status_info: Option<Hsla>,
    pub status_success: Option<Hsla>,
    pub status_warning: Option<Hsla>,
    pub status_error: Option<Hsla>,
    pub overlay_mask: Option<Hsla>,
}

impl SemanticOverrides {
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
pub struct RadiiOverrides {
    pub default: Option<Pixels>,
    pub xs: Option<Pixels>,
    pub sm: Option<Pixels>,
    pub md: Option<Pixels>,
    pub lg: Option<Pixels>,
    pub xl: Option<Pixels>,
    pub pill: Option<Pixels>,
}

impl RadiiOverrides {
    fn apply(&self, mut current: ThemeRadii) -> ThemeRadii {
        if let Some(value) = self.default {
            current.default = value;
        }
        if let Some(value) = self.xs {
            current.xs = value;
        }
        if let Some(value) = self.sm {
            current.sm = value;
        }
        if let Some(value) = self.md {
            current.md = value;
        }
        if let Some(value) = self.lg {
            current.lg = value;
        }
        if let Some(value) = self.xl {
            current.xl = value;
        }
        if let Some(value) = self.pill {
            current.pill = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ButtonOverrides {
    pub filled_bg: Option<Hsla>,
    pub filled_fg: Option<Hsla>,
    pub light_bg: Option<Hsla>,
    pub light_fg: Option<Hsla>,
    pub subtle_bg: Option<Hsla>,
    pub subtle_fg: Option<Hsla>,
    pub outline_border: Option<Hsla>,
    pub outline_fg: Option<Hsla>,
    pub ghost_fg: Option<Hsla>,
    pub disabled_bg: Option<Hsla>,
    pub disabled_fg: Option<Hsla>,
}

impl ButtonOverrides {
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
pub struct InputOverrides {
    pub bg: Option<Hsla>,
    pub fg: Option<Hsla>,
    pub placeholder: Option<Hsla>,
    pub border: Option<Hsla>,
    pub border_focus: Option<Hsla>,
    pub border_error: Option<Hsla>,
    pub label: Option<Hsla>,
    pub description: Option<Hsla>,
    pub error: Option<Hsla>,
}

impl InputOverrides {
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
pub struct RadioOverrides {
    pub control_bg: Option<Hsla>,
    pub border: Option<Hsla>,
    pub border_checked: Option<Hsla>,
    pub indicator: Option<Hsla>,
    pub label: Option<Hsla>,
    pub description: Option<Hsla>,
}

impl RadioOverrides {
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
pub struct CheckboxOverrides {
    pub control_bg: Option<Hsla>,
    pub control_bg_checked: Option<Hsla>,
    pub border: Option<Hsla>,
    pub border_checked: Option<Hsla>,
    pub indicator: Option<Hsla>,
    pub label: Option<Hsla>,
    pub description: Option<Hsla>,
}

impl CheckboxOverrides {
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
pub struct SwitchOverrides {
    pub track_off_bg: Option<Hsla>,
    pub track_on_bg: Option<Hsla>,
    pub thumb_bg: Option<Hsla>,
    pub label: Option<Hsla>,
    pub description: Option<Hsla>,
}

impl SwitchOverrides {
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
pub struct ChipOverrides {
    pub unchecked_bg: Option<Hsla>,
    pub unchecked_fg: Option<Hsla>,
    pub unchecked_border: Option<Hsla>,
    pub filled_bg: Option<Hsla>,
    pub filled_fg: Option<Hsla>,
    pub light_bg: Option<Hsla>,
    pub light_fg: Option<Hsla>,
    pub subtle_bg: Option<Hsla>,
    pub subtle_fg: Option<Hsla>,
    pub outline_border: Option<Hsla>,
    pub outline_fg: Option<Hsla>,
    pub ghost_fg: Option<Hsla>,
    pub default_bg: Option<Hsla>,
    pub default_fg: Option<Hsla>,
    pub default_border: Option<Hsla>,
}

impl ChipOverrides {
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
pub struct BadgeOverrides {
    pub filled_bg: Option<Hsla>,
    pub filled_fg: Option<Hsla>,
    pub light_bg: Option<Hsla>,
    pub light_fg: Option<Hsla>,
    pub subtle_bg: Option<Hsla>,
    pub subtle_fg: Option<Hsla>,
    pub outline_border: Option<Hsla>,
    pub outline_fg: Option<Hsla>,
    pub default_bg: Option<Hsla>,
    pub default_fg: Option<Hsla>,
    pub default_border: Option<Hsla>,
}

impl BadgeOverrides {
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
pub struct AccordionOverrides {
    pub item_bg: Option<Hsla>,
    pub item_border: Option<Hsla>,
    pub label: Option<Hsla>,
    pub description: Option<Hsla>,
    pub content: Option<Hsla>,
    pub chevron: Option<Hsla>,
}

impl AccordionOverrides {
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
pub struct MenuOverrides {
    pub dropdown_bg: Option<Hsla>,
    pub dropdown_border: Option<Hsla>,
    pub item_fg: Option<Hsla>,
    pub item_hover_bg: Option<Hsla>,
    pub item_disabled_fg: Option<Hsla>,
    pub icon: Option<Hsla>,
}

impl MenuOverrides {
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
pub struct ProgressOverrides {
    pub track_bg: Option<Hsla>,
    pub fill_bg: Option<Hsla>,
    pub label: Option<Hsla>,
}

impl ProgressOverrides {
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
pub struct SliderOverrides {
    pub track_bg: Option<Hsla>,
    pub fill_bg: Option<Hsla>,
    pub thumb_bg: Option<Hsla>,
    pub thumb_border: Option<Hsla>,
    pub label: Option<Hsla>,
    pub value: Option<Hsla>,
}

impl SliderOverrides {
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
pub struct OverlayOverrides {
    pub bg: Option<Hsla>,
}

impl OverlayOverrides {
    fn apply(&self, mut current: OverlayTokens) -> OverlayTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LoadingOverlayOverrides {
    pub bg: Option<Hsla>,
    pub loader_color: Option<Hsla>,
    pub label: Option<Hsla>,
}

impl LoadingOverlayOverrides {
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
pub struct PopoverOverrides {
    pub bg: Option<Hsla>,
    pub border: Option<Hsla>,
    pub title: Option<Hsla>,
    pub body: Option<Hsla>,
}

impl PopoverOverrides {
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
pub struct TooltipOverrides {
    pub bg: Option<Hsla>,
    pub fg: Option<Hsla>,
    pub border: Option<Hsla>,
}

impl TooltipOverrides {
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
pub struct HoverCardOverrides {
    pub bg: Option<Hsla>,
    pub border: Option<Hsla>,
    pub title: Option<Hsla>,
    pub body: Option<Hsla>,
}

impl HoverCardOverrides {
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
pub struct SelectOverrides {
    pub bg: Option<Hsla>,
    pub fg: Option<Hsla>,
    pub placeholder: Option<Hsla>,
    pub border: Option<Hsla>,
    pub border_focus: Option<Hsla>,
    pub border_error: Option<Hsla>,
    pub dropdown_bg: Option<Hsla>,
    pub dropdown_border: Option<Hsla>,
    pub option_fg: Option<Hsla>,
    pub option_hover_bg: Option<Hsla>,
    pub option_selected_bg: Option<Hsla>,
    pub tag_bg: Option<Hsla>,
    pub tag_fg: Option<Hsla>,
    pub tag_border: Option<Hsla>,
    pub icon: Option<Hsla>,
    pub label: Option<Hsla>,
    pub description: Option<Hsla>,
    pub error: Option<Hsla>,
}

impl SelectOverrides {
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
pub struct ModalOverrides {
    pub panel_bg: Option<Hsla>,
    pub panel_border: Option<Hsla>,
    pub overlay_bg: Option<Hsla>,
    pub title: Option<Hsla>,
    pub body: Option<Hsla>,
}

impl ModalOverrides {
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
pub struct ToastOverrides {
    pub info_bg: Option<Hsla>,
    pub info_fg: Option<Hsla>,
    pub success_bg: Option<Hsla>,
    pub success_fg: Option<Hsla>,
    pub warning_bg: Option<Hsla>,
    pub warning_fg: Option<Hsla>,
    pub error_bg: Option<Hsla>,
    pub error_fg: Option<Hsla>,
}

impl ToastOverrides {
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
pub struct DividerOverrides {
    pub line: Option<Hsla>,
    pub label: Option<Hsla>,
}

impl DividerOverrides {
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
pub struct ScrollAreaOverrides {
    pub bg: Option<Hsla>,
    pub border: Option<Hsla>,
}

impl ScrollAreaOverrides {
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
pub struct DrawerOverrides {
    pub panel_bg: Option<Hsla>,
    pub panel_border: Option<Hsla>,
    pub overlay_bg: Option<Hsla>,
    pub title: Option<Hsla>,
    pub body: Option<Hsla>,
}

impl DrawerOverrides {
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
pub struct AppShellOverrides {
    pub bg: Option<Hsla>,
}

impl AppShellOverrides {
    fn apply(&self, mut current: AppShellTokens) -> AppShellTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TitleBarOverrides {
    pub bg: Option<Hsla>,
    pub border: Option<Hsla>,
    pub fg: Option<Hsla>,
    pub controls_bg: Option<Hsla>,
}

impl TitleBarOverrides {
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
pub struct SidebarOverrides {
    pub bg: Option<Hsla>,
    pub border: Option<Hsla>,
    pub header_fg: Option<Hsla>,
    pub content_fg: Option<Hsla>,
    pub footer_fg: Option<Hsla>,
}

impl SidebarOverrides {
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
pub struct TextOverrides {
    pub fg: Option<Hsla>,
    pub secondary: Option<Hsla>,
    pub muted: Option<Hsla>,
    pub accent: Option<Hsla>,
    pub success: Option<Hsla>,
    pub warning: Option<Hsla>,
    pub error: Option<Hsla>,
}

impl TextOverrides {
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
pub struct TitleOverrides {
    pub fg: Option<Hsla>,
    pub subtitle: Option<Hsla>,
    pub gap: Option<Pixels>,
    pub subtitle_size: Option<Pixels>,
    pub subtitle_line_height: Option<Pixels>,
    pub subtitle_weight: Option<FontWeight>,
    pub h1: TitleLevelOverrides,
    pub h2: TitleLevelOverrides,
    pub h3: TitleLevelOverrides,
    pub h4: TitleLevelOverrides,
    pub h5: TitleLevelOverrides,
    pub h6: TitleLevelOverrides,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TitleLevelOverrides {
    pub font_size: Option<Pixels>,
    pub line_height: Option<Pixels>,
    pub weight: Option<FontWeight>,
}

impl TitleLevelOverrides {
    fn apply(&self, mut current: TitleLevelTokens) -> TitleLevelTokens {
        if let Some(value) = &self.font_size {
            current.font_size = *value;
        }
        if let Some(value) = &self.line_height {
            current.line_height = *value;
        }
        if let Some(value) = &self.weight {
            current.weight = *value;
        }
        current
    }
}

impl TitleOverrides {
    fn apply(&self, mut current: TitleTokens) -> TitleTokens {
        if let Some(value) = &self.fg {
            current.fg = value.clone();
        }
        if let Some(value) = &self.subtitle {
            current.subtitle = value.clone();
        }
        if let Some(value) = &self.gap {
            current.gap = *value;
        }
        if let Some(value) = &self.subtitle_size {
            current.subtitle_size = *value;
        }
        if let Some(value) = &self.subtitle_line_height {
            current.subtitle_line_height = *value;
        }
        if let Some(value) = &self.subtitle_weight {
            current.subtitle_weight = *value;
        }
        current.h1 = self.h1.apply(current.h1);
        current.h2 = self.h2.apply(current.h2);
        current.h3 = self.h3.apply(current.h3);
        current.h4 = self.h4.apply(current.h4);
        current.h5 = self.h5.apply(current.h5);
        current.h6 = self.h6.apply(current.h6);
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PaperOverrides {
    pub bg: Option<Hsla>,
    pub border: Option<Hsla>,
}

impl PaperOverrides {
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
pub struct ActionIconOverrides {
    pub filled_bg: Option<Hsla>,
    pub filled_fg: Option<Hsla>,
    pub light_bg: Option<Hsla>,
    pub light_fg: Option<Hsla>,
    pub subtle_bg: Option<Hsla>,
    pub subtle_fg: Option<Hsla>,
    pub outline_border: Option<Hsla>,
    pub outline_fg: Option<Hsla>,
    pub ghost_fg: Option<Hsla>,
    pub default_bg: Option<Hsla>,
    pub default_fg: Option<Hsla>,
    pub default_border: Option<Hsla>,
    pub disabled_bg: Option<Hsla>,
    pub disabled_fg: Option<Hsla>,
    pub disabled_border: Option<Hsla>,
}

impl ActionIconOverrides {
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
pub struct SegmentedControlOverrides {
    pub bg: Option<Hsla>,
    pub border: Option<Hsla>,
    pub item_fg: Option<Hsla>,
    pub item_active_bg: Option<Hsla>,
    pub item_active_fg: Option<Hsla>,
    pub item_hover_bg: Option<Hsla>,
    pub item_disabled_fg: Option<Hsla>,
}

impl SegmentedControlOverrides {
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
pub struct TextareaOverrides {
    pub bg: Option<Hsla>,
    pub fg: Option<Hsla>,
    pub placeholder: Option<Hsla>,
    pub border: Option<Hsla>,
    pub border_focus: Option<Hsla>,
    pub border_error: Option<Hsla>,
    pub label: Option<Hsla>,
    pub description: Option<Hsla>,
    pub error: Option<Hsla>,
}

impl TextareaOverrides {
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
pub struct NumberInputOverrides {
    pub bg: Option<Hsla>,
    pub fg: Option<Hsla>,
    pub placeholder: Option<Hsla>,
    pub border: Option<Hsla>,
    pub border_focus: Option<Hsla>,
    pub border_error: Option<Hsla>,
    pub controls_bg: Option<Hsla>,
    pub controls_fg: Option<Hsla>,
    pub controls_border: Option<Hsla>,
    pub label: Option<Hsla>,
    pub description: Option<Hsla>,
    pub error: Option<Hsla>,
}

impl NumberInputOverrides {
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
pub struct RangeSliderOverrides {
    pub track_bg: Option<Hsla>,
    pub range_bg: Option<Hsla>,
    pub thumb_bg: Option<Hsla>,
    pub thumb_border: Option<Hsla>,
    pub label: Option<Hsla>,
    pub value: Option<Hsla>,
}

impl RangeSliderOverrides {
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
pub struct RatingOverrides {
    pub active: Option<Hsla>,
    pub inactive: Option<Hsla>,
}

impl RatingOverrides {
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
pub struct TabsOverrides {
    pub list_bg: Option<Hsla>,
    pub list_border: Option<Hsla>,
    pub tab_fg: Option<Hsla>,
    pub tab_active_bg: Option<Hsla>,
    pub tab_active_fg: Option<Hsla>,
    pub tab_hover_bg: Option<Hsla>,
    pub tab_disabled_fg: Option<Hsla>,
    pub panel_bg: Option<Hsla>,
    pub panel_border: Option<Hsla>,
    pub panel_fg: Option<Hsla>,
}

impl TabsOverrides {
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
pub struct PaginationOverrides {
    pub item_bg: Option<Hsla>,
    pub item_border: Option<Hsla>,
    pub item_fg: Option<Hsla>,
    pub item_active_bg: Option<Hsla>,
    pub item_active_fg: Option<Hsla>,
    pub item_hover_bg: Option<Hsla>,
    pub item_disabled_fg: Option<Hsla>,
    pub dots_fg: Option<Hsla>,
}

impl PaginationOverrides {
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
pub struct BreadcrumbsOverrides {
    pub item_fg: Option<Hsla>,
    pub item_current_fg: Option<Hsla>,
    pub separator: Option<Hsla>,
    pub item_hover_bg: Option<Hsla>,
}

impl BreadcrumbsOverrides {
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
pub struct TableOverrides {
    pub header_bg: Option<Hsla>,
    pub header_fg: Option<Hsla>,
    pub row_bg: Option<Hsla>,
    pub row_alt_bg: Option<Hsla>,
    pub row_hover_bg: Option<Hsla>,
    pub row_border: Option<Hsla>,
    pub cell_fg: Option<Hsla>,
    pub caption: Option<Hsla>,
}

impl TableOverrides {
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
pub struct StepperOverrides {
    pub step_bg: Option<Hsla>,
    pub step_border: Option<Hsla>,
    pub step_fg: Option<Hsla>,
    pub step_active_bg: Option<Hsla>,
    pub step_active_border: Option<Hsla>,
    pub step_active_fg: Option<Hsla>,
    pub step_completed_bg: Option<Hsla>,
    pub step_completed_border: Option<Hsla>,
    pub step_completed_fg: Option<Hsla>,
    pub connector: Option<Hsla>,
    pub label: Option<Hsla>,
    pub description: Option<Hsla>,
    pub panel_bg: Option<Hsla>,
    pub panel_border: Option<Hsla>,
    pub panel_fg: Option<Hsla>,
}

impl StepperOverrides {
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
pub struct TimelineOverrides {
    pub bullet_bg: Option<Hsla>,
    pub bullet_border: Option<Hsla>,
    pub bullet_fg: Option<Hsla>,
    pub bullet_active_bg: Option<Hsla>,
    pub bullet_active_border: Option<Hsla>,
    pub bullet_active_fg: Option<Hsla>,
    pub line: Option<Hsla>,
    pub line_active: Option<Hsla>,
    pub title: Option<Hsla>,
    pub title_active: Option<Hsla>,
    pub body: Option<Hsla>,
    pub card_bg: Option<Hsla>,
    pub card_border: Option<Hsla>,
}

impl TimelineOverrides {
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
pub struct TreeOverrides {
    pub row_fg: Option<Hsla>,
    pub row_selected_fg: Option<Hsla>,
    pub row_selected_bg: Option<Hsla>,
    pub row_hover_bg: Option<Hsla>,
    pub row_disabled_fg: Option<Hsla>,
    pub line: Option<Hsla>,
}

impl TreeOverrides {
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
pub struct ComponentOverrides {
    pub button: ButtonOverrides,
    pub input: InputOverrides,
    pub radio: RadioOverrides,
    pub checkbox: CheckboxOverrides,
    pub switch: SwitchOverrides,
    pub chip: ChipOverrides,
    pub badge: BadgeOverrides,
    pub accordion: AccordionOverrides,
    pub menu: MenuOverrides,
    pub progress: ProgressOverrides,
    pub slider: SliderOverrides,
    pub overlay: OverlayOverrides,
    pub loading_overlay: LoadingOverlayOverrides,
    pub popover: PopoverOverrides,
    pub tooltip: TooltipOverrides,
    pub hover_card: HoverCardOverrides,
    pub select: SelectOverrides,
    pub modal: ModalOverrides,
    pub toast: ToastOverrides,
    pub divider: DividerOverrides,
    pub scroll_area: ScrollAreaOverrides,
    pub drawer: DrawerOverrides,
    pub app_shell: AppShellOverrides,
    pub title_bar: TitleBarOverrides,
    pub sidebar: SidebarOverrides,
    pub text: TextOverrides,
    pub title: TitleOverrides,
    pub paper: PaperOverrides,
    pub action_icon: ActionIconOverrides,
    pub segmented_control: SegmentedControlOverrides,
    pub textarea: TextareaOverrides,
    pub number_input: NumberInputOverrides,
    pub range_slider: RangeSliderOverrides,
    pub rating: RatingOverrides,
    pub tabs: TabsOverrides,
    pub pagination: PaginationOverrides,
    pub breadcrumbs: BreadcrumbsOverrides,
    pub table: TableOverrides,
    pub stepper: StepperOverrides,
    pub timeline: TimelineOverrides,
    pub tree: TreeOverrides,
}

impl ComponentOverrides {
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
pub struct ThemeOverrides {
    pub primary_color: Option<PaletteKey>,
    pub primary_shade_light: Option<u8>,
    pub primary_shade_dark: Option<u8>,
    pub color_scheme: Option<ColorScheme>,
    pub palette_overrides: BTreeMap<PaletteKey, ColorScale>,
    pub radii: RadiiOverrides,
    pub semantic: SemanticOverrides,
    pub components: ComponentOverrides,
}

#[derive(Clone, Debug, Default)]
pub struct LocalTheme {
    resolved: Option<Arc<Theme>>,
    component_overrides: Option<ComponentOverrides>,
}

impl LocalTheme {
    pub fn with_component_overrides(mut self, overrides: ComponentOverrides) -> Self {
        self.component_overrides = Some(overrides);
        self
    }

    pub fn set_component_overrides(&mut self, overrides: Option<ComponentOverrides>) {
        self.component_overrides = overrides;
        self.resolved = None;
    }

    pub fn update_component_overrides(
        &mut self,
        configure: impl FnOnce(ComponentOverrides) -> ComponentOverrides,
    ) {
        let current = self.component_overrides.take().unwrap_or_default();
        self.component_overrides = Some(configure(current));
        self.resolved = None;
    }

    pub fn sync_from_provider(&mut self, cx: &gpui::App) {
        let base = crate::provider::CalmProvider::theme_arc_or_default(cx);
        if let Some(component_overrides) = &self.component_overrides {
            let mut merged = base.as_ref().clone();
            merged.components = component_overrides.apply(merged.components);
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
        assert_eq!(theme.primary_shade_light, PRIMARY_SHADE_LIGHT_DEFAULT);
        assert_eq!(theme.primary_shade_dark, PRIMARY_SHADE_DARK_DEFAULT);
    }

    #[test]
    fn default_palette_is_complete() {
        let theme = Theme::default();
        assert_eq!(theme.palette.len(), 14);
        assert_eq!(theme.palette[&PaletteKey::Blue].len(), COLOR_STOPS);
    }

    #[test]
    fn nested_theme_overrides_override_only_target_fields() {
        let base = Theme::default();
        let overrides = ThemeOverrides {
            semantic: SemanticOverrides {
                text_primary: Some(
                    Rgba::try_from(PaletteCatalog::scale(PaletteKey::Orange)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black()),
                ),
                ..SemanticOverrides::default()
            },
            ..ThemeOverrides::default()
        };
        let next = base.merged(&overrides);
        assert_eq!(
            next.semantic.text_primary,
            (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Orange)[8 as usize])
                .map(Into::into)
                .unwrap_or_else(|_| black()))
        );
        assert_eq!(next.semantic.text_secondary, base.semantic.text_secondary);
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

        let dark_checkbox = dark.components.checkbox.label;
        let light_checkbox = light.components.checkbox.label;
        let dark_radio = dark.components.radio.label;
        let light_radio = light.components.radio.label;
        let dark_switch = dark.components.switch.label;
        let light_switch = light.components.switch.label;

        assert_ne!(dark_checkbox, light_checkbox);
        assert_ne!(dark_radio, light_radio);
        assert_ne!(dark_switch, light_switch);
    }
}
