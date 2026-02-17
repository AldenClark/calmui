use std::rc::Rc;

use gpui::{
    AnyElement, ClickEvent, Component, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, VariantConfigurable, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{GroupOrientation, Radius, Size, Variant};

use super::Stack;
use super::control;
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type ChangeHandler = Rc<dyn Fn(usize, SharedString, &mut Window, &mut gpui::App)>;

pub struct StepperStep {
    pub value: SharedString,
    pub label: SharedString,
    pub description: Option<SharedString>,
    pub disabled: bool,
    content: Option<SlotRenderer>,
}

impl StepperStep {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            disabled: false,
            content: None,
        }
    }

    pub fn description(mut self, value: impl Into<SharedString>) -> Self {
        self.description = Some(value.into());
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn content(mut self, value: impl IntoElement + 'static) -> Self {
        self.content = Some(Box::new(|| value.into_any_element()));
        self
    }
}

pub struct Stepper {
    id: String,
    steps: Vec<StepperStep>,
    active: Option<usize>,
    active_controlled: bool,
    default_active: usize,
    orientation: GroupOrientation,
    content_position: StepperContentPosition,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<ChangeHandler>,
}

impl Stepper {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: stable_auto_id("stepper"),
            steps: Vec::new(),
            active: None,
            active_controlled: false,
            default_active: 0,
            orientation: GroupOrientation::Horizontal,
            content_position: StepperContentPosition::Right,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn step(mut self, step: StepperStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn steps(mut self, steps: impl IntoIterator<Item = StepperStep>) -> Self {
        self.steps.extend(steps);
        self
    }

    pub fn active(mut self, value: usize) -> Self {
        self.active = Some(value);
        self.active_controlled = true;
        self
    }

    pub fn default_active(mut self, value: usize) -> Self {
        self.default_active = value;
        self
    }

    pub fn orientation(mut self, value: GroupOrientation) -> Self {
        self.orientation = value;
        self
    }

    pub fn content_position(mut self, value: StepperContentPosition) -> Self {
        self.content_position = value;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(usize, SharedString, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn resolved_active(&self) -> usize {
        let max_index = self.steps.len().saturating_sub(1);
        let controlled = self.active_controlled.then_some(
            self.active
                .unwrap_or(self.default_active)
                .min(max_index)
                .to_string(),
        );
        let default = self.default_active.min(max_index).to_string();
        control::text_state(&self.id, "active", controlled, default)
            .parse::<usize>()
            .ok()
            .unwrap_or(0)
            .min(max_index)
    }

    fn indicator_size_px(&self) -> f32 {
        match self.size {
            Size::Xs => 18.0,
            Size::Sm => 20.0,
            Size::Md => 24.0,
            Size::Lg => 28.0,
            Size::Xl => 32.0,
        }
    }

    fn line_thickness_px(&self) -> f32 {
        match self.size {
            Size::Xs | Size::Sm => 1.0,
            Size::Md => 2.0,
            Size::Lg | Size::Xl => 3.0,
        }
    }

    fn connector_span_px(&self) -> f32 {
        match self.size {
            Size::Xs => 20.0,
            Size::Sm => 24.0,
            Size::Md => 28.0,
            Size::Lg => 32.0,
            Size::Xl => 36.0,
        }
    }

    fn active_bg(&self) -> gpui::Hsla {
        let base = resolve_hsla(&self.theme, &self.theme.components.stepper.step_active_bg);
        match self.variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.82),
            Variant::Subtle => base.alpha(0.72),
            Variant::Outline => base.alpha(0.9),
            Variant::Ghost => base.alpha(0.64),
        }
    }
}

impl WithId for Stepper {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantConfigurable for Stepper {
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

impl MotionAware for Stepper {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepperContentPosition {
    Below,
    Right,
}

impl RenderOnce for Stepper {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let tokens = self.theme.components.stepper.clone();
        let theme = self.theme.clone();
        let active = self.resolved_active();
        let indicator_size = self.indicator_size_px();
        let connector_thickness = self.line_thickness_px();
        let connector_span = self.connector_span_px();
        let on_change = self.on_change.clone();
        let active_bg = self.active_bg();
        let controlled = self.active_controlled;
        let stepper_id = self.id.clone();
        let active_step_meta = self
            .steps
            .get(active)
            .map(|step| (step.label.clone(), step.description.clone()));

        let mut panel_content: Option<AnyElement> = None;
        if let Some(active_step) = self.steps.get_mut(active) {
            if let Some(content) = active_step.content.take() {
                panel_content = Some(content());
            }
        }

        let step_nodes = self
            .steps
            .into_iter()
            .enumerate()
            .map(|(index, step)| {
                let is_completed = index < active;
                let is_active = index == active;

                let indicator_text = if is_completed {
                    "✓".into_any_element()
                } else {
                    div().child((index + 1).to_string()).into_any_element()
                };

                let (indicator_bg, indicator_border, indicator_fg) = if is_completed {
                    (
                        resolve_hsla(&theme, &tokens.step_completed_bg),
                        resolve_hsla(&theme, &tokens.step_completed_border),
                        resolve_hsla(&theme, &tokens.step_completed_fg),
                    )
                } else if is_active {
                    (
                        active_bg,
                        resolve_hsla(&theme, &tokens.step_active_border),
                        resolve_hsla(&theme, &tokens.step_active_fg),
                    )
                } else {
                    (
                        resolve_hsla(&theme, &tokens.step_bg),
                        resolve_hsla(&theme, &tokens.step_border),
                        resolve_hsla(&theme, &tokens.step_fg),
                    )
                };

                let mut indicator = div()
                    .id(format!("{}-indicator-{index}", self.id))
                    .w(px(indicator_size))
                    .h(px(indicator_size))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .border(super::utils::quantized_stroke_px(window, 1.0))
                    .border_color(indicator_border)
                    .bg(indicator_bg)
                    .text_color(indicator_fg)
                    .font_weight(if is_active {
                        gpui::FontWeight::BOLD
                    } else {
                        gpui::FontWeight::SEMIBOLD
                    })
                    .child(indicator_text);
                indicator = apply_radius(&self.theme, indicator, Radius::Pill);

                let mut text_block = Stack::vertical().gap_0p5().child(
                    div()
                        .text_color(resolve_hsla(&theme, &tokens.label))
                        .font_weight(if is_active {
                            gpui::FontWeight::SEMIBOLD
                        } else {
                            gpui::FontWeight::NORMAL
                        })
                        .child(step.label.clone()),
                );
                if let Some(description) = step.description.clone() {
                    text_block = text_block.child(
                        div()
                            .text_sm()
                            .text_color(resolve_hsla(&theme, &tokens.description))
                            .child(description),
                    );
                }

                let mut item = match self.orientation {
                    GroupOrientation::Horizontal => match self.content_position {
                        StepperContentPosition::Below => Stack::vertical()
                            .id(format!("{}-step-{index}", self.id))
                            .items_center()
                            .gap_1()
                            .p_1()
                            .flex_1()
                            .min_w_0()
                            .child(indicator)
                            .child(text_block),
                        StepperContentPosition::Right => Stack::horizontal()
                            .id(format!("{}-step-{index}", self.id))
                            .items_center()
                            .gap_2()
                            .p_1()
                            .flex_1()
                            .min_w_0()
                            .child(indicator)
                            .child(text_block.min_w_0()),
                    },
                    GroupOrientation::Vertical => Stack::horizontal()
                        .id(format!("{}-step-{index}", self.id))
                        .items_start()
                        .gap_2()
                        .p_1()
                        .w_full()
                        .child(indicator)
                        .child(text_block),
                };
                item = apply_radius(&self.theme, item, self.radius);

                if step.disabled {
                    item = item.opacity(0.55).cursor_default();
                } else {
                    let on_change = on_change.clone();
                    let id = stepper_id.clone();
                    let step_value = step.value.clone();
                    let hover_bg = resolve_hsla(&theme, &tokens.step_bg).alpha(0.6);
                    item = item
                        .cursor_pointer()
                        .hover(move |style| style.bg(hover_bg))
                        .on_click(move |_: &ClickEvent, window, cx| {
                            if !controlled {
                                control::set_text_state(&id, "active", index.to_string());
                                window.refresh();
                            }
                            if let Some(handler) = on_change.as_ref() {
                                (handler)(index, step_value.clone(), window, cx);
                            }
                        });
                }

                item.into_any_element()
            })
            .collect::<Vec<_>>();

        let step_count = step_nodes.len();
        let mut steps_view = match self.orientation {
            GroupOrientation::Horizontal => {
                let mut row = Stack::horizontal()
                    .id(format!("{}-steps-row", self.id))
                    .w_full()
                    .items_start();
                for (index, node) in step_nodes.into_iter().enumerate() {
                    row = row.child(node);
                    if index < step_count.saturating_sub(1) {
                        let connector_color = if index < active {
                            resolve_hsla(&theme, &tokens.step_completed_border)
                        } else {
                            resolve_hsla(&theme, &tokens.connector)
                        };
                        row = row.child(
                            div()
                                .mt(px(indicator_size * 0.5))
                                .w(px(connector_span))
                                .h(px(connector_thickness))
                                .bg(connector_color),
                        );
                    }
                }
                row.into_any_element()
            }
            GroupOrientation::Vertical => {
                let mut col = Stack::vertical()
                    .id(format!("{}-steps-col", self.id))
                    .w_full()
                    .gap_1p5();
                for (index, node) in step_nodes.into_iter().enumerate() {
                    col = col.child(node);
                    if index < step_count.saturating_sub(1) {
                        let connector_color = if index < active {
                            resolve_hsla(&theme, &tokens.step_completed_border)
                        } else {
                            resolve_hsla(&theme, &tokens.connector)
                        };
                        col = col.child(
                            div()
                                .ml(px(indicator_size * 0.5))
                                .w(px(connector_thickness))
                                .h(px(connector_span))
                                .bg(connector_color),
                        );
                    }
                }
                col.into_any_element()
            }
        };

        if step_count == 0 {
            steps_view = div()
                .text_color(resolve_hsla(&theme, &tokens.description))
                .child("No steps")
                .into_any_element();
        }

        let mut panel = div()
            .id(format!("{}-panel", self.id))
            .w_full()
            .mt_2()
            .p_3()
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&theme, &tokens.panel_border))
            .bg(resolve_hsla(&theme, &tokens.panel_bg))
            .text_color(resolve_hsla(&theme, &tokens.panel_fg));
        panel = apply_radius(&self.theme, panel, self.radius);
        panel = panel.child(panel_content.unwrap_or_else(|| {
            if let Some((label, description)) = active_step_meta.clone() {
                let mut fallback = Stack::vertical()
                    .gap_1()
                    .child(div().font_weight(gpui::FontWeight::MEDIUM).child(label));
                if let Some(description) = description {
                    fallback = fallback.child(
                        div()
                            .text_sm()
                            .text_color(resolve_hsla(&theme, &tokens.description))
                            .child(description),
                    );
                }
                fallback.into_any_element()
            } else {
                div()
                    .text_color(resolve_hsla(&theme, &tokens.description))
                    .child("No step content")
                    .into_any_element()
            }
        }));

        Stack::vertical()
            .id(self.id.clone())
            .w_full()
            .gap_1p5()
            .child(steps_view)
            .child(panel)
            .with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for Stepper {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
    }
}

impl crate::contracts::ComponentThemeOverridable for Stepper {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(StepperStep);

impl gpui::Styled for Stepper {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
