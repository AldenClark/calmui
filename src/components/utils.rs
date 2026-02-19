use std::rc::Rc;

use gpui::{ClickEvent, FontWeight, Hsla, Pixels, StatefulInteractiveElement, Styled, Window, px};

use crate::style::{Radius, Variant};
use crate::theme::{FieldSizePreset, ResolveWithTheme, SemanticRadiusToken, Theme};

pub type PressHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut gpui::App)>;

#[derive(Clone, Default)]
pub struct InteractionStyles {
    pub hover: Option<gpui::StyleRefinement>,
    pub active: Option<gpui::StyleRefinement>,
    pub focus: Option<gpui::StyleRefinement>,
}

impl InteractionStyles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hover(mut self, value: gpui::StyleRefinement) -> Self {
        self.hover = Some(value);
        self
    }

    pub fn active(mut self, value: gpui::StyleRefinement) -> Self {
        self.active = Some(value);
        self
    }

    pub fn focus(mut self, value: gpui::StyleRefinement) -> Self {
        self.focus = Some(value);
        self
    }
}

pub fn interaction_style(
    apply: impl FnOnce(gpui::StyleRefinement) -> gpui::StyleRefinement,
) -> gpui::StyleRefinement {
    apply(gpui::StyleRefinement::default())
}

pub fn apply_interaction_styles<T>(mut node: T, styles: InteractionStyles) -> T
where
    T: StatefulInteractiveElement,
{
    if let Some(hover_style) = styles.hover {
        node = node.hover(move |_| hover_style);
    }

    if let Some(active_style) = styles.active {
        node = node.active(move |_| active_style);
    }

    if let Some(focus_style) = styles.focus {
        node = node.focus(move |_| focus_style);
    }

    node
}

pub fn default_pressable_surface_styles(bg: Hsla, focus_border: Hsla) -> InteractionStyles {
    let hover_bg = bg.blend(gpui::white().opacity(0.06));
    let active_bg = bg.blend(gpui::black().opacity(0.12));

    InteractionStyles::new()
        .hover(interaction_style(move |style| style.bg(hover_bg)))
        .active(interaction_style(move |style| style.bg(active_bg)))
        .focus(interaction_style(move |style| {
            style.border_color(focus_border)
        }))
}

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

pub fn apply_field_size<T: Styled>(div: T, preset: FieldSizePreset) -> T {
    let min_h = px(f32::from(preset.line_height) + f32::from(preset.padding_y) * 2.0);
    div.text_size(preset.font_size)
        .line_height(preset.line_height)
        .py(preset.padding_y)
        .px(preset.padding_x)
        .min_h(min_h)
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

pub fn deepened_surface_border(bg: Hsla) -> Hsla {
    bg.blend(gpui::black().opacity(0.12))
}

pub fn offset_with_progress(offset_px: i16, progress: f32) -> f32 {
    let full = offset_px as f32;
    full * (1.0 - progress)
}

fn scale_factor(window: &Window) -> f32 {
    window.scale_factor().max(f32::EPSILON)
}

pub fn snap_px(window: &Window, logical_px: f32) -> Pixels {
    if !logical_px.is_finite() {
        return px(0.0);
    }
    let scale = scale_factor(window);
    px((logical_px * scale).round() / scale)
}

pub fn hairline_px(window: &Window) -> Pixels {
    px(1.0 / scale_factor(window))
}

pub fn quantized_stroke_px(window: &Window, logical_px: f32) -> Pixels {
    if !logical_px.is_finite() || logical_px <= 0.0 {
        return px(0.0);
    }
    let snapped = snap_px(window, logical_px);
    if f32::from(snapped) > 0.0 {
        snapped
    } else {
        hairline_px(window)
    }
}
