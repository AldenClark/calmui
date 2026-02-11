use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, Component, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, div, px,
};

use crate::contracts::{MotionAware, VariantSupport, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};
use crate::theme::ColorValue;

use super::primitives::h_stack;
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

#[derive(Clone, Debug, PartialEq)]
pub struct ProgressSection {
    value: f32,
    color: Option<ColorValue>,
}

impl ProgressSection {
    pub fn new(value: f32) -> Self {
        Self { value, color: None }
    }

    pub fn color(mut self, color: ColorValue) -> Self {
        self.color = Some(color);
        self
    }
}

pub struct Progress {
    id: String,
    value: f32,
    sections: Vec<ProgressSection>,
    label: Option<SharedString>,
    show_value: bool,
    striped: bool,
    animated: bool,
    width_px: f32,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    motion: MotionConfig,
}

impl Progress {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("progress"),
            value: 0.0,
            sections: Vec::new(),
            label: None,
            show_value: false,
            striped: false,
            animated: false,
            width_px: 260.0,
            variant: Variant::Filled,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
        }
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    pub fn section(mut self, section: ProgressSection) -> Self {
        self.sections.push(section);
        self
    }

    pub fn sections(mut self, sections: impl IntoIterator<Item = ProgressSection>) -> Self {
        self.sections.extend(sections);
        self
    }

    pub fn clear_sections(mut self) -> Self {
        self.sections.clear();
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn show_value(mut self, show_value: bool) -> Self {
        self.show_value = show_value;
        self
    }

    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn width(mut self, width_px: f32) -> Self {
        self.width_px = width_px.max(80.0);
        self
    }

    fn normalized_value(value: f32) -> f32 {
        value.clamp(0.0, 100.0)
    }

    fn bar_height_px(&self) -> f32 {
        match self.size {
            Size::Xs => 4.0,
            Size::Sm => 6.0,
            Size::Md => 8.0,
            Size::Lg => 12.0,
            Size::Xl => 16.0,
        }
    }

    fn variant_fill_color(&self) -> gpui::Hsla {
        let base = resolve_hsla(&self.theme, &self.theme.components.progress.fill_bg);
        match self.variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.8),
            Variant::Subtle => base.alpha(0.65),
            Variant::Outline => base.alpha(0.9),
            Variant::Ghost => base.alpha(0.55),
        }
    }

    fn resolved_sections(&self) -> Vec<ProgressSection> {
        if self.sections.is_empty() {
            return vec![ProgressSection::new(Self::normalized_value(self.value))];
        }

        let mut remaining = 100.0_f32;
        let mut normalized = Vec::with_capacity(self.sections.len());
        for section in &self.sections {
            if remaining <= 0.0 {
                break;
            }
            let value = Self::normalized_value(section.value).min(remaining);
            remaining -= value;
            normalized.push(ProgressSection {
                value,
                color: section.color.clone(),
            });
        }
        normalized
    }

    fn striped_overlay(
        color: gpui::Hsla,
        key: String,
        width_px: f32,
        bar_height: f32,
        animated: bool,
    ) -> AnyElement {
        let stripe_width = (bar_height * 1.7).clamp(8.0, 18.0);
        let primary_band_width = width_px * 2.4;
        let secondary_band_width = width_px * 1.9;

        let primary_count = (primary_band_width / stripe_width).ceil().max(10.0) as usize;
        let secondary_count = (secondary_band_width / (stripe_width * 1.25))
            .ceil()
            .max(8.0) as usize;

        let primary_prefix = format!("{key}-primary");
        let primary_stripes = (0..primary_count).map(move |index| {
            let alpha = match index % 3 {
                0 => 0.18,
                1 => 0.10,
                _ => 0.04,
            };
            div()
                .id(format!("{primary_prefix}-{index}"))
                .w(px(stripe_width))
                .h_full()
                .bg(color.alpha(alpha))
                .into_any_element()
        });

        let secondary_prefix = format!("{key}-secondary");
        let secondary_stripes = (0..secondary_count).map(move |index| {
            let alpha = if index % 2 == 0 { 0.08 } else { 0.03 };
            div()
                .id(format!("{secondary_prefix}-{index}"))
                .w(px(stripe_width * 1.25))
                .h_full()
                .bg(color.alpha(alpha))
                .into_any_element()
        });

        let primary_band = h_stack()
            .id(format!("{key}-band-primary"))
            .absolute()
            .top_0()
            .left(px(-width_px * 1.15))
            .bottom_0()
            .w(px(primary_band_width))
            .children(primary_stripes);

        let secondary_band = h_stack()
            .id(format!("{key}-band-secondary"))
            .absolute()
            .top_0()
            .left(px(-width_px * 0.2))
            .bottom_0()
            .w(px(secondary_band_width))
            .children(secondary_stripes);

        let animation_ms = (1500.0 + width_px * 2.4).clamp(1700.0, 2900.0) as u64;

        let primary_band = if animated {
            let animation = Animation::new(Duration::from_millis(animation_ms))
                .repeat()
                .with_easing(gpui::linear);
            let travel = width_px * 1.35;
            primary_band
                .with_animation(
                    format!("{key}-move-primary"),
                    animation,
                    move |this, delta| this.left(px(-width_px * 1.15 + travel * delta)),
                )
                .into_any_element()
        } else {
            primary_band.into_any_element()
        };

        let secondary_band = if animated {
            let animation =
                Animation::new(Duration::from_millis((animation_ms as f32 * 2.1) as u64))
                    .repeat()
                    .with_easing(gpui::linear);
            let travel = width_px * 1.1;
            secondary_band
                .with_animation(
                    format!("{key}-move-secondary"),
                    animation,
                    move |this, delta| this.left(px(-width_px * 0.2 - travel * delta)),
                )
                .into_any_element()
        } else {
            secondary_band.into_any_element()
        };

        let sheen_width = (width_px * 0.22).clamp(26.0, 78.0);
        let sheen = div()
            .id(format!("{key}-sheen"))
            .absolute()
            .top_0()
            .bottom_0()
            .left(px(-sheen_width))
            .w(px(sheen_width))
            .bg(color.alpha(0.16))
            .shadow_sm();

        let sheen = if animated {
            let animation =
                Animation::new(Duration::from_millis((animation_ms as f32 * 1.9) as u64))
                    .repeat()
                    .with_easing(gpui::linear);
            let travel = width_px + sheen_width * 2.0;
            sheen
                .with_animation(
                    format!("{key}-sheen-move"),
                    animation,
                    move |this, delta| {
                        let stepped = if delta < 0.12 {
                            0.0
                        } else if delta > 0.96 {
                            1.0
                        } else {
                            gpui::ease_in_out((delta - 0.12) / 0.84)
                        };
                        this.left(px(-sheen_width + travel * stepped))
                    },
                )
                .into_any_element()
        } else {
            sheen.into_any_element()
        };

        div()
            .id(key)
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .overflow_hidden()
            .child(primary_band)
            .child(secondary_band)
            .child(sheen)
            .into_any_element()
    }
}

impl WithId for Progress {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantSupport for Progress {
    fn variant(mut self, value: Variant) -> Self {
        self.variant = value;
        self
    }

    fn size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    fn radius(mut self, value: Radius) -> Self {
        self.radius = value;
        self
    }
}

impl MotionAware for Progress {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Progress {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = &self.theme.components.progress;
        let track_bg = resolve_hsla(&self.theme, &tokens.track_bg);
        let default_fill = self.variant_fill_color();
        let sections = self.resolved_sections();
        let bar_height = self.bar_height_px();
        let total_value = sections
            .iter()
            .fold(0.0_f32, |acc, section| acc + section.value);

        let mut track = div()
            .id(format!("{}-track", self.id))
            .relative()
            .w(px(self.width_px))
            .h(px(bar_height))
            .overflow_hidden()
            .bg(track_bg);
        track = apply_radius(track, self.radius);

        let mut left = 0.0_f32;
        for (index, section) in sections.into_iter().enumerate() {
            if section.value <= 0.0 {
                continue;
            }
            let width = self.width_px * (section.value / 100.0);
            let fill_color = section
                .color
                .as_ref()
                .map(|token| resolve_hsla(&self.theme, token))
                .unwrap_or(default_fill);

            let mut fill = div()
                .id(format!("{}-fill-{index}", self.id))
                .absolute()
                .left(px(left))
                .top_0()
                .w(px(width))
                .h(px(bar_height))
                .bg(fill_color);

            if self.striped {
                let stripe_color = resolve_hsla(&self.theme, &tokens.label).opacity(0.28);
                fill = fill.child(Self::striped_overlay(
                    stripe_color,
                    format!("{}-stripe-{index}", self.id),
                    width,
                    bar_height,
                    self.animated,
                ));
            }

            let fill = fill.into_any_element();

            left += width;
            track = track.child(fill);
        }

        let mut root = div().id(self.id.clone()).flex().flex_col().gap_1p5();

        if self.label.is_some() || self.show_value {
            let mut header = h_stack()
                .justify_between()
                .items_center()
                .w(px(self.width_px))
                .text_sm()
                .text_color(resolve_hsla(&self.theme, &tokens.label));

            if let Some(label) = self.label {
                header = header.child(label);
            }
            if self.show_value {
                header = header.child(format!("{total_value:.0}%"));
            }
            root = root.child(header);
        }

        root.child(track)
            .with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for Progress {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemePatchable for Progress {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}
