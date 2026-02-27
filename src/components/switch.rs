use std::rc::Rc;

use gpui::InteractiveElement;
use gpui::{
    IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled,
    Window, div, px,
};

use crate::contracts::{FieldLike, MotionAware};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{FieldLayout, Radius, Size, Variant};

use super::Stack;
use super::control;
use super::toggle::{ToggleConfig, wire_toggle_handlers};
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla, snap_px};

type SwitchChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SwitchLabelPosition {
    Left,
    Right,
}

#[derive(IntoElement)]
pub struct Switch {
    pub(crate) id: ComponentId,
    label: Option<SharedString>,
    label_position: SwitchLabelPosition,
    description: Option<SharedString>,
    error: Option<SharedString>,
    required: bool,
    layout: FieldLayout,
    checked: Option<bool>,
    default_checked: bool,
    disabled: bool,
    variant: Variant,
    size: Size,
    radius: Radius,
    pub(crate) theme: crate::theme::LocalTheme,
    motion: MotionConfig,
    on_change: Option<SwitchChangeHandler>,
}

impl Switch {
    #[track_caller]
    pub fn new() -> Self {
        Self {
            id: ComponentId::default(),
            label: None,
            label_position: SwitchLabelPosition::Right,
            description: None,
            error: None,
            required: false,
            layout: FieldLayout::Vertical,
            checked: None,
            default_checked: false,
            disabled: false,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn label_position(mut self, value: SwitchLabelPosition) -> Self {
        self.label_position = value;
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
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

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn default_checked(mut self, checked: bool) -> Self {
        self.default_checked = checked;
        self
    }
    pub fn on_change(
        mut self,
        handler: impl Fn(bool, &mut Window, &mut gpui::App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    fn resolved_checked(&self) -> bool {
        control::bool_state(&self.id, "checked", self.checked, self.default_checked)
    }

    fn variant_track_color(&self, base: gpui::Hsla) -> gpui::Hsla {
        match self.variant {
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.85),
            Variant::Subtle => base.alpha(0.72),
            Variant::Outline => base.alpha(0.9),
            Variant::Ghost => base.alpha(0.6),
        }
    }

    fn variant_inactive_track_color(&self, base: gpui::Hsla) -> gpui::Hsla {
        match self.variant {
            Variant::Outline | Variant::Ghost => gpui::transparent_black(),
            Variant::Filled | Variant::Default => base,
            Variant::Light => base.alpha(0.85),
            Variant::Subtle => base.alpha(0.72),
        }
    }
}

impl Switch {}

crate::impl_variant_size_radius_via_methods!(Switch, variant, size, radius);

impl MotionAware for Switch {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Switch {
    fn render(mut self, window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let checked = self.resolved_checked();
        let is_controlled = self.checked.is_some();
        let is_focused = control::focused_state(&self.id, None, false);
        let tokens = &self.theme.components.switch;
        let size_preset = tokens.sizes.for_size(self.size);
        let track_w = f32::from(size_preset.track_width);
        let track_h = f32::from(size_preset.track_height);
        let thumb_size = f32::from(size_preset.thumb_size);
        // Snap thumb offsets to device pixels to avoid sub-pixel drift on scaled displays.
        let thumb_inset = f32::from(snap_px(window, ((track_h - thumb_size) * 0.5).max(0.0)));
        let active = self.variant_track_color(resolve_hsla(&self.theme, tokens.track_on_bg));
        let inactive =
            self.variant_inactive_track_color(resolve_hsla(&self.theme, tokens.track_off_bg));
        let track_bg = if checked { active } else { inactive };
        let label_fg = resolve_hsla(&self.theme, tokens.label);
        let description_fg = resolve_hsla(&self.theme, tokens.description);
        let error_fg = resolve_hsla(&self.theme, self.theme.semantic.status_error);
        let description_indent = match self.label_position {
            SwitchLabelPosition::Left => 0.0,
            SwitchLabelPosition::Right => track_w + f32::from(size_preset.description_indent_gap),
        };

        let mut thumb = div()
            .w(px(thumb_size))
            .h(px(thumb_size))
            .bg(resolve_hsla(&self.theme, tokens.thumb_bg));
        thumb = apply_radius(&self.theme, thumb, Radius::Pill);

        let mut track = div()
            .flex()
            .items_center()
            .px(px(thumb_inset))
            .w(px(track_w))
            .h(px(track_h))
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(if is_focused {
                self.variant_track_color(resolve_hsla(&self.theme, tokens.track_focus_border))
            } else {
                track_bg
            })
            .bg(track_bg)
            .child(thumb);
        track = if checked {
            track.justify_end()
        } else {
            track.justify_start()
        };
        track = apply_radius(&self.theme, track, self.radius);
        if !self.disabled {
            let hover_border =
                self.variant_track_color(resolve_hsla(&self.theme, tokens.track_hover_border));
            track = track.hover(move |style| style.border_color(hover_border));
        }

        let label_text = self.label.clone().map(|label| {
            if self.required {
                SharedString::from(format!("{label} *"))
            } else {
                label
            }
        });

        let switch_with_label = match self.label_position {
            SwitchLabelPosition::Left => Stack::horizontal()
                .items_center()
                .gap(size_preset.label_gap)
                .children(label_text.clone().map(|label| {
                    div()
                        .text_size(size_preset.label_size)
                        .text_color(label_fg)
                        .child(label)
                }))
                .child(track),
            SwitchLabelPosition::Right => Stack::horizontal()
                .items_center()
                .gap(size_preset.label_gap)
                .child(track)
                .children(label_text.map(|label| {
                    div()
                        .text_size(size_preset.label_size)
                        .text_color(label_fg)
                        .child(label)
                })),
        };

        let support_block = if self.description.is_some() || self.error.is_some() {
            let mut block = Stack::vertical().gap(tokens.label_description_gap);
            if let Some(description) = self.description.clone() {
                let description_row = div()
                    .text_size(size_preset.description_size)
                    .text_color(description_fg)
                    .child(description);
                if self.layout == FieldLayout::Vertical {
                    block = block.child(description_row.ml(px(description_indent)));
                } else {
                    block = block.child(description_row);
                }
            }
            if let Some(error) = self.error.clone() {
                let error_row = div()
                    .text_size(size_preset.description_size)
                    .text_color(error_fg)
                    .child(error);
                if self.layout == FieldLayout::Vertical {
                    block = block.child(error_row.ml(px(description_indent)));
                } else {
                    block = block.child(error_row);
                }
            }
            Some(block.into_any_element())
        } else {
            None
        };

        let content = match self.layout {
            FieldLayout::Vertical => {
                let mut block = Stack::vertical()
                    .gap(tokens.label_description_gap)
                    .child(switch_with_label);
                if let Some(support) = support_block {
                    block = block.child(support);
                }
                block.into_any_element()
            }
            FieldLayout::Horizontal => {
                let mut row = Stack::horizontal()
                    .items_start()
                    .gap(tokens.label_description_gap)
                    .child(switch_with_label);
                if let Some(support) = support_block {
                    row = row.child(support);
                }
                row.into_any_element()
            }
        };

        let mut row = div()
            .id(self.id.clone())
            .flex()
            .flex_row()
            .focusable()
            .cursor_pointer()
            .child(content);

        if self.disabled {
            row = row.cursor_default().opacity(0.55);
        } else {
            row = wire_toggle_handlers(
                row,
                ToggleConfig {
                    id: self.id.clone(),
                    checked,
                    controlled: is_controlled,
                    allow_uncheck: true,
                    on_change: self.on_change.clone(),
                },
            );
        }

        row.with_enter_transition(self.id.slot("enter"), self.motion)
    }
}

crate::impl_disableable!(Switch, |this, value| this.disabled = value);

impl FieldLike for Switch {
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
