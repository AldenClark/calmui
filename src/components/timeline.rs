use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    div, px,
};

use crate::contracts::{MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::Stack;
use super::icon::Icon;
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;

pub struct TimelineItem {
    pub title: Option<SharedString>,
    pub body: Option<SharedString>,
    pub marker_icon: Option<SharedString>,
    content: Option<SlotRenderer>,
}

impl TimelineItem {
    pub fn new() -> Self {
        Self {
            title: None,
            body: None,
            marker_icon: None,
            content: None,
        }
    }

    pub fn titled(title: impl Into<SharedString>) -> Self {
        Self::new().title(title)
    }

    pub fn title(mut self, value: impl Into<SharedString>) -> Self {
        self.title = Some(value.into());
        self
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

#[derive(IntoElement)]
pub struct Timeline {
    id: ComponentId,
    items: Vec<TimelineItem>,
    active: usize,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
}

impl Timeline {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            items: Vec::new(),
            active: 0,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
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

    fn size_preset(&self) -> crate::theme::TimelineSizePreset {
        self.theme.components.timeline.sizes.for_size(self.size)
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

impl Timeline {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for Timeline {
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

impl MotionAware for Timeline {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Timeline {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = self.theme.components.timeline.clone();
        let size_preset = self.size_preset();
        let theme = self.theme.clone();
        let bullet_size = f32::from(size_preset.bullet_size);
        let line_width = f32::from(size_preset.line_width);
        let active_bullet_bg = self.active_bullet_bg();
        let total_items = self.items.len();
        let active = self.active.min(total_items.saturating_sub(1));

        if total_items == 0 {
            return Stack::vertical()
                .id(self.id.clone())
                .w_full()
                .child(
                    div()
                        .text_color(resolve_hsla(&theme, &tokens.body))
                        .child("No timeline items"),
                )
                .with_enter_transition(self.id.slot("enter"), self.motion);
        }

        let mut rows = Stack::vertical()
            .id(self.id.clone())
            .w_full()
            .gap(tokens.root_gap);
        for (index, mut item) in self.items.into_iter().enumerate() {
            let is_done = index < active;
            let is_current = index == active;
            let is_active_marker = is_done || is_current;
            let has_next = index < total_items.saturating_sub(1);

            let mut bullet = div()
                .id(self.id.slot_index("bullet", index.to_string()))
                .w(px(bullet_size))
                .h(px(bullet_size))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .border(super::utils::quantized_stroke_px(window, 1.0))
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
            bullet = apply_radius(&self.theme, bullet, self.radius);

            if let Some(icon) = item.marker_icon.take() {
                bullet = bullet.child(
                    Icon::named(icon.to_string())
                        .with_id(self.id.slot_index("bullet-icon", index.to_string()))
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

            let mut left_col = div()
                .w(px(bullet_size))
                .h_full()
                .min_h(px(bullet_size + f32::from(tokens.line_extra_height)))
                .flex()
                .flex_col()
                .items_center()
                .child(bullet);
            if has_next {
                left_col = left_col.child(
                    div()
                        .mt(px(0.0))
                        .w(px(line_width))
                        .flex_1()
                        .min_h(tokens.line_min_height)
                        .bg(if is_done {
                            resolve_hsla(&theme, &tokens.line_active)
                        } else {
                            resolve_hsla(&theme, &tokens.line)
                        }),
                );
            }

            let mut right_col =
                Stack::vertical()
                    .gap(tokens.content_gap)
                    .min_w_0()
                    .child(div().children(item.title.map(|title| {
                        div()
                            .text_size(size_preset.title_size)
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
                            .child(title)
                    })));
            if let Some(body) = item.body {
                right_col = right_col.child(
                    div()
                        .text_size(size_preset.body_size)
                        .text_color(resolve_hsla(&theme, &tokens.body))
                        .child(body),
                );
            }
            if let Some(content) = item.content.take() {
                let mut content_wrap = div()
                    .mt(tokens.card_margin_top)
                    .p(size_preset.card_padding)
                    .border(super::utils::quantized_stroke_px(window, 1.0))
                    .border_color(resolve_hsla(&theme, &tokens.card_border))
                    .bg(resolve_hsla(&theme, &tokens.card_bg))
                    .child(content());
                content_wrap = apply_radius(&self.theme, content_wrap, Radius::Sm);
                right_col = right_col.child(content_wrap);
            }

            rows = rows.child(
                Stack::horizontal()
                    .id(self.id.slot_index("item", index.to_string()))
                    .items_start()
                    .gap(tokens.row_gap)
                    .py(tokens.row_padding_y)
                    .w_full()
                    .child(left_col)
                    .child(right_col),
            );
        }

        rows.with_enter_transition(self.id.slot("enter"), self.motion)
    }
}

impl crate::contracts::ComponentThemeOverridable for Timeline {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

impl gpui::Styled for Timeline {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
