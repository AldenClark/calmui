use std::rc::Rc;

use gpui::{
    ClickEvent, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, px,
};

use crate::contracts::{MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::{MotionConfig, MotionLevel, TransitionPreset};
use crate::style::{Radius, Size, Variant};

use super::control;
use super::transition::{TransitionExt, TransitionStage};
use super::utils::{
    InteractionStyles, PressHandler, PressableBehavior, apply_interaction_styles, apply_radius,
    interaction_style, resolve_hsla, wire_pressable,
};

type ChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut gpui::App)>;

pub struct SegmentedControlItem {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl SegmentedControlItem {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}

#[derive(IntoElement)]
pub struct SegmentedControl {
    id: ComponentId,
    items: Vec<SegmentedControlItem>,
    value: Option<SharedString>,
    value_controlled: bool,
    default_value: Option<SharedString>,
    full_width: bool,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<ChangeHandler>,
}

impl SegmentedControl {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            items: Vec::new(),
            value: None,
            value_controlled: false,
            default_value: None,
            full_width: false,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Md,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
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

        control::optional_text_state(
            &self.id,
            "value",
            self.value_controlled
                .then_some(self.value.as_ref().map(|value| value.to_string())),
            default.map(|value| value.to_string()),
        )
        .map(SharedString::from)
    }

    fn apply_item_size<T: Styled>(size: Size, node: T) -> T {
        match size {
            Size::Xs => node.text_xs().py_1().px_2(),
            Size::Sm => node.text_sm().py_1().px_2p5(),
            Size::Md => node.text_base().py_1p5().px_3(),
            Size::Lg => node.text_lg().py_2().px_3p5(),
            Size::Xl => node.text_xl().py_2p5().px_4(),
        }
    }

    fn indicator_inset_px(size: Size) -> f32 {
        match size {
            Size::Xs => 0.5,
            Size::Sm => 1.0,
            Size::Md => 1.0,
            Size::Lg => 1.5,
            Size::Xl => 1.5,
        }
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

impl SegmentedControl {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for SegmentedControl {
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
        let size = self.size;
        let _full_width = self.full_width;
        let theme = self.theme.clone();
        let on_change = self.on_change.clone();
        let controlled = self.value_controlled;
        let control_id = self.id.clone();
        let root_id = self.id.clone();
        let enter_id = self.id.clone();
        let motion = self.motion;
        let divider = resolve_hsla(&theme, &tokens.border).alpha(0.6);
        let divider_width = super::utils::quantized_stroke_px(window, 1.0);
        let transparent = resolve_hsla(&theme, &gpui::transparent_black());
        let indicator_inset = Self::indicator_inset_px(size);
        let selected_index = selected.as_ref().and_then(|value| {
            self.items
                .iter()
                .position(|item| item.value.as_ref() == value.as_ref())
        });
        let previous_index = control::optional_text_state(&self.id, "prev-index", None, None)
            .and_then(|value| value.parse::<usize>().ok());
        let divider_height = match size {
            Size::Xs => 12.0,
            Size::Sm => 14.0,
            Size::Md => 16.0,
            Size::Lg => 18.0,
            Size::Xl => 20.0,
        };

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
                        resolve_hsla(&theme, &tokens.item_disabled_fg)
                    } else if is_active {
                        resolve_hsla(&theme, &tokens.item_active_fg)
                    } else {
                        resolve_hsla(&theme, &tokens.item_fg)
                    })
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
                            .child(
                                div()
                                    .w(divider_width)
                                    .h(gpui::px(divider_height))
                                    .bg(divider),
                            ),
                    );
                }

                segment = segment.child(div().relative().truncate().child(item.label.clone()));

                segment = Self::apply_item_size(size, segment);
                segment = apply_radius(&self.theme, segment, self.radius);

                if is_active {
                    segment = segment.shadow_sm();
                }

                if !item.disabled {
                    let on_change = on_change.clone();
                    let value = item.value.clone();
                    let id = control_id.clone();
                    let previous = selected_index.map(|value| value.to_string());
                    let hover_bg = resolve_hsla(&theme, &tokens.item_hover_bg);
                    let press_bg = hover_bg.blend(gpui::black().opacity(0.08));
                    let focus_bg = if is_active {
                        active_bg.blend(gpui::white().opacity(0.04))
                    } else {
                        hover_bg
                    };
                    let click_handler: PressHandler = Rc::new(move |_: &ClickEvent, window, cx| {
                        control::set_optional_text_state(&id, "prev-index", previous.clone());
                        if !controlled {
                            control::set_optional_text_state(&id, "value", Some(value.to_string()));
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
                    segment = wire_pressable(
                        segment,
                        PressableBehavior::new().on_click(Some(click_handler)),
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
            .gap_0()
            .p_0p5()
            .bg(resolve_hsla(&theme, &tokens.bg))
            .children(items);

        track = apply_radius(&self.theme, track, self.radius);

        div()
            .id(root_id)
            .flex()
            .items_center()
            .justify_start()
            .child(track)
            .with_enter_transition(enter_id.slot("enter"), motion)
    }
}

impl crate::contracts::ComponentThemeOverridable for SegmentedControl {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(SegmentedControlItem);

impl gpui::Styled for SegmentedControl {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
