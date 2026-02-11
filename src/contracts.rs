use crate::motion::MotionConfig;
use crate::style::{ComponentState, FieldLayout, Radius, Size, StyleMap, Variant};
use crate::theme::Theme;
use gpui::SharedString;

pub trait ThemeAware {
    fn theme(&self) -> &Theme;
}

pub trait ThemeScoped: Sized {
    fn with_theme(self, theme: Theme) -> Self;
}

pub trait StyleRecipe<Props> {
    fn resolve_styles(&self, props: &Props, state: ComponentState) -> StyleMap;
}

pub trait VariantSupport: Sized {
    fn variant(self, value: Variant) -> Self;
    fn size(self, value: Size) -> Self;
    fn radius(self, value: Radius) -> Self;
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

pub trait WithId: Sized {
    fn id(&self) -> &str;
    fn id_mut(&mut self) -> &mut String;

    fn with_id(mut self, id: impl Into<String>) -> Self {
        *self.id_mut() = id.into();
        self
    }
}

#[cfg(feature = "gpui-latest")]
pub trait GpuiRenderComponent: gpui::Render {}

#[cfg(feature = "gpui-latest")]
impl<T> GpuiRenderComponent for T where T: gpui::Render {}

#[cfg(feature = "gpui-latest")]
pub trait GpuiRenderOnceComponent: gpui::RenderOnce {}

#[cfg(feature = "gpui-latest")]
impl<T> GpuiRenderOnceComponent for T where T: gpui::RenderOnce {}

#[cfg(feature = "gpui-latest")]
pub trait GpuiStyledComponent: gpui::Styled {}

#[cfg(feature = "gpui-latest")]
impl<T> GpuiStyledComponent for T where T: gpui::Styled {}
