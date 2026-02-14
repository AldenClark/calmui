use crate::motion::MotionConfig;
use crate::style::{ComponentState, FieldLayout, Radius, Size, StyleMap, Variant};
use crate::theme::{ComponentPatch, LocalTheme};
use gpui::SharedString;

pub trait StyleRecipe<Props> {
    fn resolve_styles(&self, props: &Props, state: ComponentState) -> StyleMap;
}

pub trait VariantSupport: Sized {
    fn variant(self, value: Variant) -> Self;
    fn size(self, value: Size) -> Self;
    fn radius(self, value: Radius) -> Self;
}

pub trait Variantable: Sized {
    fn variant(self, value: Variant) -> Self;
}

impl<T> Variantable for T
where
    T: VariantSupport,
{
    fn variant(self, value: Variant) -> Self {
        VariantSupport::variant(self, value)
    }
}

pub trait Sizeable: Sized {
    fn size(self, value: Size) -> Self;
}

impl<T> Sizeable for T
where
    T: VariantSupport,
{
    fn size(self, value: Size) -> Self {
        VariantSupport::size(self, value)
    }
}

pub trait Radiusable: Sized {
    fn radius(self, value: Radius) -> Self;
}

impl<T> Radiusable for T
where
    T: VariantSupport,
{
    fn radius(self, value: Radius) -> Self {
        VariantSupport::radius(self, value)
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

pub trait ComponentThemePatchable: Sized {
    fn local_theme_mut(&mut self) -> &mut LocalTheme;

    fn with_component_theme_patch(mut self, patch: ComponentPatch) -> Self {
        self.local_theme_mut().set_component_patch(Some(patch));
        self
    }

    fn clear_component_theme_patch(mut self) -> Self {
        self.local_theme_mut().set_component_patch(None);
        self
    }
}

pub trait WithId: Sized {
    fn id(&self) -> &str;
    fn id_mut(&mut self) -> &mut String;

    fn with_id(mut self, id: impl Into<String>) -> Self {
        *self.id_mut() = id.into();
        self
    }
}

pub trait GpuiRenderComponent: gpui::Render {}

impl<T> GpuiRenderComponent for T where T: gpui::Render {}

pub trait GpuiRenderOnceComponent: gpui::RenderOnce {}

impl<T> GpuiRenderOnceComponent for T where T: gpui::RenderOnce {}

pub trait GpuiStyledComponent: gpui::Styled {}

impl<T> GpuiStyledComponent for T where T: gpui::Styled {}
