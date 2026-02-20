use std::time::Duration;

use gpui::{
    Animation, AnimationExt, AnyElement, Bounds, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, Styled, canvas, div, fill, point, px, size,
};

use crate::contracts::{MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::Stack;
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla, snap_px};

#[derive(Clone, Debug, PartialEq)]
pub struct ProgressSection {
    value: f32,
    color: Option<Hsla>,
}

impl ProgressSection {
    pub fn new(value: f32) -> Self {
        Self { value, color: None }
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

#[derive(IntoElement)]
pub struct Progress {
    id: ComponentId,
    value: f32,
    sections: Vec<ProgressSection>,
    label: Option<SharedString>,
    show_value: bool,
    striped: bool,
    animated: bool,
    width_px: Option<f32>,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
}

impl Progress {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            value: 0.0,
            sections: Vec::new(),
            label: None,
            show_value: false,
            striped: false,
            animated: false,
            width_px: None,
            variant: Variant::Filled,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
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
        self.width_px = Some(width_px.max(0.0));
        self
    }

    fn normalized_value(value: f32) -> f32 {
        value.clamp(0.0, 100.0)
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
        key: ComponentId,
        filled_ranges: Vec<(f32, f32)>,
        width_px: f32,
        bar_height: f32,
        animated: bool,
    ) -> AnyElement {
        if filled_ranges.is_empty() || width_px <= 0.0 {
            return div().id(key).absolute().size_full().into_any_element();
        }

        let stripe_width = (bar_height * 1.55).clamp(6.0, 14.0);
        let stripe_step = stripe_width * 1.7;
        let overlay_width = if animated { width_px * 2.0 } else { width_px };
        let overlay_offset = if animated { width_px } else { 0.0 };
        let stripe_ranges = filled_ranges;

        let stripe_canvas = canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                if stripe_width <= 0.0 || stripe_step <= 0.0 {
                    return;
                }

                for (range_left, range_width) in &stripe_ranges {
                    if *range_width <= 0.0 {
                        continue;
                    }

                    let range_start = overlay_offset + *range_left;
                    let range_end = range_start + *range_width;
                    let mut x = range_start - stripe_step;
                    let mut stripe_index = 0usize;

                    while x < range_end + stripe_step {
                        let stripe_start = x.max(range_start);
                        let stripe_end = (x + stripe_width).min(range_end);
                        let snapped_start = f32::from(snap_px(window, stripe_start));
                        let snapped_end = f32::from(snap_px(window, stripe_end));
                        let snapped_width = (snapped_end - snapped_start).max(0.0);
                        if snapped_width > 0.0 {
                            let alpha = match stripe_index % 3 {
                                0 => 0.18,
                                1 => 0.1,
                                _ => 0.05,
                            };
                            window.paint_quad(fill(
                                Bounds::new(
                                    point(bounds.origin.x + px(snapped_start), bounds.origin.y),
                                    size(px(snapped_width), bounds.size.height),
                                ),
                                color.alpha(alpha),
                            ));
                        }
                        stripe_index += 1;
                        x += stripe_step;
                    }
                }
            },
        )
        .absolute()
        .size_full();

        if animated {
            let animation_ms = (2800.0 + width_px * 4.0).clamp(3200.0, 5200.0) as u64;
            let move_key = key.slot("move");
            div()
                .id(key)
                .absolute()
                .top_0()
                .bottom_0()
                .left(px(-width_px))
                .w(px(overlay_width))
                .overflow_hidden()
                .child(stripe_canvas)
                .with_animation(
                    move_key,
                    Animation::new(Duration::from_millis(animation_ms))
                        .repeat()
                        .with_easing(gpui::ease_in_out),
                    move |this, delta| {
                        let eased = gpui::ease_in_out(delta);
                        this.left(px(-width_px + width_px * eased))
                    },
                )
                .into_any_element()
        } else {
            div()
                .id(key)
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .overflow_hidden()
                .child(stripe_canvas)
                .into_any_element()
        }
    }
}

impl Progress {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for Progress {
    fn with_variant(mut self, value: Variant) -> Self {
        self.variant = value;
        self
    }

    fn with_size(mut self, value: Size) -> Self {
        self.size = value;
        self
    }

    fn with_radius(mut self, value: Radius) -> Self {
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
        let size_preset = tokens.sizes.for_size(self.size);
        let track_width = self
            .width_px
            .unwrap_or_else(|| f32::from(tokens.default_width))
            .max(f32::from(tokens.min_width));
        let track_bg = resolve_hsla(&self.theme, &tokens.track_bg);
        let default_fill = self.variant_fill_color();
        let sections = self.resolved_sections();
        let bar_height = f32::from(size_preset.bar_height);
        let total_value = sections
            .iter()
            .fold(0.0_f32, |acc, section| acc + section.value);

        let mut track = div()
            .id(self.id.slot("track"))
            .relative()
            .w(px(track_width))
            .h(px(bar_height))
            .overflow_hidden()
            .bg(track_bg);
        track = apply_radius(&self.theme, track, self.radius);

        let mut left = 0.0_f32;
        let mut filled_ranges = Vec::new();
        for (index, section) in sections.into_iter().enumerate() {
            if section.value <= 0.0 {
                continue;
            }
            let width = track_width * (section.value / 100.0);
            let fill_color = section
                .color
                .as_ref()
                .map(|token| resolve_hsla(&self.theme, token))
                .unwrap_or(default_fill);

            let fill = div()
                .id(self.id.slot_index("fill", index.to_string()))
                .absolute()
                .left(px(left))
                .top_0()
                .w(px(width))
                .h(px(bar_height))
                .bg(fill_color);
            filled_ranges.push((left, width));

            left += width;
            track = track.child(fill);
        }
        if self.striped && !filled_ranges.is_empty() {
            let stripe_color = resolve_hsla(&self.theme, &tokens.label).opacity(0.28);
            track = track.child(Self::striped_overlay(
                stripe_color,
                ComponentId::from(self.id.slot("stripe-overlay")),
                filled_ranges,
                track_width,
                bar_height,
                self.animated,
            ));
        }

        let mut root = Stack::vertical().id(self.id.clone()).gap(tokens.root_gap);

        if self.label.is_some() || self.show_value {
            let mut header = Stack::horizontal()
                .justify_between()
                .items_center()
                .w(px(track_width))
                .text_size(size_preset.label_size)
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
            .with_enter_transition(self.id.slot("enter"), self.motion)
    }
}

impl crate::contracts::ComponentThemeOverridable for Progress {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl gpui::Styled for Progress {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
