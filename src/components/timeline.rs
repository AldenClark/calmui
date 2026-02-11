use gpui::{
    AnyElement, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled, div, px,
};

use crate::contracts::{MotionAware, VariantSupport, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::icon::Icon;
use super::primitives::{h_stack, v_stack};
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;

pub struct TimelineItem {
    pub title: SharedString,
    pub body: Option<SharedString>,
    pub marker_icon: Option<SharedString>,
    content: Option<SlotRenderer>,
}

impl TimelineItem {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            body: None,
            marker_icon: None,
            content: None,
        }
    }

    pub fn body(mut self, value: impl Into<SharedString>) -> Self {
        self.body = Some(value.into());
        self
    }

    pub fn marker_icon(mut self, value: impl Into<SharedString>) -> Self {
        self.marker_icon = Some(value.into());
        self
    }

    pub fn content(mut self, value: impl IntoElement + 'static) -> Self {
        self.content = Some(Box::new(|| value.into_any_element()));
        self
    }
}

pub struct Timeline {
    id: String,
    items: Vec<TimelineItem>,
    active: usize,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    motion: MotionConfig,
}

impl Timeline {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("timeline"),
            items: Vec::new(),
            active: 0,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
        }
    }

    pub fn item(mut self, item: TimelineItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TimelineItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn active(mut self, value: usize) -> Self {
        self.active = value;
        self
    }

    fn bullet_size_px(&self) -> f32 {
        match self.size {
            Size::Xs => 14.0,
            Size::Sm => 16.0,
            Size::Md => 18.0,
            Size::Lg => 22.0,
            Size::Xl => 26.0,
        }
    }

    fn line_width_px(&self) -> f32 {
        match self.size {
            Size::Xs | Size::Sm => 1.0,
            Size::Md => 2.0,
            Size::Lg | Size::Xl => 3.0,
        }
    }

    fn active_bullet_bg(&self) -> gpui::Hsla {
        let base = resolve_hsla(
            &self.theme,
            &self.theme.components.timeline.bullet_active_bg,
        );
        match self.variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.82),
            Variant::Subtle => base.alpha(0.72),
            Variant::Outline => base.alpha(0.9),
            Variant::Ghost => base.alpha(0.64),
        }
    }
}

impl WithId for Timeline {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantSupport for Timeline {
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

impl MotionAware for Timeline {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Timeline {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = self.theme.components.timeline.clone();
        let theme = self.theme.clone();
        let bullet_size = self.bullet_size_px();
        let line_width = self.line_width_px();
        let active_bullet_bg = self.active_bullet_bg();
        let total_items = self.items.len();
        let active = self.active.min(total_items.saturating_sub(1));

        if total_items == 0 {
            return v_stack()
                .id(self.id.clone())
                .w_full()
                .child(
                    div()
                        .text_color(resolve_hsla(&theme, &tokens.body))
                        .child("No timeline items"),
                )
                .with_enter_transition(format!("{}-enter", self.id), self.motion);
        }

        let mut rows = v_stack().id(self.id.clone()).w_full().gap_2();
        for (index, mut item) in self.items.into_iter().enumerate() {
            let is_done = index < active;
            let is_current = index == active;
            let is_active_marker = is_done || is_current;
            let has_next = index < total_items.saturating_sub(1);

            let mut bullet = div()
                .id(format!("{}-bullet-{index}", self.id))
                .w(px(bullet_size))
                .h(px(bullet_size))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .border_1()
                .border_color(if is_active_marker {
                    resolve_hsla(&theme, &tokens.bullet_active_border)
                } else {
                    resolve_hsla(&theme, &tokens.bullet_border)
                })
                .bg(if is_active_marker {
                    active_bullet_bg
                } else {
                    resolve_hsla(&theme, &tokens.bullet_bg)
                })
                .text_color(if is_active_marker {
                    resolve_hsla(&theme, &tokens.bullet_active_fg)
                } else {
                    resolve_hsla(&theme, &tokens.bullet_fg)
                });
            bullet = apply_radius(bullet, self.radius);

            if let Some(icon) = item.marker_icon.take() {
                bullet = bullet.child(
                    Icon::named_outline(icon.to_string())
                        .with_id(format!("{}-bullet-icon-{index}", self.id))
                        .size((bullet_size * 0.56).max(10.0))
                        .color(if is_active_marker {
                            resolve_hsla(&theme, &tokens.bullet_active_fg)
                        } else {
                            resolve_hsla(&theme, &tokens.bullet_fg)
                        }),
                );
            } else if is_done {
                bullet = bullet.child("✓");
            } else {
                bullet = bullet.child("•");
            }

            let mut left_col = v_stack().items_center().gap_0().child(bullet);
            if has_next {
                left_col = left_col.child(div().w(px(line_width)).h(px(34.0)).bg(if is_done {
                    resolve_hsla(&theme, &tokens.line_active)
                } else {
                    resolve_hsla(&theme, &tokens.line)
                }));
            }

            let mut right_col = v_stack().gap_1().min_w_0().child(
                div()
                    .text_color(if is_current {
                        resolve_hsla(&theme, &tokens.title_active)
                    } else {
                        resolve_hsla(&theme, &tokens.title)
                    })
                    .font_weight(if is_current {
                        gpui::FontWeight::SEMIBOLD
                    } else {
                        gpui::FontWeight::NORMAL
                    })
                    .child(item.title),
            );
            if let Some(body) = item.body {
                right_col = right_col.child(
                    div()
                        .text_sm()
                        .text_color(resolve_hsla(&theme, &tokens.body))
                        .child(body),
                );
            }
            if let Some(content) = item.content.take() {
                let mut content_wrap = div()
                    .mt_1()
                    .p_2()
                    .border_1()
                    .border_color(resolve_hsla(&theme, &tokens.card_border))
                    .bg(resolve_hsla(&theme, &tokens.card_bg))
                    .child(content());
                content_wrap = apply_radius(content_wrap, Radius::Sm);
                right_col = right_col.child(content_wrap);
            }

            rows = rows.child(
                h_stack()
                    .id(format!("{}-item-{index}", self.id))
                    .items_start()
                    .gap_2()
                    .w_full()
                    .child(left_col)
                    .child(right_col),
            );
        }

        rows.with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for Timeline {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemePatchable for Timeline {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}
