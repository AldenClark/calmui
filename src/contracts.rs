use crate::motion::MotionConfig;
use crate::style::{ComponentState, FieldLayout, Radius, Size, StyleMap, Variant};
use crate::theme::{ComponentOverrides, LocalTheme};
use gpui::SharedString;

pub trait StyleRecipe<Props> {
    fn resolve_styles(&self, props: &Props, state: ComponentState) -> StyleMap;
}

pub(crate) trait VariantConfigurable: Sized {
    fn variant(self, value: Variant) -> Self;
    fn size(self, value: Size) -> Self;
    fn radius(self, value: Radius) -> Self;
}

pub trait Variantable: Sized {
    fn variant(self, value: Variant) -> Self;
}

impl<T> Variantable for T
where
    T: VariantConfigurable,
{
    fn variant(self, value: Variant) -> Self {
        VariantConfigurable::variant(self, value)
    }
}

pub trait Sizeable: Sized {
    fn size(self, value: Size) -> Self;
}

impl<T> Sizeable for T
where
    T: VariantConfigurable,
{
    fn size(self, value: Size) -> Self {
        VariantConfigurable::size(self, value)
    }
}

pub trait Radiusable: Sized {
    fn radius(self, value: Radius) -> Self;
}

impl<T> Radiusable for T
where
    T: VariantConfigurable,
{
    fn radius(self, value: Radius) -> Self {
        VariantConfigurable::radius(self, value)
    }
}

pub trait Disableable: Sized {
    fn disabled(self, value: bool) -> Self;
}

pub trait Openable: Sized {
    fn opened(self, value: bool) -> Self;
}

pub trait Visible: Sized {
    fn visible(self, value: bool) -> Self;
}

pub trait Placeable<P>: Sized {
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
            fn variant(self, value: $crate::style::Variant) -> Self {
                <$type>::variant(self, value)
            }

            fn size(self, value: $crate::style::Size) -> Self {
                <$type>::size(self, value)
            }

            fn radius(self, value: $crate::style::Radius) -> Self {
                <$type>::radius(self, value)
            }
        }
    };
}

pub trait FieldLike: Sized {
    fn label(self, value: impl Into<SharedString>) -> Self;
    fn description(self, value: impl Into<SharedString>) -> Self;
    fn error(self, value: impl Into<SharedString>) -> Self;
    fn required(self, value: bool) -> Self;
    fn layout(self, value: FieldLayout) -> Self;
}

pub trait MotionAware: Sized {
    fn motion(self, value: MotionConfig) -> Self;
}

pub trait ComponentThemeOverridable: Sized {
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

pub trait Themable: ComponentThemeOverridable + Sized {
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
