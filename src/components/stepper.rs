use std::rc::Rc;

use gpui::{
    AnyElement, Bounds, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, canvas, div, fill, point, px, size,
};

use crate::contracts::{MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{GroupOrientation, Radius, Size, Variant};

use super::Stack;
use super::interaction_adapter::{ActivateHandler, PressAdapter, bind_press_adapter};
use super::selection_state;
use super::transition::TransitionExt;
use super::utils::{apply_radius, quantized_stroke_px, resolve_hsla, snap_px};

type SlotRenderer = Box<dyn FnOnce() -> AnyElement>;
type ChangeHandler = Rc<dyn Fn(usize, SharedString, &mut Window, &mut gpui::App)>;

pub struct StepperStep {
    pub value: SharedString,
    pub label: Option<SharedString>,
    pub description: Option<SharedString>,
    pub disabled: bool,
    content: Option<SlotRenderer>,
}

impl StepperStep {
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: None,
            description: None,
            disabled: false,
            content: None,
        }
    }

    pub fn labeled(mut self, value: impl Into<SharedString>) -> Self {
        self.label = Some(value.into());
        self
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

#[derive(IntoElement)]
pub struct Stepper {
    id: ComponentId,
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
            id: ComponentId::default(),
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
        let controlled = self.active.unwrap_or(self.default_active).min(max_index);
        let default = self.default_active.min(max_index);
        selection_state::resolve_usize(
            &self.id,
            "active",
            self.active_controlled,
            controlled,
            default,
        )
        .min(max_index)
    }

    fn size_preset(&self) -> crate::theme::StepperSizePreset {
        self.theme.components.stepper.sizes.for_size(self.size)
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

impl Stepper {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for Stepper {
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
        let size_preset = self.size_preset();
        let theme = self.theme.clone();
        let active = self.resolved_active();
        let indicator_size = f32::from(size_preset.indicator_size);
        let connector_thickness = f32::from(size_preset.connector_thickness);
        let connector_span = f32::from(size_preset.connector_span);
        let connector_thickness_px = quantized_stroke_px(window, connector_thickness);
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
                    "✓".to_string()
                } else {
                    (index + 1).to_string()
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
                    .id(self.id.slot_index("indicator", index.to_string()))
                    .w(px(indicator_size))
                    .h(px(indicator_size))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .border(quantized_stroke_px(window, 1.0))
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

                let mut text_block = Stack::vertical().gap(tokens.text_gap);
                if let Some(label) = step.label.clone() {
                    text_block = text_block.child(
                        div()
                            .text_size(size_preset.label_size)
                            .text_color(resolve_hsla(&theme, &tokens.label))
                            .font_weight(if is_active {
                                gpui::FontWeight::SEMIBOLD
                            } else {
                                gpui::FontWeight::NORMAL
                            })
                            .child(label),
                    );
                }
                if let Some(description) = step.description.clone() {
                    text_block = text_block.child(
                        div()
                            .text_size(size_preset.description_size)
                            .text_color(resolve_hsla(&theme, &tokens.description))
                            .child(description),
                    );
                }

                let mut item = match self.orientation {
                    GroupOrientation::Horizontal => match self.content_position {
                        StepperContentPosition::Below => Stack::vertical()
                            .id(self.id.slot_index("step", index.to_string()))
                            .items_center()
                            .gap(size_preset.item_gap_vertical)
                            .p(size_preset.item_padding)
                            .flex_1()
                            .min_w_0()
                            .child(indicator)
                            .child(text_block),
                        StepperContentPosition::Right => Stack::horizontal()
                            .id(self.id.slot_index("step", index.to_string()))
                            .items_center()
                            .gap(size_preset.item_gap_horizontal)
                            .p(size_preset.item_padding)
                            .flex_1()
                            .min_w_0()
                            .child(indicator)
                            .child(text_block.min_w_0()),
                    },
                    GroupOrientation::Vertical => Stack::horizontal()
                        .id(self.id.slot_index("step", index.to_string()))
                        .items_start()
                        .gap(size_preset.item_gap_horizontal)
                        .p(size_preset.item_padding)
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
                    let activate_handler: ActivateHandler = Rc::new(move |window, cx| {
                        if selection_state::apply_usize(&id, "active", controlled, index) {
                            window.refresh();
                        }
                        if let Some(handler) = on_change.as_ref() {
                            (handler)(index, step_value.clone(), window, cx);
                        }
                    });
                    item = item.cursor_pointer().hover(move |style| style.bg(hover_bg));
                    item = bind_press_adapter(
                        item,
                        PressAdapter::new(self.id.slot_index("step", index.to_string()))
                            .on_activate(Some(activate_handler)),
                    );
                }

                item
            })
            .collect::<Vec<_>>();

        let step_count = step_nodes.len();
        let mut steps_view = match self.orientation {
            GroupOrientation::Horizontal => {
                let connector_gap = connector_span.max(0.0);
                let connector_colors = (0..step_count.saturating_sub(1))
                    .map(|index| {
                        if index < active {
                            resolve_hsla(&theme, &tokens.step_completed_border)
                        } else {
                            resolve_hsla(&theme, &tokens.connector)
                        }
                    })
                    .collect::<Vec<_>>();
                let mut row = Stack::horizontal()
                    .id(self.id.slot("steps-row"))
                    .relative()
                    .w_full()
                    .items_start()
                    .gap(px(connector_gap));
                for node in step_nodes {
                    row = row.child(node);
                }
                if !connector_colors.is_empty() && connector_gap > 0.0 {
                    let indicator_center_y =
                        f32::from(size_preset.item_padding) + indicator_size * 0.5;
                    row = row.child(
                        canvas(
                            |_, _, _| {},
                            move |bounds, _, window, _| {
                                let count = connector_colors.len() + 1;
                                if count < 2 {
                                    return;
                                }

                                let total_width = f32::from(bounds.size.width).max(0.0);
                                let total_gap = connector_gap * connector_colors.len() as f32;
                                if total_width <= total_gap {
                                    return;
                                }

                                let item_width = (total_width - total_gap) / count as f32;
                                let thickness =
                                    f32::from(quantized_stroke_px(window, connector_thickness));
                                if thickness <= 0.0 {
                                    return;
                                }

                                let line_top = f32::from(snap_px(
                                    window,
                                    indicator_center_y - thickness * 0.5,
                                ));
                                for (index, color) in connector_colors.iter().enumerate() {
                                    let start = item_width * (index as f32 + 1.0)
                                        + connector_gap * index as f32;
                                    let end = start + connector_gap;
                                    let x0 = f32::from(snap_px(window, start));
                                    let x1 = f32::from(snap_px(window, end));
                                    let width = (x1 - x0).max(0.0);
                                    if width <= 0.0 {
                                        continue;
                                    }
                                    window.paint_quad(fill(
                                        Bounds::new(
                                            point(
                                                bounds.origin.x + px(x0),
                                                bounds.origin.y + px(line_top),
                                            ),
                                            size(px(width), px(thickness)),
                                        ),
                                        *color,
                                    ));
                                }
                            },
                        )
                        .absolute()
                        .size_full(),
                    );
                }
                row
            }
            GroupOrientation::Vertical => {
                let mut col = Stack::vertical()
                    .id(self.id.slot("steps-col"))
                    .w_full()
                    .gap(tokens.steps_gap_vertical);
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
                                .w(connector_thickness_px)
                                .h(px(connector_span))
                                .bg(connector_color),
                        );
                    }
                }
                col
            }
        };

        if step_count == 0 {
            steps_view = div()
                .id(self.id.slot("steps-empty"))
                .text_color(resolve_hsla(&theme, &tokens.description))
                .child("No steps");
        }

        let mut panel = div()
            .id(self.id.slot("panel"))
            .w_full()
            .mt(tokens.panel_margin_top)
            .p(size_preset.panel_padding)
            .border(quantized_stroke_px(window, 1.0))
            .border_color(resolve_hsla(&theme, &tokens.panel_border))
            .bg(resolve_hsla(&theme, &tokens.panel_bg))
            .text_color(resolve_hsla(&theme, &tokens.panel_fg));
        panel = apply_radius(&self.theme, panel, self.radius);
        panel = panel.child(panel_content.unwrap_or_else(|| {
            if let Some((label, description)) = active_step_meta.clone() {
                let mut fallback = Stack::vertical().gap(tokens.text_gap);
                if let Some(label) = label {
                    fallback = fallback.child(
                        div()
                            .text_size(size_preset.label_size)
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .child(label),
                    );
                }
                if let Some(description) = description {
                    fallback = fallback.child(
                        div()
                            .text_size(size_preset.description_size)
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
            .gap(tokens.root_gap)
            .child(steps_view)
            .child(panel)
            .with_enter_transition(self.id.slot("enter"), self.motion)
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
