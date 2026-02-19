use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use crate::style::{Radius, Size};
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
    pub sizes: ButtonSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ButtonSizePreset {
    pub font_size: Pixels,
    pub line_height: Pixels,
    pub padding_x: Pixels,
    pub padding_y: Pixels,
    pub content_gap: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ButtonSizeScale {
    pub xs: ButtonSizePreset,
    pub sm: ButtonSizePreset,
    pub md: ButtonSizePreset,
    pub lg: ButtonSizePreset,
    pub xl: ButtonSizePreset,
}

impl ButtonSizeScale {
    pub fn for_size(&self, size: Size) -> ButtonSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_button_size_scale() -> ButtonSizeScale {
    ButtonSizeScale {
        xs: ButtonSizePreset {
            font_size: px(12.0),
            line_height: px(16.0),
            padding_x: px(8.0),
            padding_y: px(4.0),
            content_gap: px(6.0),
        },
        sm: ButtonSizePreset {
            font_size: px(13.0),
            line_height: px(18.0),
            padding_x: px(10.0),
            padding_y: px(6.0),
            content_gap: px(6.0),
        },
        md: ButtonSizePreset {
            font_size: px(14.0),
            line_height: px(20.0),
            padding_x: px(12.0),
            padding_y: px(8.0),
            content_gap: px(8.0),
        },
        lg: ButtonSizePreset {
            font_size: px(16.0),
            line_height: px(22.0),
            padding_x: px(14.0),
            padding_y: px(10.0),
            content_gap: px(8.0),
        },
        xl: ButtonSizePreset {
            font_size: px(18.0),
            line_height: px(24.0),
            padding_x: px(16.0),
            padding_y: px(12.0),
            content_gap: px(10.0),
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldSizePreset {
    pub font_size: Pixels,
    pub line_height: Pixels,
    pub padding_x: Pixels,
    pub padding_y: Pixels,
    pub caret_height: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldSizeScale {
    pub xs: FieldSizePreset,
    pub sm: FieldSizePreset,
    pub md: FieldSizePreset,
    pub lg: FieldSizePreset,
    pub xl: FieldSizePreset,
}

impl FieldSizeScale {
    pub fn for_size(&self, size: Size) -> FieldSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_field_size_scale() -> FieldSizeScale {
    FieldSizeScale {
        xs: FieldSizePreset {
            font_size: px(12.0),
            line_height: px(18.0),
            padding_x: px(8.0),
            padding_y: px(5.0),
            caret_height: px(13.0),
        },
        sm: FieldSizePreset {
            font_size: px(14.0),
            line_height: px(20.0),
            padding_x: px(10.0),
            padding_y: px(6.0),
            caret_height: px(15.0),
        },
        md: FieldSizePreset {
            font_size: px(16.0),
            line_height: px(22.0),
            padding_x: px(12.0),
            padding_y: px(8.0),
            caret_height: px(17.0),
        },
        lg: FieldSizePreset {
            font_size: px(18.0),
            line_height: px(24.0),
            padding_x: px(14.0),
            padding_y: px(10.0),
            caret_height: px(19.0),
        },
        xl: FieldSizePreset {
            font_size: px(20.0),
            line_height: px(26.0),
            padding_x: px(16.0),
            padding_y: px(12.0),
            caret_height: px(21.0),
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InsetSizeScale {
    pub xs: Pixels,
    pub sm: Pixels,
    pub md: Pixels,
    pub lg: Pixels,
    pub xl: Pixels,
}

impl InsetSizeScale {
    pub fn for_size(&self, size: Size) -> Pixels {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_inset_size_scale() -> InsetSizeScale {
    InsetSizeScale {
        xs: px(4.0),
        sm: px(8.0),
        md: px(12.0),
        lg: px(16.0),
        xl: px(20.0),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GapSizeScale {
    pub xs: Pixels,
    pub sm: Pixels,
    pub md: Pixels,
    pub lg: Pixels,
    pub xl: Pixels,
}

impl GapSizeScale {
    pub fn for_size(&self, size: Size) -> Pixels {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_layout_gap_scale() -> GapSizeScale {
    GapSizeScale {
        xs: px(4.0),
        sm: px(6.0),
        md: px(8.0),
        lg: px(12.0),
        xl: px(16.0),
    }
}

fn default_layout_space_scale() -> GapSizeScale {
    GapSizeScale {
        xs: px(4.0),
        sm: px(6.0),
        md: px(8.0),
        lg: px(12.0),
        xl: px(16.0),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputTokens {
    pub bg: Hsla,
    pub fg: Hsla,
    pub caret: Hsla,
    pub selection_bg: Hsla,
    pub placeholder: Hsla,
    pub border: Hsla,
    pub border_focus: Hsla,
    pub border_error: Hsla,
    pub label: Hsla,
    pub label_size: Pixels,
    pub label_weight: FontWeight,
    pub description: Hsla,
    pub description_size: Pixels,
    pub error: Hsla,
    pub error_size: Pixels,
    pub label_block_gap: Pixels,
    pub label_row_gap: Pixels,
    pub slot_fg: Hsla,
    pub slot_gap: Pixels,
    pub slot_min_width: Pixels,
    pub layout_gap_vertical: Pixels,
    pub layout_gap_horizontal: Pixels,
    pub horizontal_label_width: Pixels,
    pub pin_cells_gap: Pixels,
    pub pin_error_gap: Pixels,
    pub sizes: FieldSizeScale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RadioTokens {
    pub control_bg: Hsla,
    pub border: Hsla,
    pub border_hover: Hsla,
    pub border_focus: Hsla,
    pub border_checked: Hsla,
    pub indicator: Hsla,
    pub label: Hsla,
    pub description: Hsla,
    pub label_description_gap: Pixels,
    pub group_gap_horizontal: Pixels,
    pub group_gap_vertical: Pixels,
    pub sizes: ChoiceControlSizeScale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckboxTokens {
    pub control_bg: Hsla,
    pub control_bg_checked: Hsla,
    pub border: Hsla,
    pub border_hover: Hsla,
    pub border_focus: Hsla,
    pub border_checked: Hsla,
    pub indicator: Hsla,
    pub label: Hsla,
    pub description: Hsla,
    pub label_description_gap: Pixels,
    pub group_gap_horizontal: Pixels,
    pub group_gap_vertical: Pixels,
    pub sizes: ChoiceControlSizeScale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwitchTokens {
    pub track_off_bg: Hsla,
    pub track_on_bg: Hsla,
    pub track_hover_border: Hsla,
    pub track_focus_border: Hsla,
    pub thumb_bg: Hsla,
    pub label: Hsla,
    pub description: Hsla,
    pub label_description_gap: Pixels,
    pub sizes: SwitchSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChoiceControlSizePreset {
    pub control_size: Pixels,
    pub indicator_size: Pixels,
    pub label_size: Pixels,
    pub description_size: Pixels,
    pub content_gap: Pixels,
    pub description_indent_gap: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChoiceControlSizeScale {
    pub xs: ChoiceControlSizePreset,
    pub sm: ChoiceControlSizePreset,
    pub md: ChoiceControlSizePreset,
    pub lg: ChoiceControlSizePreset,
    pub xl: ChoiceControlSizePreset,
}

impl ChoiceControlSizeScale {
    pub fn for_size(&self, size: Size) -> ChoiceControlSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_choice_control_size_scale() -> ChoiceControlSizeScale {
    ChoiceControlSizeScale {
        xs: ChoiceControlSizePreset {
            control_size: px(12.0),
            indicator_size: px(6.0),
            label_size: px(12.0),
            description_size: px(12.0),
            content_gap: px(8.0),
            description_indent_gap: px(8.0),
        },
        sm: ChoiceControlSizePreset {
            control_size: px(14.0),
            indicator_size: px(7.0),
            label_size: px(13.0),
            description_size: px(12.0),
            content_gap: px(8.0),
            description_indent_gap: px(8.0),
        },
        md: ChoiceControlSizePreset {
            control_size: px(16.0),
            indicator_size: px(8.0),
            label_size: px(14.0),
            description_size: px(13.0),
            content_gap: px(8.0),
            description_indent_gap: px(8.0),
        },
        lg: ChoiceControlSizePreset {
            control_size: px(18.0),
            indicator_size: px(9.0),
            label_size: px(16.0),
            description_size: px(14.0),
            content_gap: px(10.0),
            description_indent_gap: px(8.0),
        },
        xl: ChoiceControlSizePreset {
            control_size: px(20.0),
            indicator_size: px(10.0),
            label_size: px(18.0),
            description_size: px(15.0),
            content_gap: px(10.0),
            description_indent_gap: px(8.0),
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SwitchSizePreset {
    pub track_width: Pixels,
    pub track_height: Pixels,
    pub thumb_size: Pixels,
    pub label_size: Pixels,
    pub description_size: Pixels,
    pub label_gap: Pixels,
    pub description_indent_gap: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SwitchSizeScale {
    pub xs: SwitchSizePreset,
    pub sm: SwitchSizePreset,
    pub md: SwitchSizePreset,
    pub lg: SwitchSizePreset,
    pub xl: SwitchSizePreset,
}

impl SwitchSizeScale {
    pub fn for_size(&self, size: Size) -> SwitchSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_switch_size_scale() -> SwitchSizeScale {
    SwitchSizeScale {
        xs: SwitchSizePreset {
            track_width: px(26.0),
            track_height: px(14.0),
            thumb_size: px(10.0),
            label_size: px(12.0),
            description_size: px(12.0),
            label_gap: px(8.0),
            description_indent_gap: px(8.0),
        },
        sm: SwitchSizePreset {
            track_width: px(30.0),
            track_height: px(16.0),
            thumb_size: px(12.0),
            label_size: px(13.0),
            description_size: px(12.0),
            label_gap: px(8.0),
            description_indent_gap: px(8.0),
        },
        md: SwitchSizePreset {
            track_width: px(36.0),
            track_height: px(20.0),
            thumb_size: px(16.0),
            label_size: px(14.0),
            description_size: px(13.0),
            label_gap: px(8.0),
            description_indent_gap: px(8.0),
        },
        lg: SwitchSizePreset {
            track_width: px(42.0),
            track_height: px(24.0),
            thumb_size: px(20.0),
            label_size: px(16.0),
            description_size: px(14.0),
            label_gap: px(10.0),
            description_indent_gap: px(8.0),
        },
        xl: SwitchSizePreset {
            track_width: px(48.0),
            track_height: px(28.0),
            thumb_size: px(24.0),
            label_size: px(18.0),
            description_size: px(15.0),
            label_gap: px(10.0),
            description_indent_gap: px(8.0),
        },
    }
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
    pub border_hover: Hsla,
    pub border_focus: Hsla,
    pub content_gap: Pixels,
    pub indicator_size: Pixels,
    pub group_gap_horizontal: Pixels,
    pub group_gap_vertical: Pixels,
    pub sizes: ButtonSizeScale,
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
    pub sizes: BadgeSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BadgeSizePreset {
    pub font_size: Pixels,
    pub padding_x: Pixels,
    pub padding_y: Pixels,
    pub gap: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BadgeSizeScale {
    pub xs: BadgeSizePreset,
    pub sm: BadgeSizePreset,
    pub md: BadgeSizePreset,
    pub lg: BadgeSizePreset,
    pub xl: BadgeSizePreset,
}

impl BadgeSizeScale {
    pub fn for_size(&self, size: Size) -> BadgeSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_badge_size_scale() -> BadgeSizeScale {
    BadgeSizeScale {
        xs: BadgeSizePreset {
            font_size: px(12.0),
            padding_x: px(6.0),
            padding_y: px(1.0),
            gap: px(4.0),
        },
        sm: BadgeSizePreset {
            font_size: px(12.0),
            padding_x: px(8.0),
            padding_y: px(2.0),
            gap: px(4.0),
        },
        md: BadgeSizePreset {
            font_size: px(13.0),
            padding_x: px(10.0),
            padding_y: px(3.0),
            gap: px(4.0),
        },
        lg: BadgeSizePreset {
            font_size: px(14.0),
            padding_x: px(12.0),
            padding_y: px(4.0),
            gap: px(6.0),
        },
        xl: BadgeSizePreset {
            font_size: px(16.0),
            padding_x: px(14.0),
            padding_y: px(5.0),
            gap: px(6.0),
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccordionTokens {
    pub item_bg: Hsla,
    pub item_border: Hsla,
    pub label: Hsla,
    pub description: Hsla,
    pub content: Hsla,
    pub chevron: Hsla,
    pub stack_gap: Pixels,
    pub header_gap: Pixels,
    pub label_stack_gap: Pixels,
    pub panel_gap: Pixels,
    pub sizes: AccordionSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccordionSizePreset {
    pub label_size: Pixels,
    pub description_size: Pixels,
    pub content_size: Pixels,
    pub chevron_size: Pixels,
    pub header_padding_x: Pixels,
    pub header_padding_y: Pixels,
    pub panel_padding_x: Pixels,
    pub panel_padding_bottom: Pixels,
    pub panel_padding_top: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccordionSizeScale {
    pub xs: AccordionSizePreset,
    pub sm: AccordionSizePreset,
    pub md: AccordionSizePreset,
    pub lg: AccordionSizePreset,
    pub xl: AccordionSizePreset,
}

impl AccordionSizeScale {
    pub fn for_size(&self, size: Size) -> AccordionSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_accordion_size_scale() -> AccordionSizeScale {
    AccordionSizeScale {
        xs: AccordionSizePreset {
            label_size: px(12.0),
            description_size: px(11.0),
            content_size: px(12.0),
            chevron_size: px(12.0),
            header_padding_x: px(10.0),
            header_padding_y: px(8.0),
            panel_padding_x: px(10.0),
            panel_padding_bottom: px(8.0),
            panel_padding_top: px(1.0),
        },
        sm: AccordionSizePreset {
            label_size: px(13.0),
            description_size: px(12.0),
            content_size: px(13.0),
            chevron_size: px(13.0),
            header_padding_x: px(11.0),
            header_padding_y: px(9.0),
            panel_padding_x: px(11.0),
            panel_padding_bottom: px(9.0),
            panel_padding_top: px(2.0),
        },
        md: AccordionSizePreset {
            label_size: px(14.0),
            description_size: px(13.0),
            content_size: px(14.0),
            chevron_size: px(14.0),
            header_padding_x: px(12.0),
            header_padding_y: px(10.0),
            panel_padding_x: px(12.0),
            panel_padding_bottom: px(10.0),
            panel_padding_top: px(2.0),
        },
        lg: AccordionSizePreset {
            label_size: px(16.0),
            description_size: px(14.0),
            content_size: px(16.0),
            chevron_size: px(16.0),
            header_padding_x: px(14.0),
            header_padding_y: px(12.0),
            panel_padding_x: px(14.0),
            panel_padding_bottom: px(12.0),
            panel_padding_top: px(3.0),
        },
        xl: AccordionSizePreset {
            label_size: px(18.0),
            description_size: px(15.0),
            content_size: px(18.0),
            chevron_size: px(18.0),
            header_padding_x: px(16.0),
            header_padding_y: px(14.0),
            panel_padding_x: px(16.0),
            panel_padding_bottom: px(14.0),
            panel_padding_top: px(4.0),
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuTokens {
    pub dropdown_bg: Hsla,
    pub dropdown_border: Hsla,
    pub item_fg: Hsla,
    pub item_hover_bg: Hsla,
    pub item_disabled_fg: Hsla,
    pub icon: Hsla,
    pub item_gap: Pixels,
    pub item_padding_x: Pixels,
    pub item_padding_y: Pixels,
    pub item_size: Pixels,
    pub item_icon_size: Pixels,
    pub item_radius: Pixels,
    pub dropdown_padding: Pixels,
    pub dropdown_gap: Pixels,
    pub dropdown_radius: Pixels,
    pub dropdown_width_fallback: Pixels,
    pub dropdown_min_width: Pixels,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgressTokens {
    pub track_bg: Hsla,
    pub fill_bg: Hsla,
    pub label: Hsla,
    pub default_width: Pixels,
    pub min_width: Pixels,
    pub root_gap: Pixels,
    pub sizes: ProgressSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgressSizePreset {
    pub bar_height: Pixels,
    pub label_size: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgressSizeScale {
    pub xs: ProgressSizePreset,
    pub sm: ProgressSizePreset,
    pub md: ProgressSizePreset,
    pub lg: ProgressSizePreset,
    pub xl: ProgressSizePreset,
}

impl ProgressSizeScale {
    pub fn for_size(&self, size: Size) -> ProgressSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_progress_size_scale() -> ProgressSizeScale {
    ProgressSizeScale {
        xs: ProgressSizePreset {
            bar_height: px(4.0),
            label_size: px(12.0),
        },
        sm: ProgressSizePreset {
            bar_height: px(6.0),
            label_size: px(13.0),
        },
        md: ProgressSizePreset {
            bar_height: px(8.0),
            label_size: px(14.0),
        },
        lg: ProgressSizePreset {
            bar_height: px(12.0),
            label_size: px(16.0),
        },
        xl: ProgressSizePreset {
            bar_height: px(16.0),
            label_size: px(18.0),
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliderTokens {
    pub track_bg: Hsla,
    pub fill_bg: Hsla,
    pub thumb_bg: Hsla,
    pub thumb_border: Hsla,
    pub label: Hsla,
    pub value: Hsla,
    pub label_size: Pixels,
    pub value_size: Pixels,
    pub header_gap_vertical: Pixels,
    pub header_gap_horizontal: Pixels,
    pub default_width: Pixels,
    pub min_width: Pixels,
    pub sizes: SliderSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SliderSizePreset {
    pub track_thickness: Pixels,
    pub thumb_size: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SliderSizeScale {
    pub xs: SliderSizePreset,
    pub sm: SliderSizePreset,
    pub md: SliderSizePreset,
    pub lg: SliderSizePreset,
    pub xl: SliderSizePreset,
}

impl SliderSizeScale {
    pub fn for_size(&self, size: Size) -> SliderSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_slider_size_scale() -> SliderSizeScale {
    SliderSizeScale {
        xs: SliderSizePreset {
            track_thickness: px(4.0),
            thumb_size: px(12.0),
        },
        sm: SliderSizePreset {
            track_thickness: px(5.0),
            thumb_size: px(14.0),
        },
        md: SliderSizePreset {
            track_thickness: px(6.0),
            thumb_size: px(16.0),
        },
        lg: SliderSizePreset {
            track_thickness: px(8.0),
            thumb_size: px(20.0),
        },
        xl: SliderSizePreset {
            track_thickness: px(10.0),
            thumb_size: px(24.0),
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TableSizePreset {
    pub font_size: Pixels,
    pub padding_x: Pixels,
    pub padding_y: Pixels,
    pub row_height: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TableSizeScale {
    pub xs: TableSizePreset,
    pub sm: TableSizePreset,
    pub md: TableSizePreset,
    pub lg: TableSizePreset,
    pub xl: TableSizePreset,
}

impl TableSizeScale {
    pub fn for_size(&self, size: Size) -> TableSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_table_size_scale() -> TableSizeScale {
    TableSizeScale {
        xs: TableSizePreset {
            font_size: px(12.0),
            padding_x: px(8.0),
            padding_y: px(4.0),
            row_height: px(24.0),
        },
        sm: TableSizePreset {
            font_size: px(13.0),
            padding_x: px(10.0),
            padding_y: px(6.0),
            row_height: px(28.0),
        },
        md: TableSizePreset {
            font_size: px(14.0),
            padding_x: px(12.0),
            padding_y: px(8.0),
            row_height: px(34.0),
        },
        lg: TableSizePreset {
            font_size: px(16.0),
            padding_x: px(14.0),
            padding_y: px(10.0),
            row_height: px(40.0),
        },
        xl: TableSizePreset {
            font_size: px(18.0),
            padding_x: px(16.0),
            padding_y: px(12.0),
            row_height: px(46.0),
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TreeSizePreset {
    pub label_size: Pixels,
    pub indent: Pixels,
    pub row_padding_y: Pixels,
    pub row_padding_right: Pixels,
    pub row_inner_gap: Pixels,
    pub toggle_size: Pixels,
    pub toggle_icon_size: Pixels,
    pub connector_stub_width: Pixels,
    pub child_line_margin: Pixels,
    pub child_line_padding: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TreeSizeScale {
    pub xs: TreeSizePreset,
    pub sm: TreeSizePreset,
    pub md: TreeSizePreset,
    pub lg: TreeSizePreset,
    pub xl: TreeSizePreset,
}

impl TreeSizeScale {
    pub fn for_size(&self, size: Size) -> TreeSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_tree_size_scale() -> TreeSizeScale {
    TreeSizeScale {
        xs: TreeSizePreset {
            label_size: px(12.0),
            indent: px(14.0),
            row_padding_y: px(3.0),
            row_padding_right: px(4.0),
            row_inner_gap: px(4.0),
            toggle_size: px(14.0),
            toggle_icon_size: px(11.0),
            connector_stub_width: px(6.0),
            child_line_margin: px(7.0),
            child_line_padding: px(8.0),
        },
        sm: TreeSizePreset {
            label_size: px(13.0),
            indent: px(16.0),
            row_padding_y: px(4.0),
            row_padding_right: px(5.0),
            row_inner_gap: px(4.0),
            toggle_size: px(15.0),
            toggle_icon_size: px(12.0),
            connector_stub_width: px(7.0),
            child_line_margin: px(8.0),
            child_line_padding: px(9.0),
        },
        md: TreeSizePreset {
            label_size: px(14.0),
            indent: px(18.0),
            row_padding_y: px(4.0),
            row_padding_right: px(6.0),
            row_inner_gap: px(4.0),
            toggle_size: px(16.0),
            toggle_icon_size: px(13.0),
            connector_stub_width: px(8.0),
            child_line_margin: px(8.0),
            child_line_padding: px(10.0),
        },
        lg: TreeSizePreset {
            label_size: px(16.0),
            indent: px(20.0),
            row_padding_y: px(5.0),
            row_padding_right: px(7.0),
            row_inner_gap: px(5.0),
            toggle_size: px(18.0),
            toggle_icon_size: px(14.0),
            connector_stub_width: px(9.0),
            child_line_margin: px(9.0),
            child_line_padding: px(11.0),
        },
        xl: TreeSizePreset {
            label_size: px(18.0),
            indent: px(22.0),
            row_padding_y: px(6.0),
            row_padding_right: px(8.0),
            row_inner_gap: px(6.0),
            toggle_size: px(20.0),
            toggle_icon_size: px(15.0),
            connector_stub_width: px(10.0),
            child_line_margin: px(10.0),
            child_line_padding: px(12.0),
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OverlayTokens {
    pub bg: Hsla,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoaderTokens {
    pub color: Hsla,
    pub label: Hsla,
    pub sizes: LoaderSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoaderSizePreset {
    pub dot_size: Pixels,
    pub ring_size: Pixels,
    pub bar_width: Pixels,
    pub bar_height_max: Pixels,
    pub cluster_gap: Pixels,
    pub label_size: Pixels,
    pub label_gap: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoaderSizeScale {
    pub xs: LoaderSizePreset,
    pub sm: LoaderSizePreset,
    pub md: LoaderSizePreset,
    pub lg: LoaderSizePreset,
    pub xl: LoaderSizePreset,
}

impl LoaderSizeScale {
    pub fn for_size(&self, size: Size) -> LoaderSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_loader_size_scale() -> LoaderSizeScale {
    LoaderSizeScale {
        xs: LoaderSizePreset {
            dot_size: px(5.0),
            ring_size: px(14.0),
            bar_width: px(3.0),
            bar_height_max: px(14.0),
            cluster_gap: px(4.0),
            label_size: px(12.0),
            label_gap: px(6.0),
        },
        sm: LoaderSizePreset {
            dot_size: px(6.0),
            ring_size: px(16.0),
            bar_width: px(4.0),
            bar_height_max: px(16.0),
            cluster_gap: px(4.0),
            label_size: px(13.0),
            label_gap: px(7.0),
        },
        md: LoaderSizePreset {
            dot_size: px(8.0),
            ring_size: px(20.0),
            bar_width: px(4.0),
            bar_height_max: px(18.0),
            cluster_gap: px(6.0),
            label_size: px(14.0),
            label_gap: px(8.0),
        },
        lg: LoaderSizePreset {
            dot_size: px(10.0),
            ring_size: px(24.0),
            bar_width: px(5.0),
            bar_height_max: px(20.0),
            cluster_gap: px(6.0),
            label_size: px(16.0),
            label_gap: px(9.0),
        },
        xl: LoaderSizePreset {
            dot_size: px(12.0),
            ring_size: px(28.0),
            bar_width: px(6.0),
            bar_height_max: px(22.0),
            cluster_gap: px(8.0),
            label_size: px(18.0),
            label_gap: px(10.0),
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadingOverlayTokens {
    pub bg: Hsla,
    pub loader_color: Hsla,
    pub label: Hsla,
    pub content_gap: Pixels,
    pub label_size: Pixels,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PopoverTokens {
    pub bg: Hsla,
    pub border: Hsla,
    pub title: Hsla,
    pub body: Hsla,
    pub padding: Pixels,
    pub gap: Pixels,
    pub radius: Pixels,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TooltipTokens {
    pub bg: Hsla,
    pub fg: Hsla,
    pub border: Hsla,
    pub text_size: Pixels,
    pub padding_x: Pixels,
    pub padding_y: Pixels,
    pub radius: Pixels,
    pub max_width: Pixels,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HoverCardTokens {
    pub bg: Hsla,
    pub border: Hsla,
    pub title: Hsla,
    pub body: Hsla,
    pub title_size: Pixels,
    pub title_weight: FontWeight,
    pub body_size: Pixels,
    pub min_width: Pixels,
    pub max_width: Pixels,
    pub padding: Pixels,
    pub gap: Pixels,
    pub radius: Pixels,
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
    pub label_size: Pixels,
    pub label_weight: FontWeight,
    pub description: Hsla,
    pub description_size: Pixels,
    pub error: Hsla,
    pub error_size: Pixels,
    pub label_block_gap: Pixels,
    pub label_row_gap: Pixels,
    pub slot_gap: Pixels,
    pub slot_min_width: Pixels,
    pub layout_gap_vertical: Pixels,
    pub layout_gap_horizontal: Pixels,
    pub horizontal_label_width: Pixels,
    pub icon_size: Pixels,
    pub option_size: Pixels,
    pub option_padding_x: Pixels,
    pub option_padding_y: Pixels,
    pub option_content_gap: Pixels,
    pub option_check_size: Pixels,
    pub dropdown_padding: Pixels,
    pub dropdown_gap: Pixels,
    pub dropdown_max_height: Pixels,
    pub dropdown_width_fallback: Pixels,
    pub dropdown_open_preferred_height: Pixels,
    pub tag_size: Pixels,
    pub tag_padding_x: Pixels,
    pub tag_padding_y: Pixels,
    pub tag_gap: Pixels,
    pub tag_max_width: Pixels,
    pub dropdown_anchor_offset: Pixels,
    pub sizes: FieldSizeScale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModalTokens {
    pub panel_bg: Hsla,
    pub panel_border: Hsla,
    pub overlay_bg: Hsla,
    pub title: Hsla,
    pub body: Hsla,
    pub title_size: Pixels,
    pub title_weight: FontWeight,
    pub body_size: Pixels,
    pub kind_icon_size: Pixels,
    pub kind_icon_gap: Pixels,
    pub panel_radius: Pixels,
    pub panel_padding: Pixels,
    pub header_margin_bottom: Pixels,
    pub body_margin_bottom: Pixels,
    pub actions_margin_top: Pixels,
    pub actions_gap: Pixels,
    pub close_size: Pixels,
    pub close_icon_size: Pixels,
    pub default_width: Pixels,
    pub min_width: Pixels,
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
    pub card_width: Pixels,
    pub card_padding: Pixels,
    pub row_gap: Pixels,
    pub content_gap: Pixels,
    pub icon_box_size: Pixels,
    pub icon_size: Pixels,
    pub close_button_size: Pixels,
    pub close_icon_size: Pixels,
    pub title_size: Pixels,
    pub body_size: Pixels,
    pub stack_gap: Pixels,
    pub edge_offset: Pixels,
    pub top_offset_extra: Pixels,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DividerTokens {
    pub line: Hsla,
    pub label: Hsla,
    pub label_size: Pixels,
    pub label_gap: Pixels,
    pub edge_span: Pixels,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScrollAreaTokens {
    pub bg: Hsla,
    pub border: Hsla,
    pub padding: InsetSizeScale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrawerTokens {
    pub panel_bg: Hsla,
    pub panel_border: Hsla,
    pub overlay_bg: Hsla,
    pub title: Hsla,
    pub body: Hsla,
    pub title_size: Pixels,
    pub title_weight: FontWeight,
    pub body_size: Pixels,
    pub panel_padding: Pixels,
    pub panel_radius: Pixels,
    pub header_margin_bottom: Pixels,
    pub close_size: Pixels,
    pub close_icon_size: Pixels,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppShellTokens {
    pub bg: Hsla,
    pub title_bar_bg: Hsla,
    pub sidebar_bg: Hsla,
    pub sidebar_overlay_bg: Hsla,
    pub content_bg: Hsla,
    pub bottom_panel_bg: Hsla,
    pub inspector_bg: Hsla,
    pub inspector_overlay_bg: Hsla,
    pub region_border: Hsla,
    pub title_bar_height: Pixels,
    pub sidebar_width: Pixels,
    pub sidebar_min_width: Pixels,
    pub inspector_width: Pixels,
    pub inspector_min_width: Pixels,
    pub bottom_panel_height: Pixels,
    pub bottom_panel_min_height: Pixels,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TitleBarTokens {
    pub bg: Hsla,
    pub border: Hsla,
    pub fg: Hsla,
    pub controls_bg: Hsla,
    pub height: Pixels,
    pub title_size: Pixels,
    pub title_weight: FontWeight,
    pub windows_button_width: Pixels,
    pub windows_icon_size: Pixels,
    pub linux_button_width: Pixels,
    pub linux_button_height: Pixels,
    pub linux_buttons_gap: Pixels,
    pub macos_controls_reserve: Pixels,
    pub title_padding_right: Pixels,
    pub title_max_width: Pixels,
    pub title_min_width: Pixels,
    pub platform_padding_left: Pixels,
    pub platform_padding_right: Pixels,
    pub controls_slot_gap: Pixels,
    pub control_button_radius: Pixels,
}

fn default_title_bar_height_px() -> Pixels {
    if cfg!(target_os = "macos") {
        px(30.0)
    } else if cfg!(target_os = "windows") {
        px(32.0)
    } else {
        px(34.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarTokens {
    pub bg: Hsla,
    pub border: Hsla,
    pub header_fg: Hsla,
    pub content_fg: Hsla,
    pub footer_fg: Hsla,
    pub inline_radius: Pixels,
    pub overlay_radius: Pixels,
    pub min_width: Pixels,
    pub section_padding: Pixels,
    pub footer_size: Pixels,
    pub scroll_padding: Size,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarkdownTokens {
    pub paragraph: Hsla,
    pub quote_bg: Hsla,
    pub quote_border: Hsla,
    pub quote_fg: Hsla,
    pub code_bg: Hsla,
    pub code_border: Hsla,
    pub code_fg: Hsla,
    pub code_lang_fg: Hsla,
    pub list_marker: Hsla,
    pub rule: Hsla,
    pub gap_regular: Pixels,
    pub gap_compact: Pixels,
    pub paragraph_size: Pixels,
    pub quote_size: Pixels,
    pub code_size: Pixels,
    pub code_lang_size: Pixels,
    pub list_size: Pixels,
    pub quote_padding_x: Pixels,
    pub quote_padding_y: Pixels,
    pub quote_radius: Pixels,
    pub code_padding: Pixels,
    pub code_radius: Pixels,
    pub code_gap: Pixels,
    pub list_gap: Pixels,
    pub list_item_gap: Pixels,
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
    pub sizes: TextSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextSizePreset {
    pub font_size: Pixels,
    pub line_height: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextSizeScale {
    pub xs: TextSizePreset,
    pub sm: TextSizePreset,
    pub md: TextSizePreset,
    pub lg: TextSizePreset,
    pub xl: TextSizePreset,
}

impl TextSizeScale {
    pub fn for_size(&self, size: Size) -> TextSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_text_size_scale() -> TextSizeScale {
    TextSizeScale {
        xs: TextSizePreset {
            font_size: px(12.0),
            line_height: px(14.0),
        },
        sm: TextSizePreset {
            font_size: px(13.0),
            line_height: px(16.0),
        },
        md: TextSizePreset {
            font_size: px(14.0),
            line_height: px(18.0),
        },
        lg: TextSizePreset {
            font_size: px(16.0),
            line_height: px(22.0),
        },
        xl: TextSizePreset {
            font_size: px(18.0),
            line_height: px(26.0),
        },
    }
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
    pub padding: InsetSizeScale,
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
    pub sizes: ActionIconSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionIconSizePreset {
    pub box_size: Pixels,
    pub icon_size: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionIconSizeScale {
    pub xs: ActionIconSizePreset,
    pub sm: ActionIconSizePreset,
    pub md: ActionIconSizePreset,
    pub lg: ActionIconSizePreset,
    pub xl: ActionIconSizePreset,
}

impl ActionIconSizeScale {
    pub fn for_size(&self, size: Size) -> ActionIconSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_action_icon_size_scale() -> ActionIconSizeScale {
    ActionIconSizeScale {
        xs: ActionIconSizePreset {
            box_size: px(22.0),
            icon_size: px(12.0),
        },
        sm: ActionIconSizePreset {
            box_size: px(26.0),
            icon_size: px(14.0),
        },
        md: ActionIconSizePreset {
            box_size: px(30.0),
            icon_size: px(16.0),
        },
        lg: ActionIconSizePreset {
            box_size: px(36.0),
            icon_size: px(18.0),
        },
        xl: ActionIconSizePreset {
            box_size: px(42.0),
            icon_size: px(20.0),
        },
    }
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
    pub track_padding: Pixels,
    pub item_gap: Pixels,
    pub sizes: SegmentedControlSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentedControlSizePreset {
    pub font_size: Pixels,
    pub line_height: Pixels,
    pub padding_x: Pixels,
    pub padding_y: Pixels,
    pub indicator_inset: Pixels,
    pub divider_height: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SegmentedControlSizeScale {
    pub xs: SegmentedControlSizePreset,
    pub sm: SegmentedControlSizePreset,
    pub md: SegmentedControlSizePreset,
    pub lg: SegmentedControlSizePreset,
    pub xl: SegmentedControlSizePreset,
}

impl SegmentedControlSizeScale {
    pub fn for_size(&self, size: Size) -> SegmentedControlSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_segmented_control_size_scale() -> SegmentedControlSizeScale {
    SegmentedControlSizeScale {
        xs: SegmentedControlSizePreset {
            font_size: px(12.0),
            line_height: px(16.0),
            padding_x: px(8.0),
            padding_y: px(4.0),
            indicator_inset: px(0.5),
            divider_height: px(12.0),
        },
        sm: SegmentedControlSizePreset {
            font_size: px(13.0),
            line_height: px(18.0),
            padding_x: px(10.0),
            padding_y: px(4.0),
            indicator_inset: px(1.0),
            divider_height: px(14.0),
        },
        md: SegmentedControlSizePreset {
            font_size: px(14.0),
            line_height: px(20.0),
            padding_x: px(12.0),
            padding_y: px(6.0),
            indicator_inset: px(1.0),
            divider_height: px(16.0),
        },
        lg: SegmentedControlSizePreset {
            font_size: px(16.0),
            line_height: px(22.0),
            padding_x: px(14.0),
            padding_y: px(8.0),
            indicator_inset: px(1.5),
            divider_height: px(18.0),
        },
        xl: SegmentedControlSizePreset {
            font_size: px(18.0),
            line_height: px(24.0),
            padding_x: px(16.0),
            padding_y: px(10.0),
            indicator_inset: px(1.5),
            divider_height: px(20.0),
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextareaTokens {
    pub bg: Hsla,
    pub fg: Hsla,
    pub caret: Hsla,
    pub selection_bg: Hsla,
    pub placeholder: Hsla,
    pub border: Hsla,
    pub border_focus: Hsla,
    pub border_error: Hsla,
    pub label: Hsla,
    pub label_size: Pixels,
    pub label_weight: FontWeight,
    pub description: Hsla,
    pub description_size: Pixels,
    pub error: Hsla,
    pub error_size: Pixels,
    pub label_block_gap: Pixels,
    pub label_row_gap: Pixels,
    pub layout_gap_vertical: Pixels,
    pub layout_gap_horizontal: Pixels,
    pub horizontal_label_width: Pixels,
    pub content_width_fallback: Pixels,
    pub sizes: FieldSizeScale,
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
    pub label_size: Pixels,
    pub label_weight: FontWeight,
    pub description: Hsla,
    pub description_size: Pixels,
    pub error: Hsla,
    pub error_size: Pixels,
    pub controls_width: Pixels,
    pub controls_height: Pixels,
    pub controls_icon_size: Pixels,
    pub controls_gap: Pixels,
    pub sizes: FieldSizeScale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RangeSliderTokens {
    pub track_bg: Hsla,
    pub range_bg: Hsla,
    pub thumb_bg: Hsla,
    pub thumb_border: Hsla,
    pub label: Hsla,
    pub value: Hsla,
    pub label_size: Pixels,
    pub value_size: Pixels,
    pub header_gap_vertical: Pixels,
    pub header_gap_horizontal: Pixels,
    pub default_width: Pixels,
    pub min_width: Pixels,
    pub sizes: SliderSizeScale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RatingTokens {
    pub active: Hsla,
    pub inactive: Hsla,
    pub sizes: RatingSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RatingSizePreset {
    pub icon_size: Pixels,
    pub gap: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RatingSizeScale {
    pub xs: RatingSizePreset,
    pub sm: RatingSizePreset,
    pub md: RatingSizePreset,
    pub lg: RatingSizePreset,
    pub xl: RatingSizePreset,
}

impl RatingSizeScale {
    pub fn for_size(&self, size: Size) -> RatingSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_rating_size_scale() -> RatingSizeScale {
    RatingSizeScale {
        xs: RatingSizePreset {
            icon_size: px(14.0),
            gap: px(4.0),
        },
        sm: RatingSizePreset {
            icon_size: px(16.0),
            gap: px(4.0),
        },
        md: RatingSizePreset {
            icon_size: px(18.0),
            gap: px(4.0),
        },
        lg: RatingSizePreset {
            icon_size: px(22.0),
            gap: px(6.0),
        },
        xl: RatingSizePreset {
            icon_size: px(26.0),
            gap: px(6.0),
        },
    }
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
    pub root_gap: Pixels,
    pub list_gap: Pixels,
    pub list_padding: Pixels,
    pub panel_padding: Pixels,
    pub sizes: TabsSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TabsSizePreset {
    pub font_size: Pixels,
    pub line_height: Pixels,
    pub padding_x: Pixels,
    pub padding_y: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TabsSizeScale {
    pub xs: TabsSizePreset,
    pub sm: TabsSizePreset,
    pub md: TabsSizePreset,
    pub lg: TabsSizePreset,
    pub xl: TabsSizePreset,
}

impl TabsSizeScale {
    pub fn for_size(&self, size: Size) -> TabsSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_tabs_size_scale() -> TabsSizeScale {
    TabsSizeScale {
        xs: TabsSizePreset {
            font_size: px(12.0),
            line_height: px(16.0),
            padding_x: px(8.0),
            padding_y: px(2.0),
        },
        sm: TabsSizePreset {
            font_size: px(13.0),
            line_height: px(18.0),
            padding_x: px(10.0),
            padding_y: px(4.0),
        },
        md: TabsSizePreset {
            font_size: px(14.0),
            line_height: px(20.0),
            padding_x: px(12.0),
            padding_y: px(6.0),
        },
        lg: TabsSizePreset {
            font_size: px(16.0),
            line_height: px(22.0),
            padding_x: px(14.0),
            padding_y: px(8.0),
        },
        xl: TabsSizePreset {
            font_size: px(18.0),
            line_height: px(24.0),
            padding_x: px(16.0),
            padding_y: px(10.0),
        },
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaginationSizePreset {
    pub font_size: Pixels,
    pub padding_x: Pixels,
    pub padding_y: Pixels,
    pub min_width: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaginationSizeScale {
    pub xs: PaginationSizePreset,
    pub sm: PaginationSizePreset,
    pub md: PaginationSizePreset,
    pub lg: PaginationSizePreset,
    pub xl: PaginationSizePreset,
}

impl PaginationSizeScale {
    pub fn for_size(&self, size: Size) -> PaginationSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_pagination_size_scale() -> PaginationSizeScale {
    PaginationSizeScale {
        xs: PaginationSizePreset {
            font_size: px(12.0),
            padding_x: px(6.0),
            padding_y: px(3.0),
            min_width: px(24.0),
        },
        sm: PaginationSizePreset {
            font_size: px(13.0),
            padding_x: px(8.0),
            padding_y: px(4.0),
            min_width: px(28.0),
        },
        md: PaginationSizePreset {
            font_size: px(14.0),
            padding_x: px(10.0),
            padding_y: px(4.0),
            min_width: px(32.0),
        },
        lg: PaginationSizePreset {
            font_size: px(16.0),
            padding_x: px(12.0),
            padding_y: px(6.0),
            min_width: px(36.0),
        },
        xl: PaginationSizePreset {
            font_size: px(18.0),
            padding_x: px(14.0),
            padding_y: px(8.0),
            min_width: px(40.0),
        },
    }
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
    pub root_gap: Pixels,
    pub sizes: PaginationSizeScale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreadcrumbsTokens {
    pub item_fg: Hsla,
    pub item_current_fg: Hsla,
    pub separator: Hsla,
    pub item_hover_bg: Hsla,
    pub root_gap: Pixels,
    pub sizes: BreadcrumbsSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BreadcrumbsSizePreset {
    pub font_size: Pixels,
    pub item_padding_x: Pixels,
    pub item_padding_y: Pixels,
    pub item_radius: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BreadcrumbsSizeScale {
    pub xs: BreadcrumbsSizePreset,
    pub sm: BreadcrumbsSizePreset,
    pub md: BreadcrumbsSizePreset,
    pub lg: BreadcrumbsSizePreset,
    pub xl: BreadcrumbsSizePreset,
}

impl BreadcrumbsSizeScale {
    pub fn for_size(&self, size: Size) -> BreadcrumbsSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_breadcrumbs_size_scale() -> BreadcrumbsSizeScale {
    BreadcrumbsSizeScale {
        xs: BreadcrumbsSizePreset {
            font_size: px(12.0),
            item_padding_x: px(4.0),
            item_padding_y: px(1.0),
            item_radius: px(4.0),
        },
        sm: BreadcrumbsSizePreset {
            font_size: px(13.0),
            item_padding_x: px(4.0),
            item_padding_y: px(2.0),
            item_radius: px(4.0),
        },
        md: BreadcrumbsSizePreset {
            font_size: px(14.0),
            item_padding_x: px(5.0),
            item_padding_y: px(2.0),
            item_radius: px(5.0),
        },
        lg: BreadcrumbsSizePreset {
            font_size: px(16.0),
            item_padding_x: px(6.0),
            item_padding_y: px(3.0),
            item_radius: px(6.0),
        },
        xl: BreadcrumbsSizePreset {
            font_size: px(18.0),
            item_padding_x: px(7.0),
            item_padding_y: px(4.0),
            item_radius: px(7.0),
        },
    }
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
    pub caption_size: Pixels,
    pub row_gap: Pixels,
    pub pagination_summary_size: Pixels,
    pub page_chip_size: Pixels,
    pub page_chip_padding_x: Pixels,
    pub page_chip_padding_y: Pixels,
    pub page_chip_radius: Pixels,
    pub page_chip_gap: Pixels,
    pub pagination_items_gap: Pixels,
    pub pagination_padding_x: Pixels,
    pub pagination_padding_y: Pixels,
    pub pagination_gap: Pixels,
    pub virtualization_padding: Pixels,
    pub min_viewport_height: Pixels,
    pub sizes: TableSizeScale,
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
    pub root_gap: Pixels,
    pub steps_gap_vertical: Pixels,
    pub text_gap: Pixels,
    pub panel_margin_top: Pixels,
    pub sizes: StepperSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StepperSizePreset {
    pub indicator_size: Pixels,
    pub connector_thickness: Pixels,
    pub connector_span: Pixels,
    pub label_size: Pixels,
    pub description_size: Pixels,
    pub item_padding: Pixels,
    pub item_gap_vertical: Pixels,
    pub item_gap_horizontal: Pixels,
    pub panel_padding: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StepperSizeScale {
    pub xs: StepperSizePreset,
    pub sm: StepperSizePreset,
    pub md: StepperSizePreset,
    pub lg: StepperSizePreset,
    pub xl: StepperSizePreset,
}

impl StepperSizeScale {
    pub fn for_size(&self, size: Size) -> StepperSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_stepper_size_scale() -> StepperSizeScale {
    StepperSizeScale {
        xs: StepperSizePreset {
            indicator_size: px(18.0),
            connector_thickness: px(1.0),
            connector_span: px(20.0),
            label_size: px(12.0),
            description_size: px(11.0),
            item_padding: px(4.0),
            item_gap_vertical: px(4.0),
            item_gap_horizontal: px(6.0),
            panel_padding: px(10.0),
        },
        sm: StepperSizePreset {
            indicator_size: px(20.0),
            connector_thickness: px(1.0),
            connector_span: px(24.0),
            label_size: px(13.0),
            description_size: px(12.0),
            item_padding: px(5.0),
            item_gap_vertical: px(4.0),
            item_gap_horizontal: px(6.0),
            panel_padding: px(11.0),
        },
        md: StepperSizePreset {
            indicator_size: px(24.0),
            connector_thickness: px(2.0),
            connector_span: px(28.0),
            label_size: px(14.0),
            description_size: px(13.0),
            item_padding: px(6.0),
            item_gap_vertical: px(6.0),
            item_gap_horizontal: px(8.0),
            panel_padding: px(12.0),
        },
        lg: StepperSizePreset {
            indicator_size: px(28.0),
            connector_thickness: px(3.0),
            connector_span: px(32.0),
            label_size: px(16.0),
            description_size: px(14.0),
            item_padding: px(7.0),
            item_gap_vertical: px(8.0),
            item_gap_horizontal: px(10.0),
            panel_padding: px(14.0),
        },
        xl: StepperSizePreset {
            indicator_size: px(32.0),
            connector_thickness: px(3.0),
            connector_span: px(36.0),
            label_size: px(18.0),
            description_size: px(15.0),
            item_padding: px(8.0),
            item_gap_vertical: px(10.0),
            item_gap_horizontal: px(12.0),
            panel_padding: px(16.0),
        },
    }
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
    pub root_gap: Pixels,
    pub row_gap: Pixels,
    pub content_gap: Pixels,
    pub card_margin_top: Pixels,
    pub row_padding_y: Pixels,
    pub line_min_height: Pixels,
    pub line_extra_height: Pixels,
    pub sizes: TimelineSizeScale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimelineSizePreset {
    pub bullet_size: Pixels,
    pub line_width: Pixels,
    pub title_size: Pixels,
    pub body_size: Pixels,
    pub card_padding: Pixels,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimelineSizeScale {
    pub xs: TimelineSizePreset,
    pub sm: TimelineSizePreset,
    pub md: TimelineSizePreset,
    pub lg: TimelineSizePreset,
    pub xl: TimelineSizePreset,
}

impl TimelineSizeScale {
    pub fn for_size(&self, size: Size) -> TimelineSizePreset {
        match size {
            Size::Xs => self.xs,
            Size::Sm => self.sm,
            Size::Md => self.md,
            Size::Lg => self.lg,
            Size::Xl => self.xl,
        }
    }
}

fn default_timeline_size_scale() -> TimelineSizeScale {
    TimelineSizeScale {
        xs: TimelineSizePreset {
            bullet_size: px(14.0),
            line_width: px(1.0),
            title_size: px(12.0),
            body_size: px(11.0),
            card_padding: px(8.0),
        },
        sm: TimelineSizePreset {
            bullet_size: px(16.0),
            line_width: px(1.0),
            title_size: px(13.0),
            body_size: px(12.0),
            card_padding: px(8.0),
        },
        md: TimelineSizePreset {
            bullet_size: px(18.0),
            line_width: px(2.0),
            title_size: px(14.0),
            body_size: px(13.0),
            card_padding: px(10.0),
        },
        lg: TimelineSizePreset {
            bullet_size: px(22.0),
            line_width: px(3.0),
            title_size: px(16.0),
            body_size: px(14.0),
            card_padding: px(12.0),
        },
        xl: TimelineSizePreset {
            bullet_size: px(26.0),
            line_width: px(3.0),
            title_size: px(18.0),
            body_size: px(15.0),
            card_padding: px(14.0),
        },
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeTokens {
    pub row_fg: Hsla,
    pub row_selected_fg: Hsla,
    pub row_selected_bg: Hsla,
    pub row_hover_bg: Hsla,
    pub row_disabled_fg: Hsla,
    pub line: Hsla,
    pub root_gap: Pixels,
    pub children_gap: Pixels,
    pub sizes: TreeSizeScale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutTokens {
    pub gap: GapSizeScale,
    pub space: GapSizeScale,
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
    pub loader: LoaderTokens,
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
    pub markdown: MarkdownTokens,
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
    pub layout: LayoutTokens,
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
                    sizes: default_button_size_scale(),
                },
                input: InputTokens {
                    bg: white(),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    caret: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    selection_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[1 as usize])
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
                    label_size: px(14.0),
                    label_weight: FontWeight::MEDIUM,
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    description_size: px(13.0),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    error_size: px(13.0),
                    label_block_gap: px(4.0),
                    label_row_gap: px(4.0),
                    slot_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    slot_gap: px(8.0),
                    slot_min_width: px(16.0),
                    layout_gap_vertical: px(8.0),
                    layout_gap_horizontal: px(12.0),
                    horizontal_label_width: px(168.0),
                    pin_cells_gap: px(8.0),
                    pin_error_gap: px(4.0),
                    sizes: default_field_size_scale(),
                },
                radio: RadioTokens {
                    control_bg: white(),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border_hover: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
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
                    label_description_gap: px(2.0),
                    group_gap_horizontal: px(12.0),
                    group_gap_vertical: px(8.0),
                    sizes: default_choice_control_size_scale(),
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
                    border_hover: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
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
                    label_description_gap: px(2.0),
                    group_gap_horizontal: px(12.0),
                    group_gap_vertical: px(8.0),
                    sizes: default_choice_control_size_scale(),
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
                    track_hover_border: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    track_focus_border: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[6 as usize],
                    )
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
                    label_description_gap: px(2.0),
                    sizes: default_switch_size_scale(),
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
                    border_hover: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    content_gap: px(4.0),
                    indicator_size: px(12.0),
                    group_gap_horizontal: px(8.0),
                    group_gap_vertical: px(8.0),
                    sizes: default_button_size_scale(),
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
                    sizes: default_badge_size_scale(),
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
                    stack_gap: px(8.0),
                    header_gap: px(8.0),
                    label_stack_gap: px(2.0),
                    panel_gap: px(4.0),
                    sizes: default_accordion_size_scale(),
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
                    item_gap: px(8.0),
                    item_padding_x: px(10.0),
                    item_padding_y: px(8.0),
                    item_size: px(14.0),
                    item_icon_size: px(14.0),
                    item_radius: px(6.0),
                    dropdown_padding: px(6.0),
                    dropdown_gap: px(4.0),
                    dropdown_radius: px(8.0),
                    dropdown_width_fallback: px(220.0),
                    dropdown_min_width: px(180.0),
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
                    default_width: px(260.0),
                    min_width: px(80.0),
                    root_gap: px(6.0),
                    sizes: default_progress_size_scale(),
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
                    label_size: px(14.0),
                    value_size: px(14.0),
                    header_gap_vertical: px(6.0),
                    header_gap_horizontal: px(8.0),
                    default_width: px(260.0),
                    min_width: px(120.0),
                    sizes: default_slider_size_scale(),
                },
                overlay: OverlayTokens {
                    bg: (Rgba::try_from("#000000E6")
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                loader: LoaderTokens {
                    color: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    sizes: default_loader_size_scale(),
                },
                loading_overlay: LoadingOverlayTokens {
                    bg: (Rgba::try_from("#000000E6")
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    loader_color: (Rgba::try_from(PaletteCatalog::scale(primary)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: white(),
                    content_gap: px(8.0),
                    label_size: px(13.0),
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
                    padding: px(12.0),
                    gap: px(8.0),
                    radius: px(8.0),
                },
                tooltip: TooltipTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fg: white(),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    text_size: px(12.0),
                    padding_x: px(8.0),
                    padding_y: px(5.0),
                    radius: px(8.0),
                    max_width: px(240.0),
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
                    title_size: px(15.0),
                    title_weight: FontWeight::SEMIBOLD,
                    body_size: px(14.0),
                    min_width: px(120.0),
                    max_width: px(360.0),
                    padding: px(12.0),
                    gap: px(6.0),
                    radius: px(8.0),
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
                    label_size: px(14.0),
                    label_weight: FontWeight::MEDIUM,
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    description_size: px(13.0),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    error_size: px(13.0),
                    label_block_gap: px(4.0),
                    label_row_gap: px(4.0),
                    slot_gap: px(8.0),
                    slot_min_width: px(16.0),
                    layout_gap_vertical: px(8.0),
                    layout_gap_horizontal: px(12.0),
                    horizontal_label_width: px(168.0),
                    icon_size: px(14.0),
                    option_size: px(14.0),
                    option_padding_x: px(10.0),
                    option_padding_y: px(8.0),
                    option_content_gap: px(8.0),
                    option_check_size: px(12.0),
                    dropdown_padding: px(6.0),
                    dropdown_gap: px(4.0),
                    dropdown_max_height: px(280.0),
                    dropdown_width_fallback: px(220.0),
                    dropdown_open_preferred_height: px(260.0),
                    tag_size: px(12.0),
                    tag_padding_x: px(8.0),
                    tag_padding_y: px(3.0),
                    tag_gap: px(4.0),
                    tag_max_width: px(120.0),
                    dropdown_anchor_offset: px(2.0),
                    sizes: default_field_size_scale(),
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
                    title_size: px(18.0),
                    title_weight: FontWeight::SEMIBOLD,
                    body_size: px(14.0),
                    kind_icon_size: px(16.0),
                    kind_icon_gap: px(8.0),
                    panel_radius: px(12.0),
                    panel_padding: px(16.0),
                    header_margin_bottom: px(8.0),
                    body_margin_bottom: px(8.0),
                    actions_margin_top: px(12.0),
                    actions_gap: px(8.0),
                    close_size: px(26.0),
                    close_icon_size: px(14.0),
                    default_width: px(560.0),
                    min_width: px(240.0),
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
                    card_width: px(360.0),
                    card_padding: px(12.0),
                    row_gap: px(8.0),
                    content_gap: px(4.0),
                    icon_box_size: px(24.0),
                    icon_size: px(17.0),
                    close_button_size: px(24.0),
                    close_icon_size: px(13.0),
                    title_size: px(14.0),
                    body_size: px(13.0),
                    stack_gap: px(8.0),
                    edge_offset: px(16.0),
                    top_offset_extra: px(8.0),
                },
                divider: DividerTokens {
                    line: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label_size: px(12.0),
                    label_gap: px(8.0),
                    edge_span: px(16.0),
                },
                scroll_area: ScrollAreaTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    padding: default_inset_size_scale(),
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
                    title_size: px(16.0),
                    title_weight: FontWeight::SEMIBOLD,
                    body_size: px(14.0),
                    panel_padding: px(16.0),
                    panel_radius: px(0.0),
                    header_margin_bottom: px(8.0),
                    close_size: px(28.0),
                    close_icon_size: px(14.0),
                },
                app_shell: AppShellTokens {
                    bg: white(),
                    title_bar_bg: white(),
                    sidebar_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    sidebar_overlay_bg: white(),
                    content_bg: white(),
                    bottom_panel_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    inspector_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    inspector_overlay_bg: white(),
                    region_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    title_bar_height: px(44.0),
                    sidebar_width: px(260.0),
                    sidebar_min_width: px(120.0),
                    inspector_width: px(320.0),
                    inspector_min_width: px(120.0),
                    bottom_panel_height: px(180.0),
                    bottom_panel_min_height: px(80.0),
                },
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
                    height: default_title_bar_height_px(),
                    title_size: px(14.0),
                    title_weight: FontWeight::MEDIUM,
                    windows_button_width: px(45.0),
                    windows_icon_size: px(10.0),
                    linux_button_width: px(28.0),
                    linux_button_height: px(24.0),
                    linux_buttons_gap: px(6.0),
                    macos_controls_reserve: px(72.0),
                    title_padding_right: px(12.0),
                    title_max_width: px(320.0),
                    title_min_width: px(72.0),
                    platform_padding_left: px(12.0),
                    platform_padding_right: px(12.0),
                    controls_slot_gap: px(10.0),
                    control_button_radius: px(6.0),
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
                    inline_radius: px(10.0),
                    overlay_radius: px(18.0),
                    min_width: px(120.0),
                    section_padding: px(12.0),
                    footer_size: px(14.0),
                    scroll_padding: Size::Md,
                },
                markdown: MarkdownTokens {
                    paragraph: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    quote_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    quote_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    quote_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    code_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    code_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    code_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    code_lang_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    list_marker: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[6 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    rule: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    gap_regular: px(8.0),
                    gap_compact: px(6.0),
                    paragraph_size: px(14.0),
                    quote_size: px(14.0),
                    code_size: px(13.0),
                    code_lang_size: px(12.0),
                    list_size: px(14.0),
                    quote_padding_x: px(12.0),
                    quote_padding_y: px(8.0),
                    quote_radius: px(8.0),
                    code_padding: px(10.0),
                    code_radius: px(8.0),
                    code_gap: px(6.0),
                    list_gap: px(6.0),
                    list_item_gap: px(8.0),
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
                    sizes: default_text_size_scale(),
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
                    padding: default_inset_size_scale(),
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
                    sizes: default_action_icon_size_scale(),
                },
                segmented_control: SegmentedControlTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
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
                    track_padding: px(2.0),
                    item_gap: px(0.0),
                    sizes: default_segmented_control_size_scale(),
                },
                textarea: TextareaTokens {
                    bg: white(),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    caret: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    selection_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[1 as usize])
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
                    label_size: px(14.0),
                    label_weight: FontWeight::MEDIUM,
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    description_size: px(13.0),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    error_size: px(13.0),
                    label_block_gap: px(4.0),
                    label_row_gap: px(4.0),
                    layout_gap_vertical: px(8.0),
                    layout_gap_horizontal: px(12.0),
                    horizontal_label_width: px(168.0),
                    content_width_fallback: px(240.0),
                    sizes: default_field_size_scale(),
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
                    label_size: px(14.0),
                    label_weight: FontWeight::MEDIUM,
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[7 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    description_size: px(13.0),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    error_size: px(13.0),
                    controls_width: px(18.0),
                    controls_height: px(12.0),
                    controls_icon_size: px(12.0),
                    controls_gap: px(8.0),
                    sizes: default_field_size_scale(),
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
                    label_size: px(14.0),
                    value_size: px(14.0),
                    header_gap_vertical: px(6.0),
                    header_gap_horizontal: px(8.0),
                    default_width: px(260.0),
                    min_width: px(140.0),
                    sizes: default_slider_size_scale(),
                },
                rating: RatingTokens {
                    active: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Yellow)[6 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    inactive: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    sizes: default_rating_size_scale(),
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
                    root_gap: px(8.0),
                    list_gap: px(2.0),
                    list_padding: px(2.0),
                    panel_padding: px(16.0),
                    sizes: default_tabs_size_scale(),
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
                    root_gap: px(4.0),
                    sizes: default_pagination_size_scale(),
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
                    root_gap: px(4.0),
                    sizes: default_breadcrumbs_size_scale(),
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
                    caption_size: px(13.0),
                    row_gap: px(0.0),
                    pagination_summary_size: px(13.0),
                    page_chip_size: px(12.0),
                    page_chip_padding_x: px(8.0),
                    page_chip_padding_y: px(4.0),
                    page_chip_radius: px(6.0),
                    page_chip_gap: px(4.0),
                    pagination_items_gap: px(8.0),
                    pagination_padding_x: px(12.0),
                    pagination_padding_y: px(8.0),
                    pagination_gap: px(8.0),
                    virtualization_padding: px(4.0),
                    min_viewport_height: px(80.0),
                    sizes: default_table_size_scale(),
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
                    root_gap: px(6.0),
                    steps_gap_vertical: px(6.0),
                    text_gap: px(2.0),
                    panel_margin_top: px(8.0),
                    sizes: default_stepper_size_scale(),
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
                    root_gap: px(0.0),
                    row_gap: px(8.0),
                    content_gap: px(4.0),
                    card_margin_top: px(4.0),
                    row_padding_y: px(0.0),
                    line_min_height: px(24.0),
                    line_extra_height: px(8.0),
                    sizes: default_timeline_size_scale(),
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
                    root_gap: px(2.0),
                    children_gap: px(0.0),
                    sizes: default_tree_size_scale(),
                },
                layout: LayoutTokens {
                    gap: default_layout_gap_scale(),
                    space: default_layout_space_scale(),
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
                    sizes: default_button_size_scale(),
                },
                input: InputTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    caret: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    selection_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[8 as usize])
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
                    label_size: px(14.0),
                    label_weight: FontWeight::MEDIUM,
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    description_size: px(13.0),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    error_size: px(13.0),
                    label_block_gap: px(4.0),
                    label_row_gap: px(4.0),
                    slot_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    slot_gap: px(8.0),
                    slot_min_width: px(16.0),
                    layout_gap_vertical: px(8.0),
                    layout_gap_horizontal: px(12.0),
                    horizontal_label_width: px(168.0),
                    pin_cells_gap: px(8.0),
                    pin_error_gap: px(4.0),
                    sizes: default_field_size_scale(),
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
                    border_hover: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
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
                    label_description_gap: px(2.0),
                    group_gap_horizontal: px(12.0),
                    group_gap_vertical: px(8.0),
                    sizes: default_choice_control_size_scale(),
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
                    border_hover: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
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
                    label_description_gap: px(2.0),
                    group_gap_horizontal: px(12.0),
                    group_gap_vertical: px(8.0),
                    sizes: default_choice_control_size_scale(),
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
                    track_hover_border: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    track_focus_border: (Rgba::try_from(
                        PaletteCatalog::scale(primary)[5 as usize],
                    )
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
                    label_description_gap: px(2.0),
                    sizes: default_switch_size_scale(),
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
                    border_hover: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[3 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    border_focus: (Rgba::try_from(PaletteCatalog::scale(primary)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    content_gap: px(4.0),
                    indicator_size: px(12.0),
                    group_gap_horizontal: px(8.0),
                    group_gap_vertical: px(8.0),
                    sizes: default_button_size_scale(),
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
                    sizes: default_badge_size_scale(),
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
                    stack_gap: px(8.0),
                    header_gap: px(8.0),
                    label_stack_gap: px(2.0),
                    panel_gap: px(4.0),
                    sizes: default_accordion_size_scale(),
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
                    item_gap: px(8.0),
                    item_padding_x: px(10.0),
                    item_padding_y: px(8.0),
                    item_size: px(14.0),
                    item_icon_size: px(14.0),
                    item_radius: px(6.0),
                    dropdown_padding: px(6.0),
                    dropdown_gap: px(4.0),
                    dropdown_radius: px(8.0),
                    dropdown_width_fallback: px(220.0),
                    dropdown_min_width: px(180.0),
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
                    default_width: px(260.0),
                    min_width: px(80.0),
                    root_gap: px(6.0),
                    sizes: default_progress_size_scale(),
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
                    label_size: px(14.0),
                    value_size: px(14.0),
                    header_gap_vertical: px(6.0),
                    header_gap_horizontal: px(8.0),
                    default_width: px(260.0),
                    min_width: px(120.0),
                    sizes: default_slider_size_scale(),
                },
                overlay: OverlayTokens {
                    bg: (Rgba::try_from("#000000E6")
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                },
                loader: LoaderTokens {
                    color: (Rgba::try_from(PaletteCatalog::scale(primary)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    sizes: default_loader_size_scale(),
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
                    content_gap: px(8.0),
                    label_size: px(13.0),
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
                    padding: px(12.0),
                    gap: px(8.0),
                    radius: px(8.0),
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
                    text_size: px(12.0),
                    padding_x: px(8.0),
                    padding_y: px(5.0),
                    radius: px(8.0),
                    max_width: px(240.0),
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
                    title_size: px(15.0),
                    title_weight: FontWeight::SEMIBOLD,
                    body_size: px(14.0),
                    min_width: px(120.0),
                    max_width: px(360.0),
                    padding: px(12.0),
                    gap: px(6.0),
                    radius: px(8.0),
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
                    label_size: px(14.0),
                    label_weight: FontWeight::MEDIUM,
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    description_size: px(13.0),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    error_size: px(13.0),
                    label_block_gap: px(4.0),
                    label_row_gap: px(4.0),
                    slot_gap: px(8.0),
                    slot_min_width: px(16.0),
                    layout_gap_vertical: px(8.0),
                    layout_gap_horizontal: px(12.0),
                    horizontal_label_width: px(168.0),
                    icon_size: px(14.0),
                    option_size: px(14.0),
                    option_padding_x: px(10.0),
                    option_padding_y: px(8.0),
                    option_content_gap: px(8.0),
                    option_check_size: px(12.0),
                    dropdown_padding: px(6.0),
                    dropdown_gap: px(4.0),
                    dropdown_max_height: px(280.0),
                    dropdown_width_fallback: px(220.0),
                    dropdown_open_preferred_height: px(260.0),
                    tag_size: px(12.0),
                    tag_padding_x: px(8.0),
                    tag_padding_y: px(3.0),
                    tag_gap: px(4.0),
                    tag_max_width: px(120.0),
                    dropdown_anchor_offset: px(2.0),
                    sizes: default_field_size_scale(),
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
                    title_size: px(18.0),
                    title_weight: FontWeight::SEMIBOLD,
                    body_size: px(14.0),
                    kind_icon_size: px(16.0),
                    kind_icon_gap: px(8.0),
                    panel_radius: px(12.0),
                    panel_padding: px(16.0),
                    header_margin_bottom: px(8.0),
                    body_margin_bottom: px(8.0),
                    actions_margin_top: px(12.0),
                    actions_gap: px(8.0),
                    close_size: px(26.0),
                    close_icon_size: px(14.0),
                    default_width: px(560.0),
                    min_width: px(240.0),
                },
                toast: ToastTokens {
                    info_bg: resolve_palette_hsla(PaletteKey::Blue, 4).opacity(0.15),
                    info_fg: resolve_palette_hsla(PaletteKey::Blue, 4),
                    success_bg: resolve_palette_hsla(PaletteKey::Green, 4).opacity(0.15),
                    success_fg: resolve_palette_hsla(PaletteKey::Green, 4),
                    warning_bg: resolve_palette_hsla(PaletteKey::Yellow, 4).opacity(0.15),
                    warning_fg: resolve_palette_hsla(PaletteKey::Yellow, 4),
                    error_bg: resolve_palette_hsla(PaletteKey::Red, 4).opacity(0.15),
                    error_fg: resolve_palette_hsla(PaletteKey::Red, 4),
                    card_width: px(360.0),
                    card_padding: px(12.0),
                    row_gap: px(8.0),
                    content_gap: px(4.0),
                    icon_box_size: px(24.0),
                    icon_size: px(17.0),
                    close_button_size: px(24.0),
                    close_icon_size: px(13.0),
                    title_size: px(14.0),
                    body_size: px(13.0),
                    stack_gap: px(8.0),
                    edge_offset: px(16.0),
                    top_offset_extra: px(8.0),
                },
                divider: DividerTokens {
                    line: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    label_size: px(12.0),
                    label_gap: px(8.0),
                    edge_span: px(16.0),
                },
                scroll_area: ScrollAreaTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    padding: default_inset_size_scale(),
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
                    title_size: px(16.0),
                    title_weight: FontWeight::SEMIBOLD,
                    body_size: px(14.0),
                    panel_padding: px(16.0),
                    panel_radius: px(0.0),
                    header_margin_bottom: px(8.0),
                    close_size: px(28.0),
                    close_icon_size: px(14.0),
                },
                app_shell: AppShellTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[9 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    title_bar_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[9 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    sidebar_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    sidebar_overlay_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    content_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[9 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    bottom_panel_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    inspector_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    inspector_overlay_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[8 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    region_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    title_bar_height: px(44.0),
                    sidebar_width: px(260.0),
                    sidebar_min_width: px(120.0),
                    inspector_width: px(320.0),
                    inspector_min_width: px(120.0),
                    bottom_panel_height: px(180.0),
                    bottom_panel_min_height: px(80.0),
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
                    height: default_title_bar_height_px(),
                    title_size: px(14.0),
                    title_weight: FontWeight::MEDIUM,
                    windows_button_width: px(45.0),
                    windows_icon_size: px(10.0),
                    linux_button_width: px(28.0),
                    linux_button_height: px(24.0),
                    linux_buttons_gap: px(6.0),
                    macos_controls_reserve: px(72.0),
                    title_padding_right: px(12.0),
                    title_max_width: px(320.0),
                    title_min_width: px(72.0),
                    platform_padding_left: px(12.0),
                    platform_padding_right: px(12.0),
                    controls_slot_gap: px(10.0),
                    control_button_radius: px(6.0),
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
                    inline_radius: px(10.0),
                    overlay_radius: px(18.0),
                    min_width: px(120.0),
                    section_padding: px(12.0),
                    footer_size: px(14.0),
                    scroll_padding: Size::Md,
                },
                markdown: MarkdownTokens {
                    paragraph: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[2 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    quote_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    quote_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    quote_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    code_bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    code_border: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    code_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    code_lang_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    list_marker: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    rule: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[5 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    gap_regular: px(8.0),
                    gap_compact: px(6.0),
                    paragraph_size: px(14.0),
                    quote_size: px(14.0),
                    code_size: px(13.0),
                    code_lang_size: px(12.0),
                    list_size: px(14.0),
                    quote_padding_x: px(12.0),
                    quote_padding_y: px(8.0),
                    quote_radius: px(8.0),
                    code_padding: px(10.0),
                    code_radius: px(8.0),
                    code_gap: px(6.0),
                    list_gap: px(6.0),
                    list_item_gap: px(8.0),
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
                    sizes: default_text_size_scale(),
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
                    padding: default_inset_size_scale(),
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
                    sizes: default_action_icon_size_scale(),
                },
                segmented_control: SegmentedControlTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[7 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    border: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[2 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    item_active_bg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Dark)[5 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    item_active_fg: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[0 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
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
                    track_padding: px(2.0),
                    item_gap: px(0.0),
                    sizes: default_segmented_control_size_scale(),
                },
                textarea: TextareaTokens {
                    bg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[8 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    fg: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    caret: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Gray)[0 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    selection_bg: (Rgba::try_from(PaletteCatalog::scale(primary)[8 as usize])
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
                    label_size: px(14.0),
                    label_weight: FontWeight::MEDIUM,
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    description_size: px(13.0),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    error_size: px(13.0),
                    label_block_gap: px(4.0),
                    label_row_gap: px(4.0),
                    layout_gap_vertical: px(8.0),
                    layout_gap_horizontal: px(12.0),
                    horizontal_label_width: px(168.0),
                    content_width_fallback: px(240.0),
                    sizes: default_field_size_scale(),
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
                    label_size: px(14.0),
                    label_weight: FontWeight::MEDIUM,
                    description: (Rgba::try_from(
                        PaletteCatalog::scale(PaletteKey::Gray)[4 as usize],
                    )
                    .map(Into::into)
                    .unwrap_or_else(|_| black())),
                    description_size: px(13.0),
                    error: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Red)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    error_size: px(13.0),
                    controls_width: px(18.0),
                    controls_height: px(12.0),
                    controls_icon_size: px(12.0),
                    controls_gap: px(8.0),
                    sizes: default_field_size_scale(),
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
                    label_size: px(14.0),
                    value_size: px(14.0),
                    header_gap_vertical: px(6.0),
                    header_gap_horizontal: px(8.0),
                    default_width: px(260.0),
                    min_width: px(140.0),
                    sizes: default_slider_size_scale(),
                },
                rating: RatingTokens {
                    active: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Yellow)[4 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    inactive: (Rgba::try_from(PaletteCatalog::scale(PaletteKey::Dark)[3 as usize])
                        .map(Into::into)
                        .unwrap_or_else(|_| black())),
                    sizes: default_rating_size_scale(),
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
                    root_gap: px(8.0),
                    list_gap: px(2.0),
                    list_padding: px(2.0),
                    panel_padding: px(16.0),
                    sizes: default_tabs_size_scale(),
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
                    root_gap: px(4.0),
                    sizes: default_pagination_size_scale(),
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
                    root_gap: px(4.0),
                    sizes: default_breadcrumbs_size_scale(),
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
                    caption_size: px(13.0),
                    row_gap: px(0.0),
                    pagination_summary_size: px(13.0),
                    page_chip_size: px(12.0),
                    page_chip_padding_x: px(8.0),
                    page_chip_padding_y: px(4.0),
                    page_chip_radius: px(6.0),
                    page_chip_gap: px(4.0),
                    pagination_items_gap: px(8.0),
                    pagination_padding_x: px(12.0),
                    pagination_padding_y: px(8.0),
                    pagination_gap: px(8.0),
                    virtualization_padding: px(4.0),
                    min_viewport_height: px(80.0),
                    sizes: default_table_size_scale(),
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
                    root_gap: px(6.0),
                    steps_gap_vertical: px(6.0),
                    text_gap: px(2.0),
                    panel_margin_top: px(8.0),
                    sizes: default_stepper_size_scale(),
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
                    root_gap: px(0.0),
                    row_gap: px(8.0),
                    content_gap: px(4.0),
                    card_margin_top: px(4.0),
                    row_padding_y: px(0.0),
                    line_min_height: px(24.0),
                    line_extra_height: px(8.0),
                    sizes: default_timeline_size_scale(),
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
                    root_gap: px(2.0),
                    children_gap: px(0.0),
                    sizes: default_tree_size_scale(),
                },
                layout: LayoutTokens {
                    gap: default_layout_gap_scale(),
                    space: default_layout_space_scale(),
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
    pub sizes: Option<ButtonSizeScale>,
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
        if let Some(value) = self.sizes {
            current.sizes = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InputOverrides {
    pub bg: Option<Hsla>,
    pub fg: Option<Hsla>,
    pub caret: Option<Hsla>,
    pub selection_bg: Option<Hsla>,
    pub placeholder: Option<Hsla>,
    pub border: Option<Hsla>,
    pub border_focus: Option<Hsla>,
    pub border_error: Option<Hsla>,
    pub label: Option<Hsla>,
    pub label_size: Option<Pixels>,
    pub label_weight: Option<FontWeight>,
    pub description: Option<Hsla>,
    pub description_size: Option<Pixels>,
    pub error: Option<Hsla>,
    pub error_size: Option<Pixels>,
    pub label_block_gap: Option<Pixels>,
    pub label_row_gap: Option<Pixels>,
    pub slot_fg: Option<Hsla>,
    pub slot_gap: Option<Pixels>,
    pub slot_min_width: Option<Pixels>,
    pub layout_gap_vertical: Option<Pixels>,
    pub layout_gap_horizontal: Option<Pixels>,
    pub horizontal_label_width: Option<Pixels>,
    pub pin_cells_gap: Option<Pixels>,
    pub pin_error_gap: Option<Pixels>,
    pub sizes: Option<FieldSizeScale>,
}

impl InputOverrides {
    fn apply(&self, mut current: InputTokens) -> InputTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.fg {
            current.fg = value.clone();
        }
        if let Some(value) = &self.caret {
            current.caret = value.clone();
        }
        if let Some(value) = &self.selection_bg {
            current.selection_bg = value.clone();
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
        if let Some(value) = self.label_size {
            current.label_size = value;
        }
        if let Some(value) = self.label_weight {
            current.label_weight = value;
        }
        if let Some(value) = &self.description {
            current.description = value.clone();
        }
        if let Some(value) = self.description_size {
            current.description_size = value;
        }
        if let Some(value) = &self.error {
            current.error = value.clone();
        }
        if let Some(value) = self.error_size {
            current.error_size = value;
        }
        if let Some(value) = self.label_block_gap {
            current.label_block_gap = value;
        }
        if let Some(value) = self.label_row_gap {
            current.label_row_gap = value;
        }
        if let Some(value) = &self.slot_fg {
            current.slot_fg = value.clone();
        }
        if let Some(value) = self.slot_gap {
            current.slot_gap = value;
        }
        if let Some(value) = self.slot_min_width {
            current.slot_min_width = value;
        }
        if let Some(value) = self.layout_gap_vertical {
            current.layout_gap_vertical = value;
        }
        if let Some(value) = self.layout_gap_horizontal {
            current.layout_gap_horizontal = value;
        }
        if let Some(value) = self.horizontal_label_width {
            current.horizontal_label_width = value;
        }
        if let Some(value) = self.pin_cells_gap {
            current.pin_cells_gap = value;
        }
        if let Some(value) = self.pin_error_gap {
            current.pin_error_gap = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RadioOverrides {
    pub control_bg: Option<Hsla>,
    pub border: Option<Hsla>,
    pub border_hover: Option<Hsla>,
    pub border_focus: Option<Hsla>,
    pub border_checked: Option<Hsla>,
    pub indicator: Option<Hsla>,
    pub label: Option<Hsla>,
    pub description: Option<Hsla>,
    pub label_description_gap: Option<Pixels>,
    pub group_gap_horizontal: Option<Pixels>,
    pub group_gap_vertical: Option<Pixels>,
    pub sizes: Option<ChoiceControlSizeScale>,
}

impl RadioOverrides {
    fn apply(&self, mut current: RadioTokens) -> RadioTokens {
        if let Some(value) = &self.control_bg {
            current.control_bg = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = &self.border_hover {
            current.border_hover = value.clone();
        }
        if let Some(value) = &self.border_focus {
            current.border_focus = value.clone();
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
        if let Some(value) = self.label_description_gap {
            current.label_description_gap = value;
        }
        if let Some(value) = self.group_gap_horizontal {
            current.group_gap_horizontal = value;
        }
        if let Some(value) = self.group_gap_vertical {
            current.group_gap_vertical = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CheckboxOverrides {
    pub control_bg: Option<Hsla>,
    pub control_bg_checked: Option<Hsla>,
    pub border: Option<Hsla>,
    pub border_hover: Option<Hsla>,
    pub border_focus: Option<Hsla>,
    pub border_checked: Option<Hsla>,
    pub indicator: Option<Hsla>,
    pub label: Option<Hsla>,
    pub description: Option<Hsla>,
    pub label_description_gap: Option<Pixels>,
    pub group_gap_horizontal: Option<Pixels>,
    pub group_gap_vertical: Option<Pixels>,
    pub sizes: Option<ChoiceControlSizeScale>,
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
        if let Some(value) = &self.border_hover {
            current.border_hover = value.clone();
        }
        if let Some(value) = &self.border_focus {
            current.border_focus = value.clone();
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
        if let Some(value) = self.label_description_gap {
            current.label_description_gap = value;
        }
        if let Some(value) = self.group_gap_horizontal {
            current.group_gap_horizontal = value;
        }
        if let Some(value) = self.group_gap_vertical {
            current.group_gap_vertical = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SwitchOverrides {
    pub track_off_bg: Option<Hsla>,
    pub track_on_bg: Option<Hsla>,
    pub track_hover_border: Option<Hsla>,
    pub track_focus_border: Option<Hsla>,
    pub thumb_bg: Option<Hsla>,
    pub label: Option<Hsla>,
    pub description: Option<Hsla>,
    pub label_description_gap: Option<Pixels>,
    pub sizes: Option<SwitchSizeScale>,
}

impl SwitchOverrides {
    fn apply(&self, mut current: SwitchTokens) -> SwitchTokens {
        if let Some(value) = &self.track_off_bg {
            current.track_off_bg = value.clone();
        }
        if let Some(value) = &self.track_on_bg {
            current.track_on_bg = value.clone();
        }
        if let Some(value) = &self.track_hover_border {
            current.track_hover_border = value.clone();
        }
        if let Some(value) = &self.track_focus_border {
            current.track_focus_border = value.clone();
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
        if let Some(value) = self.label_description_gap {
            current.label_description_gap = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub border_hover: Option<Hsla>,
    pub border_focus: Option<Hsla>,
    pub content_gap: Option<Pixels>,
    pub indicator_size: Option<Pixels>,
    pub group_gap_horizontal: Option<Pixels>,
    pub group_gap_vertical: Option<Pixels>,
    pub sizes: Option<ButtonSizeScale>,
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
        if let Some(value) = &self.border_hover {
            current.border_hover = value.clone();
        }
        if let Some(value) = &self.border_focus {
            current.border_focus = value.clone();
        }
        if let Some(value) = self.content_gap {
            current.content_gap = value;
        }
        if let Some(value) = self.indicator_size {
            current.indicator_size = value;
        }
        if let Some(value) = self.group_gap_horizontal {
            current.group_gap_horizontal = value;
        }
        if let Some(value) = self.group_gap_vertical {
            current.group_gap_vertical = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub sizes: Option<BadgeSizeScale>,
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
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub stack_gap: Option<Pixels>,
    pub header_gap: Option<Pixels>,
    pub label_stack_gap: Option<Pixels>,
    pub panel_gap: Option<Pixels>,
    pub sizes: Option<AccordionSizeScale>,
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
        if let Some(value) = self.stack_gap {
            current.stack_gap = value;
        }
        if let Some(value) = self.header_gap {
            current.header_gap = value;
        }
        if let Some(value) = self.label_stack_gap {
            current.label_stack_gap = value;
        }
        if let Some(value) = self.panel_gap {
            current.panel_gap = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub item_gap: Option<Pixels>,
    pub item_padding_x: Option<Pixels>,
    pub item_padding_y: Option<Pixels>,
    pub item_size: Option<Pixels>,
    pub item_icon_size: Option<Pixels>,
    pub item_radius: Option<Pixels>,
    pub dropdown_padding: Option<Pixels>,
    pub dropdown_gap: Option<Pixels>,
    pub dropdown_radius: Option<Pixels>,
    pub dropdown_width_fallback: Option<Pixels>,
    pub dropdown_min_width: Option<Pixels>,
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
        if let Some(value) = self.item_gap {
            current.item_gap = value;
        }
        if let Some(value) = self.item_padding_x {
            current.item_padding_x = value;
        }
        if let Some(value) = self.item_padding_y {
            current.item_padding_y = value;
        }
        if let Some(value) = self.item_size {
            current.item_size = value;
        }
        if let Some(value) = self.item_icon_size {
            current.item_icon_size = value;
        }
        if let Some(value) = self.item_radius {
            current.item_radius = value;
        }
        if let Some(value) = self.dropdown_padding {
            current.dropdown_padding = value;
        }
        if let Some(value) = self.dropdown_gap {
            current.dropdown_gap = value;
        }
        if let Some(value) = self.dropdown_radius {
            current.dropdown_radius = value;
        }
        if let Some(value) = self.dropdown_width_fallback {
            current.dropdown_width_fallback = value;
        }
        if let Some(value) = self.dropdown_min_width {
            current.dropdown_min_width = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProgressOverrides {
    pub track_bg: Option<Hsla>,
    pub fill_bg: Option<Hsla>,
    pub label: Option<Hsla>,
    pub default_width: Option<Pixels>,
    pub min_width: Option<Pixels>,
    pub root_gap: Option<Pixels>,
    pub sizes: Option<ProgressSizeScale>,
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
        if let Some(value) = self.default_width {
            current.default_width = value;
        }
        if let Some(value) = self.min_width {
            current.min_width = value;
        }
        if let Some(value) = self.root_gap {
            current.root_gap = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub label_size: Option<Pixels>,
    pub value_size: Option<Pixels>,
    pub header_gap_vertical: Option<Pixels>,
    pub header_gap_horizontal: Option<Pixels>,
    pub default_width: Option<Pixels>,
    pub min_width: Option<Pixels>,
    pub sizes: Option<SliderSizeScale>,
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
        if let Some(value) = self.label_size {
            current.label_size = value;
        }
        if let Some(value) = self.value_size {
            current.value_size = value;
        }
        if let Some(value) = self.header_gap_vertical {
            current.header_gap_vertical = value;
        }
        if let Some(value) = self.header_gap_horizontal {
            current.header_gap_horizontal = value;
        }
        if let Some(value) = self.default_width {
            current.default_width = value;
        }
        if let Some(value) = self.min_width {
            current.min_width = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
pub struct LoaderOverrides {
    pub color: Option<Hsla>,
    pub label: Option<Hsla>,
    pub sizes: Option<LoaderSizeScale>,
}

impl LoaderOverrides {
    fn apply(&self, mut current: LoaderTokens) -> LoaderTokens {
        if let Some(value) = &self.color {
            current.color = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LoadingOverlayOverrides {
    pub bg: Option<Hsla>,
    pub loader_color: Option<Hsla>,
    pub label: Option<Hsla>,
    pub content_gap: Option<Pixels>,
    pub label_size: Option<Pixels>,
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
        if let Some(value) = self.content_gap {
            current.content_gap = value;
        }
        if let Some(value) = self.label_size {
            current.label_size = value;
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
    pub padding: Option<Pixels>,
    pub gap: Option<Pixels>,
    pub radius: Option<Pixels>,
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
        if let Some(value) = self.padding {
            current.padding = value;
        }
        if let Some(value) = self.gap {
            current.gap = value;
        }
        if let Some(value) = self.radius {
            current.radius = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TooltipOverrides {
    pub bg: Option<Hsla>,
    pub fg: Option<Hsla>,
    pub border: Option<Hsla>,
    pub text_size: Option<Pixels>,
    pub padding_x: Option<Pixels>,
    pub padding_y: Option<Pixels>,
    pub radius: Option<Pixels>,
    pub max_width: Option<Pixels>,
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
        if let Some(value) = self.text_size {
            current.text_size = value;
        }
        if let Some(value) = self.padding_x {
            current.padding_x = value;
        }
        if let Some(value) = self.padding_y {
            current.padding_y = value;
        }
        if let Some(value) = self.radius {
            current.radius = value;
        }
        if let Some(value) = self.max_width {
            current.max_width = value;
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
    pub title_size: Option<Pixels>,
    pub title_weight: Option<FontWeight>,
    pub body_size: Option<Pixels>,
    pub min_width: Option<Pixels>,
    pub max_width: Option<Pixels>,
    pub padding: Option<Pixels>,
    pub gap: Option<Pixels>,
    pub radius: Option<Pixels>,
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
        if let Some(value) = self.title_size {
            current.title_size = value;
        }
        if let Some(value) = self.title_weight {
            current.title_weight = value;
        }
        if let Some(value) = self.body_size {
            current.body_size = value;
        }
        if let Some(value) = self.min_width {
            current.min_width = value;
        }
        if let Some(value) = self.max_width {
            current.max_width = value;
        }
        if let Some(value) = self.padding {
            current.padding = value;
        }
        if let Some(value) = self.gap {
            current.gap = value;
        }
        if let Some(value) = self.radius {
            current.radius = value;
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
    pub label_size: Option<Pixels>,
    pub label_weight: Option<FontWeight>,
    pub description: Option<Hsla>,
    pub description_size: Option<Pixels>,
    pub error: Option<Hsla>,
    pub error_size: Option<Pixels>,
    pub label_block_gap: Option<Pixels>,
    pub label_row_gap: Option<Pixels>,
    pub slot_gap: Option<Pixels>,
    pub slot_min_width: Option<Pixels>,
    pub layout_gap_vertical: Option<Pixels>,
    pub layout_gap_horizontal: Option<Pixels>,
    pub horizontal_label_width: Option<Pixels>,
    pub icon_size: Option<Pixels>,
    pub option_size: Option<Pixels>,
    pub option_padding_x: Option<Pixels>,
    pub option_padding_y: Option<Pixels>,
    pub option_content_gap: Option<Pixels>,
    pub option_check_size: Option<Pixels>,
    pub dropdown_padding: Option<Pixels>,
    pub dropdown_gap: Option<Pixels>,
    pub dropdown_max_height: Option<Pixels>,
    pub dropdown_width_fallback: Option<Pixels>,
    pub dropdown_open_preferred_height: Option<Pixels>,
    pub tag_size: Option<Pixels>,
    pub tag_padding_x: Option<Pixels>,
    pub tag_padding_y: Option<Pixels>,
    pub tag_gap: Option<Pixels>,
    pub tag_max_width: Option<Pixels>,
    pub dropdown_anchor_offset: Option<Pixels>,
    pub sizes: Option<FieldSizeScale>,
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
        if let Some(value) = self.label_size {
            current.label_size = value;
        }
        if let Some(value) = self.label_weight {
            current.label_weight = value;
        }
        if let Some(value) = &self.description {
            current.description = value.clone();
        }
        if let Some(value) = self.description_size {
            current.description_size = value;
        }
        if let Some(value) = &self.error {
            current.error = value.clone();
        }
        if let Some(value) = self.error_size {
            current.error_size = value;
        }
        if let Some(value) = self.label_block_gap {
            current.label_block_gap = value;
        }
        if let Some(value) = self.label_row_gap {
            current.label_row_gap = value;
        }
        if let Some(value) = self.slot_gap {
            current.slot_gap = value;
        }
        if let Some(value) = self.slot_min_width {
            current.slot_min_width = value;
        }
        if let Some(value) = self.layout_gap_vertical {
            current.layout_gap_vertical = value;
        }
        if let Some(value) = self.layout_gap_horizontal {
            current.layout_gap_horizontal = value;
        }
        if let Some(value) = self.horizontal_label_width {
            current.horizontal_label_width = value;
        }
        if let Some(value) = self.icon_size {
            current.icon_size = value;
        }
        if let Some(value) = self.option_size {
            current.option_size = value;
        }
        if let Some(value) = self.option_padding_x {
            current.option_padding_x = value;
        }
        if let Some(value) = self.option_padding_y {
            current.option_padding_y = value;
        }
        if let Some(value) = self.option_content_gap {
            current.option_content_gap = value;
        }
        if let Some(value) = self.option_check_size {
            current.option_check_size = value;
        }
        if let Some(value) = self.dropdown_padding {
            current.dropdown_padding = value;
        }
        if let Some(value) = self.dropdown_gap {
            current.dropdown_gap = value;
        }
        if let Some(value) = self.dropdown_max_height {
            current.dropdown_max_height = value;
        }
        if let Some(value) = self.dropdown_width_fallback {
            current.dropdown_width_fallback = value;
        }
        if let Some(value) = self.dropdown_open_preferred_height {
            current.dropdown_open_preferred_height = value;
        }
        if let Some(value) = self.tag_size {
            current.tag_size = value;
        }
        if let Some(value) = self.tag_padding_x {
            current.tag_padding_x = value;
        }
        if let Some(value) = self.tag_padding_y {
            current.tag_padding_y = value;
        }
        if let Some(value) = self.tag_gap {
            current.tag_gap = value;
        }
        if let Some(value) = self.tag_max_width {
            current.tag_max_width = value;
        }
        if let Some(value) = self.dropdown_anchor_offset {
            current.dropdown_anchor_offset = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub title_size: Option<Pixels>,
    pub title_weight: Option<FontWeight>,
    pub body_size: Option<Pixels>,
    pub kind_icon_size: Option<Pixels>,
    pub kind_icon_gap: Option<Pixels>,
    pub panel_radius: Option<Pixels>,
    pub panel_padding: Option<Pixels>,
    pub header_margin_bottom: Option<Pixels>,
    pub body_margin_bottom: Option<Pixels>,
    pub actions_margin_top: Option<Pixels>,
    pub actions_gap: Option<Pixels>,
    pub close_size: Option<Pixels>,
    pub close_icon_size: Option<Pixels>,
    pub default_width: Option<Pixels>,
    pub min_width: Option<Pixels>,
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
        if let Some(value) = self.title_size {
            current.title_size = value;
        }
        if let Some(value) = self.title_weight {
            current.title_weight = value;
        }
        if let Some(value) = self.body_size {
            current.body_size = value;
        }
        if let Some(value) = self.kind_icon_size {
            current.kind_icon_size = value;
        }
        if let Some(value) = self.kind_icon_gap {
            current.kind_icon_gap = value;
        }
        if let Some(value) = self.panel_radius {
            current.panel_radius = value;
        }
        if let Some(value) = self.panel_padding {
            current.panel_padding = value;
        }
        if let Some(value) = self.header_margin_bottom {
            current.header_margin_bottom = value;
        }
        if let Some(value) = self.body_margin_bottom {
            current.body_margin_bottom = value;
        }
        if let Some(value) = self.actions_margin_top {
            current.actions_margin_top = value;
        }
        if let Some(value) = self.actions_gap {
            current.actions_gap = value;
        }
        if let Some(value) = self.close_size {
            current.close_size = value;
        }
        if let Some(value) = self.close_icon_size {
            current.close_icon_size = value;
        }
        if let Some(value) = self.default_width {
            current.default_width = value;
        }
        if let Some(value) = self.min_width {
            current.min_width = value;
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
    pub card_width: Option<Pixels>,
    pub card_padding: Option<Pixels>,
    pub row_gap: Option<Pixels>,
    pub content_gap: Option<Pixels>,
    pub icon_box_size: Option<Pixels>,
    pub icon_size: Option<Pixels>,
    pub close_button_size: Option<Pixels>,
    pub close_icon_size: Option<Pixels>,
    pub title_size: Option<Pixels>,
    pub body_size: Option<Pixels>,
    pub stack_gap: Option<Pixels>,
    pub edge_offset: Option<Pixels>,
    pub top_offset_extra: Option<Pixels>,
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
        if let Some(value) = self.card_width {
            current.card_width = value;
        }
        if let Some(value) = self.card_padding {
            current.card_padding = value;
        }
        if let Some(value) = self.row_gap {
            current.row_gap = value;
        }
        if let Some(value) = self.content_gap {
            current.content_gap = value;
        }
        if let Some(value) = self.icon_box_size {
            current.icon_box_size = value;
        }
        if let Some(value) = self.icon_size {
            current.icon_size = value;
        }
        if let Some(value) = self.close_button_size {
            current.close_button_size = value;
        }
        if let Some(value) = self.close_icon_size {
            current.close_icon_size = value;
        }
        if let Some(value) = self.title_size {
            current.title_size = value;
        }
        if let Some(value) = self.body_size {
            current.body_size = value;
        }
        if let Some(value) = self.stack_gap {
            current.stack_gap = value;
        }
        if let Some(value) = self.edge_offset {
            current.edge_offset = value;
        }
        if let Some(value) = self.top_offset_extra {
            current.top_offset_extra = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DividerOverrides {
    pub line: Option<Hsla>,
    pub label: Option<Hsla>,
    pub label_size: Option<Pixels>,
    pub label_gap: Option<Pixels>,
    pub edge_span: Option<Pixels>,
}

impl DividerOverrides {
    fn apply(&self, mut current: DividerTokens) -> DividerTokens {
        if let Some(value) = &self.line {
            current.line = value.clone();
        }
        if let Some(value) = &self.label {
            current.label = value.clone();
        }
        if let Some(value) = self.label_size {
            current.label_size = value;
        }
        if let Some(value) = self.label_gap {
            current.label_gap = value;
        }
        if let Some(value) = self.edge_span {
            current.edge_span = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScrollAreaOverrides {
    pub bg: Option<Hsla>,
    pub border: Option<Hsla>,
    pub padding: Option<InsetSizeScale>,
}

impl ScrollAreaOverrides {
    fn apply(&self, mut current: ScrollAreaTokens) -> ScrollAreaTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = self.padding {
            current.padding = value;
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
    pub title_size: Option<Pixels>,
    pub title_weight: Option<FontWeight>,
    pub body_size: Option<Pixels>,
    pub panel_padding: Option<Pixels>,
    pub panel_radius: Option<Pixels>,
    pub header_margin_bottom: Option<Pixels>,
    pub close_size: Option<Pixels>,
    pub close_icon_size: Option<Pixels>,
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
        if let Some(value) = self.title_size {
            current.title_size = value;
        }
        if let Some(value) = self.title_weight {
            current.title_weight = value;
        }
        if let Some(value) = self.body_size {
            current.body_size = value;
        }
        if let Some(value) = self.panel_padding {
            current.panel_padding = value;
        }
        if let Some(value) = self.panel_radius {
            current.panel_radius = value;
        }
        if let Some(value) = self.header_margin_bottom {
            current.header_margin_bottom = value;
        }
        if let Some(value) = self.close_size {
            current.close_size = value;
        }
        if let Some(value) = self.close_icon_size {
            current.close_icon_size = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AppShellOverrides {
    pub bg: Option<Hsla>,
    pub title_bar_bg: Option<Hsla>,
    pub sidebar_bg: Option<Hsla>,
    pub sidebar_overlay_bg: Option<Hsla>,
    pub content_bg: Option<Hsla>,
    pub bottom_panel_bg: Option<Hsla>,
    pub inspector_bg: Option<Hsla>,
    pub inspector_overlay_bg: Option<Hsla>,
    pub region_border: Option<Hsla>,
    pub title_bar_height: Option<Pixels>,
    pub sidebar_width: Option<Pixels>,
    pub sidebar_min_width: Option<Pixels>,
    pub inspector_width: Option<Pixels>,
    pub inspector_min_width: Option<Pixels>,
    pub bottom_panel_height: Option<Pixels>,
    pub bottom_panel_min_height: Option<Pixels>,
}

impl AppShellOverrides {
    fn apply(&self, mut current: AppShellTokens) -> AppShellTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.title_bar_bg {
            current.title_bar_bg = value.clone();
        }
        if let Some(value) = &self.sidebar_bg {
            current.sidebar_bg = value.clone();
        }
        if let Some(value) = &self.sidebar_overlay_bg {
            current.sidebar_overlay_bg = value.clone();
        }
        if let Some(value) = &self.content_bg {
            current.content_bg = value.clone();
        }
        if let Some(value) = &self.bottom_panel_bg {
            current.bottom_panel_bg = value.clone();
        }
        if let Some(value) = &self.inspector_bg {
            current.inspector_bg = value.clone();
        }
        if let Some(value) = &self.inspector_overlay_bg {
            current.inspector_overlay_bg = value.clone();
        }
        if let Some(value) = &self.region_border {
            current.region_border = value.clone();
        }
        if let Some(value) = self.title_bar_height {
            current.title_bar_height = value;
        }
        if let Some(value) = self.sidebar_width {
            current.sidebar_width = value;
        }
        if let Some(value) = self.sidebar_min_width {
            current.sidebar_min_width = value;
        }
        if let Some(value) = self.inspector_width {
            current.inspector_width = value;
        }
        if let Some(value) = self.inspector_min_width {
            current.inspector_min_width = value;
        }
        if let Some(value) = self.bottom_panel_height {
            current.bottom_panel_height = value;
        }
        if let Some(value) = self.bottom_panel_min_height {
            current.bottom_panel_min_height = value;
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
    pub height: Option<Pixels>,
    pub title_size: Option<Pixels>,
    pub title_weight: Option<FontWeight>,
    pub windows_button_width: Option<Pixels>,
    pub windows_icon_size: Option<Pixels>,
    pub linux_button_width: Option<Pixels>,
    pub linux_button_height: Option<Pixels>,
    pub linux_buttons_gap: Option<Pixels>,
    pub macos_controls_reserve: Option<Pixels>,
    pub title_padding_right: Option<Pixels>,
    pub title_max_width: Option<Pixels>,
    pub title_min_width: Option<Pixels>,
    pub platform_padding_left: Option<Pixels>,
    pub platform_padding_right: Option<Pixels>,
    pub controls_slot_gap: Option<Pixels>,
    pub control_button_radius: Option<Pixels>,
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
        if let Some(value) = self.height {
            current.height = value;
        }
        if let Some(value) = self.title_size {
            current.title_size = value;
        }
        if let Some(value) = self.title_weight {
            current.title_weight = value;
        }
        if let Some(value) = self.windows_button_width {
            current.windows_button_width = value;
        }
        if let Some(value) = self.windows_icon_size {
            current.windows_icon_size = value;
        }
        if let Some(value) = self.linux_button_width {
            current.linux_button_width = value;
        }
        if let Some(value) = self.linux_button_height {
            current.linux_button_height = value;
        }
        if let Some(value) = self.linux_buttons_gap {
            current.linux_buttons_gap = value;
        }
        if let Some(value) = self.macos_controls_reserve {
            current.macos_controls_reserve = value;
        }
        if let Some(value) = self.title_padding_right {
            current.title_padding_right = value;
        }
        if let Some(value) = self.title_max_width {
            current.title_max_width = value;
        }
        if let Some(value) = self.title_min_width {
            current.title_min_width = value;
        }
        if let Some(value) = self.platform_padding_left {
            current.platform_padding_left = value;
        }
        if let Some(value) = self.platform_padding_right {
            current.platform_padding_right = value;
        }
        if let Some(value) = self.controls_slot_gap {
            current.controls_slot_gap = value;
        }
        if let Some(value) = self.control_button_radius {
            current.control_button_radius = value;
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
    pub inline_radius: Option<Pixels>,
    pub overlay_radius: Option<Pixels>,
    pub min_width: Option<Pixels>,
    pub section_padding: Option<Pixels>,
    pub footer_size: Option<Pixels>,
    pub scroll_padding: Option<Size>,
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
        if let Some(value) = self.inline_radius {
            current.inline_radius = value;
        }
        if let Some(value) = self.overlay_radius {
            current.overlay_radius = value;
        }
        if let Some(value) = self.min_width {
            current.min_width = value;
        }
        if let Some(value) = self.section_padding {
            current.section_padding = value;
        }
        if let Some(value) = self.footer_size {
            current.footer_size = value;
        }
        if let Some(value) = self.scroll_padding {
            current.scroll_padding = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MarkdownOverrides {
    pub paragraph: Option<Hsla>,
    pub quote_bg: Option<Hsla>,
    pub quote_border: Option<Hsla>,
    pub quote_fg: Option<Hsla>,
    pub code_bg: Option<Hsla>,
    pub code_border: Option<Hsla>,
    pub code_fg: Option<Hsla>,
    pub code_lang_fg: Option<Hsla>,
    pub list_marker: Option<Hsla>,
    pub rule: Option<Hsla>,
    pub gap_regular: Option<Pixels>,
    pub gap_compact: Option<Pixels>,
    pub paragraph_size: Option<Pixels>,
    pub quote_size: Option<Pixels>,
    pub code_size: Option<Pixels>,
    pub code_lang_size: Option<Pixels>,
    pub list_size: Option<Pixels>,
    pub quote_padding_x: Option<Pixels>,
    pub quote_padding_y: Option<Pixels>,
    pub quote_radius: Option<Pixels>,
    pub code_padding: Option<Pixels>,
    pub code_radius: Option<Pixels>,
    pub code_gap: Option<Pixels>,
    pub list_gap: Option<Pixels>,
    pub list_item_gap: Option<Pixels>,
}

impl MarkdownOverrides {
    fn apply(&self, mut current: MarkdownTokens) -> MarkdownTokens {
        if let Some(value) = &self.paragraph {
            current.paragraph = value.clone();
        }
        if let Some(value) = &self.quote_bg {
            current.quote_bg = value.clone();
        }
        if let Some(value) = &self.quote_border {
            current.quote_border = value.clone();
        }
        if let Some(value) = &self.quote_fg {
            current.quote_fg = value.clone();
        }
        if let Some(value) = &self.code_bg {
            current.code_bg = value.clone();
        }
        if let Some(value) = &self.code_border {
            current.code_border = value.clone();
        }
        if let Some(value) = &self.code_fg {
            current.code_fg = value.clone();
        }
        if let Some(value) = &self.code_lang_fg {
            current.code_lang_fg = value.clone();
        }
        if let Some(value) = &self.list_marker {
            current.list_marker = value.clone();
        }
        if let Some(value) = &self.rule {
            current.rule = value.clone();
        }
        if let Some(value) = self.gap_regular {
            current.gap_regular = value;
        }
        if let Some(value) = self.gap_compact {
            current.gap_compact = value;
        }
        if let Some(value) = self.paragraph_size {
            current.paragraph_size = value;
        }
        if let Some(value) = self.quote_size {
            current.quote_size = value;
        }
        if let Some(value) = self.code_size {
            current.code_size = value;
        }
        if let Some(value) = self.code_lang_size {
            current.code_lang_size = value;
        }
        if let Some(value) = self.list_size {
            current.list_size = value;
        }
        if let Some(value) = self.quote_padding_x {
            current.quote_padding_x = value;
        }
        if let Some(value) = self.quote_padding_y {
            current.quote_padding_y = value;
        }
        if let Some(value) = self.quote_radius {
            current.quote_radius = value;
        }
        if let Some(value) = self.code_padding {
            current.code_padding = value;
        }
        if let Some(value) = self.code_radius {
            current.code_radius = value;
        }
        if let Some(value) = self.code_gap {
            current.code_gap = value;
        }
        if let Some(value) = self.list_gap {
            current.list_gap = value;
        }
        if let Some(value) = self.list_item_gap {
            current.list_item_gap = value;
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
    pub sizes: Option<TextSizeScale>,
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
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub padding: Option<InsetSizeScale>,
}

impl PaperOverrides {
    fn apply(&self, mut current: PaperTokens) -> PaperTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.border {
            current.border = value.clone();
        }
        if let Some(value) = self.padding {
            current.padding = value;
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
    pub sizes: Option<ActionIconSizeScale>,
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
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub track_padding: Option<Pixels>,
    pub item_gap: Option<Pixels>,
    pub sizes: Option<SegmentedControlSizeScale>,
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
        if let Some(value) = self.track_padding {
            current.track_padding = value;
        }
        if let Some(value) = self.item_gap {
            current.item_gap = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TextareaOverrides {
    pub bg: Option<Hsla>,
    pub fg: Option<Hsla>,
    pub caret: Option<Hsla>,
    pub selection_bg: Option<Hsla>,
    pub placeholder: Option<Hsla>,
    pub border: Option<Hsla>,
    pub border_focus: Option<Hsla>,
    pub border_error: Option<Hsla>,
    pub label: Option<Hsla>,
    pub label_size: Option<Pixels>,
    pub label_weight: Option<FontWeight>,
    pub description: Option<Hsla>,
    pub description_size: Option<Pixels>,
    pub error: Option<Hsla>,
    pub error_size: Option<Pixels>,
    pub label_block_gap: Option<Pixels>,
    pub label_row_gap: Option<Pixels>,
    pub layout_gap_vertical: Option<Pixels>,
    pub layout_gap_horizontal: Option<Pixels>,
    pub horizontal_label_width: Option<Pixels>,
    pub content_width_fallback: Option<Pixels>,
    pub sizes: Option<FieldSizeScale>,
}

impl TextareaOverrides {
    fn apply(&self, mut current: TextareaTokens) -> TextareaTokens {
        if let Some(value) = &self.bg {
            current.bg = value.clone();
        }
        if let Some(value) = &self.fg {
            current.fg = value.clone();
        }
        if let Some(value) = &self.caret {
            current.caret = value.clone();
        }
        if let Some(value) = &self.selection_bg {
            current.selection_bg = value.clone();
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
        if let Some(value) = self.label_size {
            current.label_size = value;
        }
        if let Some(value) = self.label_weight {
            current.label_weight = value;
        }
        if let Some(value) = &self.description {
            current.description = value.clone();
        }
        if let Some(value) = self.description_size {
            current.description_size = value;
        }
        if let Some(value) = &self.error {
            current.error = value.clone();
        }
        if let Some(value) = self.error_size {
            current.error_size = value;
        }
        if let Some(value) = self.label_block_gap {
            current.label_block_gap = value;
        }
        if let Some(value) = self.label_row_gap {
            current.label_row_gap = value;
        }
        if let Some(value) = self.layout_gap_vertical {
            current.layout_gap_vertical = value;
        }
        if let Some(value) = self.layout_gap_horizontal {
            current.layout_gap_horizontal = value;
        }
        if let Some(value) = self.horizontal_label_width {
            current.horizontal_label_width = value;
        }
        if let Some(value) = self.content_width_fallback {
            current.content_width_fallback = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub label_size: Option<Pixels>,
    pub label_weight: Option<FontWeight>,
    pub description: Option<Hsla>,
    pub description_size: Option<Pixels>,
    pub error: Option<Hsla>,
    pub error_size: Option<Pixels>,
    pub controls_width: Option<Pixels>,
    pub controls_height: Option<Pixels>,
    pub controls_icon_size: Option<Pixels>,
    pub controls_gap: Option<Pixels>,
    pub sizes: Option<FieldSizeScale>,
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
        if let Some(value) = self.label_size {
            current.label_size = value;
        }
        if let Some(value) = self.label_weight {
            current.label_weight = value;
        }
        if let Some(value) = &self.description {
            current.description = value.clone();
        }
        if let Some(value) = self.description_size {
            current.description_size = value;
        }
        if let Some(value) = &self.error {
            current.error = value.clone();
        }
        if let Some(value) = self.error_size {
            current.error_size = value;
        }
        if let Some(value) = self.controls_width {
            current.controls_width = value;
        }
        if let Some(value) = self.controls_height {
            current.controls_height = value;
        }
        if let Some(value) = self.controls_icon_size {
            current.controls_icon_size = value;
        }
        if let Some(value) = self.controls_gap {
            current.controls_gap = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub label_size: Option<Pixels>,
    pub value_size: Option<Pixels>,
    pub header_gap_vertical: Option<Pixels>,
    pub header_gap_horizontal: Option<Pixels>,
    pub default_width: Option<Pixels>,
    pub min_width: Option<Pixels>,
    pub sizes: Option<SliderSizeScale>,
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
        if let Some(value) = self.label_size {
            current.label_size = value;
        }
        if let Some(value) = self.value_size {
            current.value_size = value;
        }
        if let Some(value) = self.header_gap_vertical {
            current.header_gap_vertical = value;
        }
        if let Some(value) = self.header_gap_horizontal {
            current.header_gap_horizontal = value;
        }
        if let Some(value) = self.default_width {
            current.default_width = value;
        }
        if let Some(value) = self.min_width {
            current.min_width = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RatingOverrides {
    pub active: Option<Hsla>,
    pub inactive: Option<Hsla>,
    pub sizes: Option<RatingSizeScale>,
}

impl RatingOverrides {
    fn apply(&self, mut current: RatingTokens) -> RatingTokens {
        if let Some(value) = &self.active {
            current.active = value.clone();
        }
        if let Some(value) = &self.inactive {
            current.inactive = value.clone();
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub root_gap: Option<Pixels>,
    pub list_gap: Option<Pixels>,
    pub list_padding: Option<Pixels>,
    pub panel_padding: Option<Pixels>,
    pub sizes: Option<TabsSizeScale>,
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
        if let Some(value) = self.root_gap {
            current.root_gap = value;
        }
        if let Some(value) = self.list_gap {
            current.list_gap = value;
        }
        if let Some(value) = self.list_padding {
            current.list_padding = value;
        }
        if let Some(value) = self.panel_padding {
            current.panel_padding = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub root_gap: Option<Pixels>,
    pub sizes: Option<PaginationSizeScale>,
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
        if let Some(value) = self.root_gap {
            current.root_gap = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub root_gap: Option<Pixels>,
    pub sizes: Option<BreadcrumbsSizeScale>,
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
        if let Some(value) = self.root_gap {
            current.root_gap = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub caption_size: Option<Pixels>,
    pub row_gap: Option<Pixels>,
    pub pagination_summary_size: Option<Pixels>,
    pub page_chip_size: Option<Pixels>,
    pub page_chip_padding_x: Option<Pixels>,
    pub page_chip_padding_y: Option<Pixels>,
    pub page_chip_radius: Option<Pixels>,
    pub page_chip_gap: Option<Pixels>,
    pub pagination_items_gap: Option<Pixels>,
    pub pagination_padding_x: Option<Pixels>,
    pub pagination_padding_y: Option<Pixels>,
    pub pagination_gap: Option<Pixels>,
    pub virtualization_padding: Option<Pixels>,
    pub min_viewport_height: Option<Pixels>,
    pub sizes: Option<TableSizeScale>,
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
        if let Some(value) = self.caption_size {
            current.caption_size = value;
        }
        if let Some(value) = self.row_gap {
            current.row_gap = value;
        }
        if let Some(value) = self.pagination_summary_size {
            current.pagination_summary_size = value;
        }
        if let Some(value) = self.page_chip_size {
            current.page_chip_size = value;
        }
        if let Some(value) = self.page_chip_padding_x {
            current.page_chip_padding_x = value;
        }
        if let Some(value) = self.page_chip_padding_y {
            current.page_chip_padding_y = value;
        }
        if let Some(value) = self.page_chip_radius {
            current.page_chip_radius = value;
        }
        if let Some(value) = self.page_chip_gap {
            current.page_chip_gap = value;
        }
        if let Some(value) = self.pagination_items_gap {
            current.pagination_items_gap = value;
        }
        if let Some(value) = self.pagination_padding_x {
            current.pagination_padding_x = value;
        }
        if let Some(value) = self.pagination_padding_y {
            current.pagination_padding_y = value;
        }
        if let Some(value) = self.pagination_gap {
            current.pagination_gap = value;
        }
        if let Some(value) = self.virtualization_padding {
            current.virtualization_padding = value;
        }
        if let Some(value) = self.min_viewport_height {
            current.min_viewport_height = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub root_gap: Option<Pixels>,
    pub steps_gap_vertical: Option<Pixels>,
    pub text_gap: Option<Pixels>,
    pub panel_margin_top: Option<Pixels>,
    pub sizes: Option<StepperSizeScale>,
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
        if let Some(value) = self.root_gap {
            current.root_gap = value;
        }
        if let Some(value) = self.steps_gap_vertical {
            current.steps_gap_vertical = value;
        }
        if let Some(value) = self.text_gap {
            current.text_gap = value;
        }
        if let Some(value) = self.panel_margin_top {
            current.panel_margin_top = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub root_gap: Option<Pixels>,
    pub row_gap: Option<Pixels>,
    pub content_gap: Option<Pixels>,
    pub card_margin_top: Option<Pixels>,
    pub row_padding_y: Option<Pixels>,
    pub line_min_height: Option<Pixels>,
    pub line_extra_height: Option<Pixels>,
    pub sizes: Option<TimelineSizeScale>,
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
        if let Some(value) = self.root_gap {
            current.root_gap = value;
        }
        if let Some(value) = self.row_gap {
            current.row_gap = value;
        }
        if let Some(value) = self.content_gap {
            current.content_gap = value;
        }
        if let Some(value) = self.card_margin_top {
            current.card_margin_top = value;
        }
        if let Some(value) = self.row_padding_y {
            current.row_padding_y = value;
        }
        if let Some(value) = self.line_min_height {
            current.line_min_height = value;
        }
        if let Some(value) = self.line_extra_height {
            current.line_extra_height = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
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
    pub root_gap: Option<Pixels>,
    pub children_gap: Option<Pixels>,
    pub sizes: Option<TreeSizeScale>,
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
        if let Some(value) = self.root_gap {
            current.root_gap = value;
        }
        if let Some(value) = self.children_gap {
            current.children_gap = value;
        }
        if let Some(value) = self.sizes {
            current.sizes = value;
        }
        current
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LayoutOverrides {
    pub gap: Option<GapSizeScale>,
    pub space: Option<GapSizeScale>,
}

impl LayoutOverrides {
    fn apply(&self, mut current: LayoutTokens) -> LayoutTokens {
        if let Some(value) = self.gap {
            current.gap = value;
        }
        if let Some(value) = self.space {
            current.space = value;
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
    pub loader: LoaderOverrides,
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
    pub markdown: MarkdownOverrides,
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
    pub layout: LayoutOverrides,
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
            loader: self.loader.apply(current.loader),
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
            markdown: self.markdown.apply(current.markdown),
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
            layout: self.layout.apply(current.layout),
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
        let base = crate::provider::CalmProvider::theme(cx);
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

    #[test]
    fn dark_theme_uses_subtle_alert_surfaces() {
        let dark = Theme::default().with_color_scheme(ColorScheme::Dark);
        let toast = dark.components.toast;

        assert_eq!(
            toast.info_bg,
            resolve_palette_hsla(PaletteKey::Blue, 4).opacity(0.15)
        );
        assert_eq!(toast.info_fg, resolve_palette_hsla(PaletteKey::Blue, 4));

        assert_eq!(
            toast.success_bg,
            resolve_palette_hsla(PaletteKey::Green, 4).opacity(0.15)
        );
        assert_eq!(toast.success_fg, resolve_palette_hsla(PaletteKey::Green, 4));

        assert_eq!(
            toast.warning_bg,
            resolve_palette_hsla(PaletteKey::Yellow, 4).opacity(0.15)
        );
        assert_eq!(
            toast.warning_fg,
            resolve_palette_hsla(PaletteKey::Yellow, 4)
        );

        assert_eq!(
            toast.error_bg,
            resolve_palette_hsla(PaletteKey::Red, 4).opacity(0.15)
        );
        assert_eq!(toast.error_fg, resolve_palette_hsla(PaletteKey::Red, 4));
    }

    #[test]
    fn input_dimension_overrides_are_applied() {
        let mut scale = default_field_size_scale();
        scale.md.font_size = px(17.0);
        scale.md.padding_x = px(13.0);

        let themed = Theme::default().with_overrides(|overrides| {
            overrides.input(|input| {
                input
                    .label_size(px(15.0))
                    .horizontal_label_width(px(196.0))
                    .slot_gap(px(10.0))
                    .sizes(scale)
            })
        });

        assert_eq!(themed.components.input.label_size, px(15.0));
        assert_eq!(themed.components.input.horizontal_label_width, px(196.0));
        assert_eq!(themed.components.input.slot_gap, px(10.0));
        assert_eq!(themed.components.input.sizes.md.font_size, px(17.0));
        assert_eq!(themed.components.input.sizes.md.padding_x, px(13.0));
    }
}
