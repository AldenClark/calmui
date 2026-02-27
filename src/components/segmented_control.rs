use std::rc::Rc;

use gpui::InteractiveElement;
use gpui::{IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, px};

use crate::contracts::{FieldLike, MotionAware};
use crate::id::ComponentId;
use crate::motion::{MotionConfig, MotionLevel, TransitionPreset};
use crate::style::{FieldLayout, Radius, Size, Variant};

use super::Stack;
use super::interaction_adapter::{ActivateHandler, PressAdapter, bind_press_adapter};
use super::selection_state;
use super::transition::{TransitionExt, TransitionStage};
use super::utils::{
    InteractionStyles, apply_interaction_styles, apply_radius, interaction_style, resolve_hsla,
};

type ChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;

pub struct SegmentedControlItem {
    pub value: SharedString,
    pub label: Option<SharedString>,
    pub disabled: bool,
}

impl SegmentedControlItem {
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: None,
            disabled: false,
        }
    }

    pub fn labeled(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self::new(value).label(label)
    }

    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
    }
}

#[derive(IntoElement)]
pub struct SegmentedControl {
    pub(crate) id: ComponentId,
    items: Vec<SegmentedControlItem>,
    label: Option<SharedString>,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    layout: FieldLayout,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: Option<SharedString>,
    full_width: bool,
    variant: Variant,
    size: Size,
    radius: Radius,
    pub(crate) theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_change: Option<ChangeHandler>,
}

impl SegmentedControl {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            items: Vec::new(),
            label: None,
            description: None,
            error: None,
            required: false,
            layout: FieldLayout::Vertical,
            value: None,
            value_controlled: false,
            default_value: None,
            full_width: false,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Md,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn item(mut self, item: SegmentedControlItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SegmentedControlItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
    }

    pub fn description(mut self, value: impl Into<SharedString>) -> Self {
        self.description = Some(value.into());
        self
    }

    pub fn error(mut self, value: impl Into<SharedString>) -> Self {
        self.error = Some(value.into());
        self
    }

    pub fn required(mut self, value: bool) -> Self {
        self.required = value;
        self
    }

    pub fn layout(mut self, value: FieldLayout) -> Self {
        self.layout = value;
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self.value_controlled = true;
        self
    }

    pub fn clear_value(mut self) -> Self {
        self.value = None;
        self.value_controlled = true;
        self
    }

    pub fn default_value(mut self, value: impl Into<SharedString>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    pub fn full_width(mut self, value: bool) -> Self {
        self.full_width = value;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn resolved_value(&self) -> Option<SharedString> {
        let default = self
            .default_value
            .clone()
            .or_else(|| self.items.first().map(|item| item.value.clone()));

        selection_state::resolve_optional_text(
            &self.id,
            "value",
            self.value_controlled,
            self.value.as_ref().map(|value| value.to_string()),
            default.map(|value| value.to_string()),
        )
        .map(SharedString::from)
    }

    fn size_preset(&self) -> crate::theme::SegmentedControlSizePreset {
        self.theme
            .components
            .segmented_control
            .sizes
            .for_size(self.size)
    }

    fn active_bg(&self) -> gpui::Hsla {
        let token = &self.theme.components.segmented_control.item_active_bg;
        let base = resolve_hsla(&self.theme, token);
        match self.variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.8),
            Variant::Subtle => base.alpha(0.72),
            Variant::Outline => base.alpha(0.9),
            Variant::Ghost => base.alpha(0.64),
        }
    }
}

impl SegmentedControl {}

crate::impl_variant_size_radius_via_methods!(SegmentedControl, variant, size, radius);

impl MotionAware for SegmentedControl {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for SegmentedControl {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = self.theme.components.segmented_control.clone();
        let selected = self.resolved_value();
        let active_bg = self.active_bg();
        let size_preset = self.size_preset();
        let full_width = self.full_width;
        let theme = self.theme.clone();
        let on_change = self.on_change.clone();
        let controlled = self.value_controlled;
        let control_id = self.id.clone();
        let root_id = self.id.clone();
        let enter_id = self.id.clone();
        let motion = self.motion;
        let divider = resolve_hsla(&theme, tokens.border).alpha(0.6);
        let divider_width = super::utils::quantized_stroke_px(window, 1.0);
        let transparent = resolve_hsla(&theme, gpui::transparent_black());
        let indicator_inset = f32::from(size_preset.indicator_inset);
        let selected_index = selected.as_ref().and_then(|value| {
            self.items
                .iter()
                .position(|item| item.value.as_ref() == value.as_ref())
        });
        let previous_index =
            selection_state::resolve_optional_usize(&self.id, "prev-index", None, None);
        let divider_height = size_preset.divider_height;

        let items = self
            .items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let is_active = selected
                    .as_ref()
                    .is_some_and(|value| value.as_ref() == item.value.as_ref());
                let show_divider = index > 0
                    && !selected_index
                        .is_some_and(|active| active == index || active.saturating_add(1) == index);

                let mut segment = div()
                    .id(self.id.slot_index("item", index.to_string()))
                    .relative()
                    .overflow_hidden()
                    .flex()
                    .items_center()
                    .justify_center()
                    .min_w_0()
                    .font_weight(if is_active {
                        gpui::FontWeight::SEMIBOLD
                    } else {
                        gpui::FontWeight::MEDIUM
                    })
                    .text_color(if item.disabled {
                        resolve_hsla(&theme, tokens.item_disabled_fg)
                    } else if is_active {
                        resolve_hsla(&theme, tokens.item_active_fg)
                    } else {
                        resolve_hsla(&theme, tokens.item_fg)
                    })
                    .text_size(size_preset.font_size)
                    .line_height(size_preset.line_height)
                    .py(size_preset.padding_y)
                    .px(size_preset.padding_x)
                    .bg(transparent);

                if is_active {
                    let indicator = div()
                        .id(self.id.slot_index("indicator", index.to_string()))
                        .absolute()
                        .left(px(indicator_inset))
                        .top(px(indicator_inset))
                        .right(px(indicator_inset))
                        .bottom(px(indicator_inset))
                        .bg(active_bg);

                    let mut profile = motion.enter;
                    profile.preset = match (previous_index, selected_index) {
                        (Some(prev), Some(current)) if current < prev => TransitionPreset::FadeLeft,
                        _ => TransitionPreset::FadeRight,
                    };
                    profile.offset_px = profile.offset_px.max(12);
                    profile.start_opacity_pct = profile.start_opacity_pct.max(40);
                    profile.duration_ms = profile.duration_ms.max(180);
                    if motion.level == MotionLevel::None {
                        profile.preset = TransitionPreset::None;
                        profile.offset_px = 0;
                        profile.start_opacity_pct = 100;
                        profile.duration_ms = 1;
                    }

                    segment = segment.child(
                        apply_radius(&self.theme, indicator, self.radius).with_transition_profile(
                            self.id.slot_index("indicator-enter", index.to_string()),
                            profile,
                            TransitionStage::Enter,
                        ),
                    );
                }

                if show_divider {
                    segment = segment.child(
                        div()
                            .absolute()
                            .left_0()
                            .top_0()
                            .bottom_0()
                            .flex()
                            .items_center()
                            .child(div().w(divider_width).h(divider_height).bg(divider)),
                    );
                }

                if let Some(label) = item.label.clone() {
                    segment = segment.child(div().relative().truncate().child(label));
                }

                if full_width {
                    segment = segment.flex_1();
                }

                segment = apply_radius(&self.theme, segment, self.radius);

                if is_active {
                    segment = segment.shadow_sm();
                }

                if !item.disabled {
                    let on_change = on_change.clone();
                    let value = item.value.clone();
                    let id = control_id.clone();
                    let previous = selected_index;
                    let hover_bg = resolve_hsla(&theme, tokens.item_hover_bg);
                    let press_bg = hover_bg.blend(gpui::black().opacity(0.08));
                    let focus_bg = if is_active {
                        active_bg.blend(gpui::white().opacity(0.04))
                    } else {
                        hover_bg
                    };
                    let activate_handler: ActivateHandler = Rc::new(move |window, cx| {
                        let _ = selection_state::apply_optional_usize(
                            &id,
                            "prev-index",
                            false,
                            previous,
                        );
                        if selection_state::apply_optional_text(
                            &id,
                            "value",
                            controlled,
                            Some(value.to_string()),
                        ) {
                            window.refresh();
                        }
                        if let Some(handler) = on_change.as_ref() {
                            (handler)(value.clone(), window, cx);
                        }
                    });

                    let mut interaction_styles = InteractionStyles::new()
                        .focus(interaction_style(move |style| style.bg(focus_bg)));
                    if !is_active {
                        interaction_styles = interaction_styles
                            .hover(interaction_style(move |style| style.bg(hover_bg)))
                            .active(interaction_style(move |style| style.bg(press_bg)));
                    }
                    segment =
                        apply_interaction_styles(segment.cursor_pointer(), interaction_styles);
                    segment = bind_press_adapter(
                        segment,
                        PressAdapter::new(self.id.slot_index("item", index.to_string()))
                            .on_activate(Some(activate_handler)),
                    );
                } else {
                    segment = segment.opacity(0.5).cursor_default();
                }

                segment
            })
            .collect::<Vec<_>>();

        let mut track = div()
            .id(root_id.slot("track"))
            .flex()
            .items_center()
            .gap(tokens.item_gap)
            .p(tokens.track_padding)
            .bg(resolve_hsla(&theme, tokens.bg))
            .children(items);
        if full_width {
            track = track.w_full();
        }

        track = apply_radius(&self.theme, track, self.radius);

        let track = track.into_any_element();
        let label = self.label.map(|value| {
            if self.required {
                SharedString::from(format!("{value} *"))
            } else {
                value
            }
        });
        let has_meta = label.is_some() || self.description.is_some() || self.error.is_some();
        if !has_meta {
            let mut root = div()
                .id(root_id)
                .flex()
                .items_center()
                .justify_start()
                .child(track);
            if full_width {
                root = root.w_full();
            }
            return root.with_enter_transition(enter_id.slot("enter"), motion);
        }

        let mut meta = Stack::vertical().gap(tokens.item_gap);
        if let Some(label) = label {
            meta = meta.child(
                div()
                    .text_color(resolve_hsla(&theme, tokens.item_fg))
                    .child(label),
            );
        }
        if let Some(description) = self.description {
            meta = meta.child(
                div()
                    .text_color(resolve_hsla(&theme, tokens.item_fg).alpha(0.78))
                    .child(description),
            );
        }
        if let Some(error) = self.error {
            meta = meta.child(
                div()
                    .text_color(resolve_hsla(&theme, theme.semantic.status_error))
                    .child(error),
            );
        }

        let content = match self.layout {
            FieldLayout::Vertical => Stack::vertical()
                .gap(tokens.item_gap)
                .child(meta)
                .child(track)
                .into_any_element(),
            FieldLayout::Horizontal => Stack::horizontal()
                .items_start()
                .gap(tokens.item_gap)
                .child(meta)
                .child(track)
                .into_any_element(),
        };

        let mut root = div()
            .id(root_id)
            .flex()
            .items_center()
            .justify_start()
            .child(content);
        if full_width {
            root = root.w_full();
        }
        root.with_enter_transition(enter_id.slot("enter"), motion)
    }
}

crate::impl_disableable!(SegmentedControlItem, |this, value| this.disabled = value);

impl FieldLike for SegmentedControl {
    fn label(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
    }

    fn description(mut self, value: impl Into<SharedString>) -> Self {
        self.description = Some(value.into());
        self
    }

    fn error(mut self, value: impl Into<SharedString>) -> Self {
        self.error = Some(value.into());
        self
    }

    fn required(mut self, value: bool) -> Self {
        self.required = value;
        self
    }

    fn layout(mut self, value: FieldLayout) -> Self {
        self.layout = value;
        self
    }
}
