use gpui::{FontWeight, Hsla, Rgba, Styled, black, px};

use crate::style::{Radius, Size, Variant};
use crate::theme::{ColorValue, Theme};

pub fn resolve_hsla(theme: &Theme, token: &ColorValue) -> Hsla {
    let raw = theme.resolve_color(token);
    if let Ok(rgba) = Rgba::try_from(raw.as_str()) {
        rgba.into()
    } else {
        black()
    }
}

pub fn apply_radius<T: Styled>(div: T, radius: Radius) -> T {
    match radius {
        Radius::Xs => div.rounded_xs(),
        Radius::Sm => div.rounded_sm(),
        Radius::Md => div.rounded_md(),
        Radius::Lg => div.rounded_lg(),
        Radius::Xl => div.rounded_xl(),
        Radius::Pill => div.rounded_full(),
    }
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
