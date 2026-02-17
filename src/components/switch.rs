use std::rc::Rc;

use gpui::{
    Component, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::contracts::{MotionAware, VariantConfigurable, WithId};
use crate::id::stable_auto_id;
use crate::motion::MotionConfig;
use crate::style::{Radius, Size, Variant};

use super::Stack;
use super::control;
use super::transition::TransitionExt;
use super::utils::{apply_radius, resolve_hsla};

type SwitchChangeHandler = Rc<dyn Fn(bool, &mut Window, &mut gpui::App)>;

pub struct Switch {
    id: String,
    label: SharedString,
    description: Option<SharedString>,
    checked: Option<bool>,
    default_checked: bool,
    disabled: bool,
    size: Size,
    radius: Radius,
    theme: crate::theme::LocalTheme,
    style: gpui::StyleRefinement,
    motion: MotionConfig,
    on_change: Option<SwitchChangeHandler>,
}

impl Switch {
    #[track_caller]
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            id: stable_auto_id("switch"),
            label: label.into(),
            description: None,
            checked: None,
            default_checked: false,
            disabled: false,
            size: Size::Md,
            radius: Radius::Pill,
            theme: crate::theme::LocalTheme::default(),
            style: gpui::StyleRefinement::default(),
            motion: MotionConfig::default(),
            on_change: None,
        }
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

    fn switch_dimensions(&self) -> (f32, f32) {
        match self.size {
            Size::Xs => (26.0, 14.0),
            Size::Sm => (30.0, 16.0),
            Size::Md => (36.0, 20.0),
            Size::Lg => (42.0, 24.0),
            Size::Xl => (48.0, 28.0),
        }
    }

    fn resolved_checked(&self) -> bool {
        control::bool_state(&self.id, "checked", self.checked, self.default_checked)
    }
}

impl WithId for Switch {
    fn id(&self) -> &str {
        &self.id
    }

    fn id_mut(&mut self) -> &mut String {
        &mut self.id
    }
}

impl VariantConfigurable for Switch {
    fn variant(self, _value: Variant) -> Self {
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

impl MotionAware for Switch {
    fn motion(mut self, value: MotionConfig) -> Self {
        self.motion = value;
        self
    }
}

impl RenderOnce for Switch {
    fn render(mut self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        self.theme.sync_from_provider(_cx);
        let checked = self.resolved_checked();
        let is_controlled = self.checked.is_some();
        let (track_w, track_h) = self.switch_dimensions();
        let thumb_size = (track_h - 4.0).max(8.0);
        let thumb_inset = ((track_h - thumb_size) / 2.0).max(1.0);
        let thumb_top = (thumb_inset - 0.5).max(0.0);
        let thumb_left = if checked {
            track_w - thumb_size - thumb_inset
        } else {
            thumb_inset
        };

        let tokens = &self.theme.components.switch;
        let active = resolve_hsla(&self.theme, &tokens.track_on_bg);
        let inactive = resolve_hsla(&self.theme, &tokens.track_off_bg);
        let track_bg = if checked { active } else { inactive };
        let label_fg = resolve_hsla(&self.theme, &tokens.label);
        let description_fg = resolve_hsla(&self.theme, &tokens.description);

        let mut thumb = div()
            .absolute()
            .left(px(thumb_left))
            .top(px(thumb_top))
            .w(px(thumb_size))
            .h(px(thumb_size))
            .bg(resolve_hsla(&self.theme, &tokens.thumb_bg));
        thumb = apply_radius(&self.theme, thumb, Radius::Pill);

        let mut track = div()
            .relative()
            .w(px(track_w))
            .h(px(track_h))
            .border_1()
            .border_color(track_bg)
            .bg(track_bg)
            .child(thumb);
        track = apply_radius(&self.theme, track, self.radius);

        let mut row = Stack::horizontal()
            .id(self.id.clone())
            .cursor_pointer()
            .child(
                Stack::vertical()
                    .gap_0p5()
                    .child(
                        Stack::horizontal()
                            .items_center()
                            .gap_2()
                            .child(track)
                            .child(div().text_color(label_fg).child(self.label)),
                    )
                    .children(self.description.map(|description| {
                        div()
                            .ml(px(track_w + 8.0))
                            .text_sm()
                            .text_color(description_fg)
                            .child(description)
                    })),
            );

        if self.disabled {
            row = row.cursor_default().opacity(0.55);
        } else if let Some(handler) = self.on_change.clone() {
            let id = self.id.clone();
            row = row.on_click(move |_, window, cx| {
                let next = !checked;
                if !is_controlled {
                    control::set_bool_state(&id, "checked", next);
                    window.refresh();
                }
                (handler)(next, window, cx);
            });
        } else if !is_controlled {
            let id = self.id.clone();
            row = row.on_click(move |_, window, _cx| {
                control::set_bool_state(&id, "checked", !checked);
                window.refresh();
            });
        }

        row.with_enter_transition(format!("{}-enter", self.id), self.motion)
    }
}

impl IntoElement for Switch {
    type Element = Component<Self>;

    fn into_element(self) -> Self::Element {
        Component::new(self)
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
