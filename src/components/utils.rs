use gpui::{FontWeight, Hsla, Pixels, Styled, px};

use crate::style::{Radius, Size, Variant};
use crate::theme::{ResolveWithTheme, SemanticRadiusToken, Theme};

pub fn resolve_hsla<T>(theme: &Theme, token: T) -> Hsla
where
    T: ResolveWithTheme<Hsla>,
{
    theme.resolve_hsla(token)
}

pub fn resolve_radius<T>(theme: &Theme, token: T) -> Pixels
where
    T: ResolveWithTheme<Pixels>,
{
    theme.resolve_radius(token)
}

pub fn apply_radius<T: Styled>(theme: &Theme, div: T, radius: Radius) -> T {
    div.rounded(resolve_radius(theme, SemanticRadiusToken::from(radius)))
}

pub fn apply_button_size<T: Styled>(div: T, size: Size) -> T {
    match size {
        Size::Xs => div.text_xs().py(px(4.0)).px(px(8.0)),
        Size::Sm => div.text_sm().py(px(6.0)).px(px(10.0)),
        Size::Md => div.text_base().py(px(8.0)).px(px(12.0)),
        Size::Lg => div.text_lg().py(px(10.0)).px(px(14.0)),
        Size::Xl => div.text_xl().py(px(12.0)).px(px(16.0)),
    }
}

pub fn apply_input_size<T: Styled>(div: T, size: Size) -> T {
    match size {
        Size::Xs => div.text_xs().py(px(5.0)).px(px(8.0)),
        Size::Sm => div.text_sm().py(px(6.0)).px(px(10.0)),
        Size::Md => div.text_base().py(px(8.0)).px(px(12.0)),
        Size::Lg => div.text_lg().py(px(10.0)).px(px(14.0)),
        Size::Xl => div.text_xl().py(px(12.0)).px(px(16.0)),
    }
}

pub fn variant_text_weight(variant: Variant) -> FontWeight {
    match variant {
        Variant::Filled => FontWeight::SEMIBOLD,
        Variant::Light => FontWeight::MEDIUM,
        Variant::Subtle => FontWeight::MEDIUM,
        Variant::Outline => FontWeight::MEDIUM,
        Variant::Ghost => FontWeight::MEDIUM,
        Variant::Default => FontWeight::MEDIUM,
    }
}

pub fn offset_with_progress(offset_px: i16, progress: f32) -> f32 {
    let full = offset_px as f32;
    full * (1.0 - progress)
}
