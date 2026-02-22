use std::rc::Rc;

use gpui::{
    InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, VariantConfigurable};
use crate::id::ComponentId;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

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
    id: ComponentId,
    label: Option<SharedString>,
    label_position: SwitchLabelPosition,
    description: Option<SharedString>,
    checked: Option<bool>,
    default_checked: bool,
    disabled: bool,
    variant: Variant,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
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
            checked: None,
            default_checked: false,
            disabled: false,
            variant: Variant::Default,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
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

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn default_checked(mut self, checked: bool) -> Self {
        self.default_checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

impl Switch {
    pub fn with_id(mut self, id: impl Into<ComponentId>) -> Self {
        self.id = id.into();
        self
    }
}

impl VariantConfigurable for Switch {
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
        let active = self.variant_track_color(resolve_hsla(&self.theme, &tokens.track_on_bg));
        let inactive =
            self.variant_inactive_track_color(resolve_hsla(&self.theme, &tokens.track_off_bg));
        let track_bg = if checked { active } else { inactive };
        let label_fg = resolve_hsla(&self.theme, &tokens.label);
        let description_fg = resolve_hsla(&self.theme, &tokens.description);
        let description_indent = match self.label_position {
            SwitchLabelPosition::Left => 0.0,
            SwitchLabelPosition::Right => track_w + f32::from(size_preset.description_indent_gap),
        };

        let mut thumb = div()
            .w(px(thumb_size))
            .h(px(thumb_size))
            .bg(resolve_hsla(&self.theme, &tokens.thumb_bg));
        thumb = apply_radius(&self.theme, thumb, Radius::Pill);

        let mut track = div()
            .flex()
            .items_center()
            .px(px(thumb_inset))
            .w(px(track_w))
            .h(px(track_h))
            .border(super::utils::quantized_stroke_px(window, 1.0))
            .border_color(if is_focused {
                self.variant_track_color(resolve_hsla(&self.theme, &tokens.track_focus_border))
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
                self.variant_track_color(resolve_hsla(&self.theme, &tokens.track_hover_border));
            track = track.hover(move |style| style.border_color(hover_border));
        }

        let switch_with_label = match self.label_position {
            SwitchLabelPosition::Left => Stack::horizontal()
                .items_center()
                .gap(size_preset.label_gap)
                .children(self.label.clone().map(|label| {
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
                .children(self.label.clone().map(|label| {
                    div()
                        .text_size(size_preset.label_size)
                        .text_color(label_fg)
                        .child(label)
                })),
        };

        let mut row = Stack::horizontal()
            .id(self.id.clone())
            .focusable()
            .cursor_pointer()
            .child(
                Stack::vertical()
                    .gap(tokens.label_description_gap)
                    .child(switch_with_label)
                    .children(self.description.map(|description| {
                        div()
                            .ml(px(description_indent))
                            .text_size(size_preset.description_size)
                            .text_color(description_fg)
                            .child(description)
                    })),
            );

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

impl crate::contracts::ComponentThemeOverridable for Switch {
    fn local_theme_mut(&mut self) -> &mut crate::theme::LocalTheme {
        &mut self.theme
    }
}

crate::impl_disableable!(Switch);

impl gpui::Styled for Switch {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
