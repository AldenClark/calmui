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
