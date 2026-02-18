use crate::motion::MotionConfig;
use crate::style::{ComponentState, FieldLayout, Radius, Size, StyleMap, Variant};
use crate::theme::{ComponentOverrides, LocalTheme};
use gpui::{ClickEvent, FocusHandle, SharedString, Window};

pub trait StyleRecipe<Props> {
    fn resolve_styles(&self, props: &Props, state: ComponentState) -> StyleMap;
}

pub(crate) trait VariantConfigurable: std::marker::Sized {
    fn with_variant(self, value: Variant) -> Self;
    fn with_size(self, value: Size) -> Self;
    fn with_radius(self, value: Radius) -> Self;
}

pub trait Varianted: std::marker::Sized {
    fn with_variant(self, value: Variant) -> Self;
}

impl<T> Varianted for T
where
    T: VariantConfigurable,
{
    fn with_variant(self, value: Variant) -> Self {
        VariantConfigurable::with_variant(self, value)
    }
}

pub trait Sized: std::marker::Sized {
    fn with_size(self, value: Size) -> Self;
}

impl<T> Sized for T
where
    T: VariantConfigurable,
{
    fn with_size(self, value: Size) -> Self {
        VariantConfigurable::with_size(self, value)
    }
}

pub trait Radiused: std::marker::Sized {
    fn with_radius(self, value: Radius) -> Self;
}

impl<T> Radiused for T
where
    T: VariantConfigurable,
{
    fn with_radius(self, value: Radius) -> Self {
        VariantConfigurable::with_radius(self, value)
    }
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
macro_rules! impl_openable {
    ($type:ty) => {
        impl $crate::contracts::Openable for $type {
            fn opened(self, value: bool) -> Self {
                <$type>::opened(self, value)
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
    ($type:ty) => {
        impl $crate::contracts::VariantConfigurable for $type {
            fn with_variant(self, value: $crate::style::Variant) -> Self {
                <$type>::with_variant(self, value)
            }

            fn with_size(self, value: $crate::style::Size) -> Self {
                <$type>::with_size(self, value)
            }

            fn with_radius(self, value: $crate::style::Radius) -> Self {
                <$type>::with_radius(self, value)
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
