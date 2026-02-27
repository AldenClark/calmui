use crate::motion::MotionConfig;
use crate::style::{ComponentState, FieldLayout, Radius, Size, StyleMap, Variant};
use crate::theme::{ComponentOverrides, LocalTheme};
use gpui::{ClickEvent, FocusHandle, SharedString, Window};

pub trait StyleRecipe<Props> {
    fn resolve_styles(&self, props: &Props, state: ComponentState) -> StyleMap;
}

pub trait Varianted: std::marker::Sized {
    fn with_variant(self, value: Variant) -> Self;
}

pub trait Sized: std::marker::Sized {
    fn with_size(self, value: Size) -> Self;
}

pub trait Radiused: std::marker::Sized {
    fn with_radius(self, value: Radius) -> Self;
}

pub trait Disableable: std::marker::Sized {
    fn disabled(self, value: bool) -> Self;
}

pub trait Clickable: std::marker::Sized {
    fn on_click(self, handler: impl Fn(&ClickEvent, &mut Window, &mut gpui::App) + 'static)
    -> Self;
}

pub trait Focusable: std::marker::Sized {
    fn focus_handle(self, value: FocusHandle) -> Self;
}

pub trait WithId: std::marker::Sized {
    fn with_id(self, id: impl Into<crate::id::ComponentId>) -> Self;
}

pub trait Openable: std::marker::Sized {
    fn opened(self, value: bool) -> Self;
}

pub trait Visible: std::marker::Sized {
    fn visible(self, value: bool) -> Self;
}

pub trait Placeable<P>: std::marker::Sized {
    fn placement(self, value: P) -> Self;
}

#[macro_export]
macro_rules! impl_disableable {
    ($type:ty) => {
        impl $crate::contracts::Disableable for $type {
            fn disabled(self, value: bool) -> Self {
                <$type>::disabled(self, value)
            }
        }
    };
    ($type:ty, |$this:ident, $value:ident| $body:expr) => {
        impl $crate::contracts::Disableable for $type {
            fn disabled(mut self, value: bool) -> Self {
                let $this = &mut self;
                let $value = value;
                $body;
                self
            }
        }
    };
}

#[macro_export]
macro_rules! impl_clickable {
    ($type:ty) => {
        impl $crate::contracts::Clickable for $type {
            fn on_click(
                self,
                handler: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static,
            ) -> Self {
                <$type>::on_click(self, handler)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_focusable {
    ($type:ty) => {
        impl $crate::contracts::Focusable for $type {
            fn focus_handle(self, value: gpui::FocusHandle) -> Self {
                <$type>::focus_handle(self, value)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_with_id_for_field {
    ($type:ty, $field:ident) => {
        impl $crate::contracts::WithId for $type {
            fn with_id(mut self, id: impl Into<$crate::id::ComponentId>) -> Self {
                self.$field = id.into();
                self
            }
        }
    };
}

#[macro_export]
macro_rules! impl_default_via_new {
    ($($type:ty),+ $(,)?) => {
        $(
            impl Default for $type {
                fn default() -> Self {
                    Self::new()
                }
            }
        )+
    };
}

#[macro_export]
macro_rules! impl_component_theme_overridable {
    ($type:ty, |$this:ident| $body:expr) => {
        impl $crate::contracts::ComponentThemeOverridable for $type {
            fn local_theme_mut(&mut self) -> &mut $crate::theme::LocalTheme {
                let $this = self;
                $body
            }
        }
    };
}

#[macro_export]
macro_rules! impl_openable {
    ($type:ty) => {
        impl $crate::contracts::Openable for $type {
            fn opened(self, value: bool) -> Self {
                <$type>::opened(self, value)
            }
        }
    };
    ($type:ty, |$this:ident, $value:ident| $body:expr) => {
        impl $crate::contracts::Openable for $type {
            fn opened(mut self, value: bool) -> Self {
                let $this = &mut self;
                let $value = value;
                $body;
                self
            }
        }
    };
}

#[macro_export]
macro_rules! impl_visible {
    ($type:ty) => {
        impl $crate::contracts::Visible for $type {
            fn visible(self, value: bool) -> Self {
                <$type>::visible(self, value)
            }
        }
    };
    ($type:ty, |$this:ident, $value:ident| $body:expr) => {
        impl $crate::contracts::Visible for $type {
            fn visible(mut self, value: bool) -> Self {
                let $this = &mut self;
                let $value = value;
                $body;
                self
            }
        }
    };
}

#[macro_export]
macro_rules! impl_placeable {
    ($type:ty, $placement:ty) => {
        impl $crate::contracts::Placeable<$placement> for $type {
            fn placement(self, value: $placement) -> Self {
                <$type>::placement(self, value)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_variant_size_radius_via_methods {
    ($type:ty, $variant:ident, $size:ident, $radius:ident) => {
        impl $crate::contracts::Varianted for $type {
            fn with_variant(mut self, value: $crate::style::Variant) -> Self {
                self.$variant = value;
                self
            }
        }

        impl $crate::contracts::Sized for $type {
            fn with_size(mut self, value: $crate::style::Size) -> Self {
                self.$size = value;
                self
            }
        }

        impl $crate::contracts::Radiused for $type {
            fn with_radius(mut self, value: $crate::style::Radius) -> Self {
                self.$radius = value;
                self
            }
        }
    };
}

#[macro_export]
macro_rules! impl_varianted_via_method {
    ($type:ty) => {
        impl $crate::contracts::Varianted for $type {
            fn with_variant(self, value: $crate::style::Variant) -> Self {
                <$type>::with_variant(self, value)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_sized_via_method {
    ($type:ty) => {
        impl $crate::contracts::Sized for $type {
            fn with_size(self, value: $crate::style::Size) -> Self {
                <$type>::with_size(self, value)
            }
        }
    };
    ($type:ty, $field:ident) => {
        impl $crate::contracts::Sized for $type {
            fn with_size(mut self, value: $crate::style::Size) -> Self {
                self.$field = value;
                self
            }
        }
    };
    ($type:ty, |$this:ident, $value:ident| $body:expr) => {
        impl $crate::contracts::Sized for $type {
            fn with_size(mut self, value: $crate::style::Size) -> Self {
                let $this = &mut self;
                let $value = value;
                $body;
                self
            }
        }
    };
}

#[macro_export]
macro_rules! impl_radiused_via_method {
    ($type:ty) => {
        impl $crate::contracts::Radiused for $type {
            fn with_radius(self, value: $crate::style::Radius) -> Self {
                <$type>::with_radius(self, value)
            }
        }
    };
    ($type:ty, $field:ident) => {
        impl $crate::contracts::Radiused for $type {
            fn with_radius(mut self, value: $crate::style::Radius) -> Self {
                self.$field = value;
                self
            }
        }
    };
}

pub trait FieldLike: std::marker::Sized {
    fn label(self, value: impl Into<SharedString>) -> Self;
    fn description(self, value: impl Into<SharedString>) -> Self;
    fn error(self, value: impl Into<SharedString>) -> Self;
    fn required(self, value: bool) -> Self;
    fn layout(self, value: FieldLayout) -> Self;
}

pub trait MotionAware: std::marker::Sized {
    fn motion(self, value: MotionConfig) -> Self;
}

pub trait ComponentThemeOverridable: std::marker::Sized {
    fn local_theme_mut(&mut self) -> &mut LocalTheme;

    fn with_theme_overrides(mut self, overrides: ComponentOverrides) -> Self {
        self.local_theme_mut()
            .set_component_overrides(Some(overrides));
        self
    }

    fn theme(mut self, configure: impl FnOnce(ComponentOverrides) -> ComponentOverrides) -> Self {
        self.local_theme_mut().update_component_overrides(configure);
        self
    }

    fn clear_theme_overrides(mut self) -> Self {
        self.local_theme_mut().set_component_overrides(None);
        self
    }
}

pub trait Themable: ComponentThemeOverridable + std::marker::Sized {
    type ThemeOverrides: Default;

    fn component_overrides_mut(overrides: &mut ComponentOverrides) -> &mut Self::ThemeOverrides;

    fn themed(
        mut self,
        configure: impl FnOnce(Self::ThemeOverrides) -> Self::ThemeOverrides,
    ) -> Self {
        self.local_theme_mut()
            .update_component_overrides(|mut all| {
                let current = std::mem::take(Self::component_overrides_mut(&mut all));
                *Self::component_overrides_mut(&mut all) = configure(current);
                all
            });
        self
    }
}

#[macro_export]
macro_rules! impl_themable {
    ($type:ty, $field:ident, $overrides:ty) => {
        impl $crate::contracts::Themable for $type {
            type ThemeOverrides = $overrides;

            fn component_overrides_mut(
                overrides: &mut $crate::theme::ComponentOverrides,
            ) -> &mut Self::ThemeOverrides {
                &mut overrides.$field
            }
        }
    };
}

pub trait GpuiRenderComponent: gpui::Render {}

impl<T> GpuiRenderComponent for T where T: gpui::Render {}

pub trait GpuiRenderOnceComponent: gpui::RenderOnce {}

impl<T> GpuiRenderOnceComponent for T where T: gpui::RenderOnce {}

pub trait GpuiStyledComponent: gpui::Styled {}

impl<T> GpuiStyledComponent for T where T: gpui::Styled {}
