use std::{f32::consts::TAU, time::Duration};

use gpui::{
    Animation, AnimationExt, AnyElement, Component, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, Styled, div, px,
};

use crate::motion::{MotionConfig, MotionTransition, TransitionPreset};
use crate::style::Size;
use crate::{contracts::WithId, id::stable_auto_id};

use super::primitives::h_stack;
use super::transition::TransitionExt;
use super::utils::resolve_hsla;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoaderVariant {
    Dots,
    Pulse,
    Bar,
    Bars,
    Oval,
}

pub trait LoaderElement: IntoElement + WithId + Sized + 'static {
    fn size(self, size: Size) -> Self;
    fn color(self, color: impl Into<Hsla>) -> Self;
}

pub struct Loader {
    id: String,
    label: Option<SharedString>,
    variant: LoaderVariant,
    size: Size,
    color: Option<Hsla>,
    theme: crate::theme::LocalTheme,
    motion: MotionConfig,
}

impl Loader {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("loader"),
            label: None,
            variant: LoaderVariant::Dots,
            size: Size::Md,
            color: None,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::new().enter(
                MotionTransition::new()
                    .preset(TransitionPreset::Pulse)
                    .duration_ms(850)
                    .offset_px(0),
            ),
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn variant(mut self, variant: LoaderVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn motion(mut self, motion: MotionConfig) -> Self {
        self.motion = motion;
        self
    }

    fn dot_size_px(&self) -> f32 {
        match self.size {
            Size::Xs => 5.0,
            Size::Sm => 6.0,
            Size::Md => 8.0,
            Size::Lg => 10.0,
            Size::Xl => 12.0,
        }
    }

    fn ring_size_px(&self) -> f32 {
        match self.size {
            Size::Xs => 14.0,
            Size::Sm => 16.0,
            Size::Md => 20.0,
            Size::Lg => 24.0,
            Size::Xl => 28.0,
        }
    }

    fn color_token(&self) -> Hsla {
        self.color
            .clone()
            .unwrap_or_else(|| self.theme.components.button.filled_bg.clone())
    }

    fn render_dots(self) -> AnyElement {
        let color = resolve_hsla(&self.theme, &self.color_token());
        let dot = self.dot_size_px();
        let cell_h = dot * 1.8;
        let baseline_top = (cell_h - dot).max(0.0);

        let dots = (0..3).map(|index| {
            let phase = index as f32 / 3.0;
            let animation = Animation::new(Duration::from_millis(840))
                .repeat()
                .with_easing(gpui::ease_in_out);
            div()
                .id(format!("{}-dot-cell-{index}", self.id))
                .w(px(dot))
                .h(px(cell_h))
                .relative()
                .child(
                    div()
                        .id(format!("{}-dot-{index}", self.id))
                        .absolute()
                        .left_0()
                        .top(px(baseline_top))
                        .w(px(dot))
                        .h(px(dot))
                        .rounded_full()
                        .bg(color)
                        .with_animation(
                            format!("{}-dot-anim-{index}", self.id),
                            animation,
                            move |this, delta| {
                                let progress = (delta + phase).fract();
                                let wave = ((progress * TAU).sin() + 1.0) * 0.5;
                                let lift = dot * 0.6 * wave;
                                let opacity = 0.3 + (0.7 * wave);
                                this.mt(px(-lift)).opacity(opacity)
                            },
                        ),
                )
                .into_any_element()
        });

        let mut row = h_stack().items_center().gap_1().children(dots);
        if let Some(label) = self.label {
            row = row
                .gap_2()
                .child(div().text_sm().text_color(color).child(label));
        }
        row.into_any_element()
    }

    fn render_pulse(self) -> AnyElement {
        let color = resolve_hsla(&self.theme, &self.color_token());
        let dot = self.dot_size_px() + 3.0;

        let outer = div()
            .id(format!("{}-pulse-outer", self.id))
            .w(px(dot + 4.0))
            .h(px(dot + 4.0))
            .rounded_full()
            .bg(color.alpha(0.35))
            .with_repeating_transition(
                format!("{}-pulse-outer-anim", self.id),
                MotionTransition::new()
                    .preset(TransitionPreset::Pulse)
                    .duration_ms(980)
                    .offset_px(0)
                    .start_opacity_pct(12),
            );

        let inner = div()
            .id(format!("{}-pulse-inner", self.id))
            .w(px(dot))
            .h(px(dot))
            .rounded_full()
            .bg(color)
            .with_repeating_transition(
                format!("{}-pulse-inner-anim", self.id),
                MotionTransition::new()
                    .preset(TransitionPreset::Pulse)
                    .duration_ms(760)
                    .delay_ms(140)
                    .offset_px(0)
                    .start_opacity_pct(20),
            );

        let mut row = h_stack().gap_2().child(
            div()
                .relative()
                .w(px(dot + 4.0))
                .h(px(dot + 4.0))
                .child(outer)
                .child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(inner),
                ),
        );
        if let Some(label) = self.label {
            row = row.child(div().text_sm().text_color(color).child(label));
        }
        row.into_any_element()
    }

    fn render_bars(self) -> AnyElement {
        let color = resolve_hsla(&self.theme, &self.color_token());
        let (bar_w, bar_h_max) = match self.size {
            Size::Xs => (3.0, 14.0),
            Size::Sm => (4.0, 16.0),
            Size::Md => (4.0, 18.0),
            Size::Lg => (5.0, 20.0),
            Size::Xl => (6.0, 22.0),
        };
        let bar_h_min = bar_h_max * 0.35;

        let bars = (0..3).map(|index| {
            let phase = index as f32 / 3.0;
            let animation = Animation::new(Duration::from_millis(900))
                .repeat()
                .with_easing(gpui::ease_in_out);

            div()
                .h(px(bar_h_max))
                .flex()
                .items_end()
                .child(
                    div()
                        .id(format!("{}-bar-{index}", self.id))
                        .w(px(bar_w))
                        .h(px(bar_h_max))
                        .rounded_full()
                        .bg(color)
                        .with_animation(
                            format!("{}-bar-anim-{index}", self.id),
                            animation,
                            move |this, delta| {
                                let progress = (delta + phase).fract();
                                let wave = ((progress * TAU).sin() + 1.0) * 0.5;
                                let h = bar_h_min + ((bar_h_max - bar_h_min) * wave);
                                let opacity = 0.35 + (0.65 * wave);
                                this.h(px(h)).opacity(opacity)
                            },
                        ),
                )
                .into_any_element()
        });

        let mut row = h_stack().items_end().gap_1().children(bars);
        if let Some(label) = self.label {
            row = row.child(div().text_sm().text_color(color).child(label));
        }
        row.into_any_element()
    }

    fn render_oval(self) -> AnyElement {
        let color = resolve_hsla(&self.theme, &self.color_token());
        let ring = self.ring_size_px();
        let segment_size = (ring * 0.17).max(2.0);
        let segment_count = 12usize;
        let radius = (ring - segment_size) * 0.5;

        let segments = (0..segment_count).map(|index| {
            let angle = -std::f32::consts::FRAC_PI_2 + (index as f32 / segment_count as f32) * TAU;
            let x = (ring * 0.5) + radius * angle.cos() - (segment_size * 0.5);
            let y = (ring * 0.5) + radius * angle.sin() - (segment_size * 0.5);
            let phase = index as f32 / segment_count as f32;
            let animation = Animation::new(Duration::from_millis(920))
                .repeat()
                .with_easing(gpui::linear);

            div()
                .id(format!("{}-oval-segment-{index}", self.id))
                .absolute()
                .left(px(x))
                .top(px(y))
                .w(px(segment_size))
                .h(px(segment_size))
                .rounded_full()
                .bg(color)
                .with_animation(
                    format!("{}-oval-anim-{index}", self.id),
                    animation,
                    move |this, delta| {
                        let distance = (delta - phase).rem_euclid(1.0);
                        let trail = 0.42;
                        let intensity = if distance <= trail {
                            1.0 - (distance / trail)
                        } else {
                            0.0
                        };
                        this.opacity(0.16 + (0.84 * intensity))
                    },
                )
                .into_any_element()
        });

        let oval = div().relative().w(px(ring)).h(px(ring)).child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .children(segments),
        );

        let mut row = h_stack().gap_2().items_center().child(oval);
        if let Some(label) = self.label {
            row = row.child(div().text_sm().text_color(color).child(label));
        }
        row.into_any_element()
    }
}

impl WithId for Loader {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl RenderOnce for Loader {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        match self.variant {
            LoaderVariant::Dots => self.render_dots(),
            LoaderVariant::Pulse => self.render_pulse(),
            LoaderVariant::Bar | LoaderVariant::Bars => self.render_bars(),
            LoaderVariant::Oval => self.render_oval(),
        }
    }
}

impl LoaderElement for Loader {
    fn size(self, size: Size) -> Self {
        Loader::size(self, size)
    }

    fn color(self, color: impl Into<Hsla>) -> Self {
        Loader::color(self, color)
    }
}

impl IntoElement for Loader {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemePatchable for Loader {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}
