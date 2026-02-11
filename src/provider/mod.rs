use crate::feedback::ToastManager;
use crate::icon::IconRegistry;
use crate::motion::MotionConfig;
use crate::overlay::ModalManager;
use crate::theme::{Theme, ThemePatch};

#[derive(Clone, Default)]
pub struct CalmProvider {
    theme: Theme,
    motion: MotionConfig,
    icons: IconRegistry,
    toast_manager: ToastManager,
    modal_manager: ModalManager,
}

impl CalmProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn motion(&self) -> MotionConfig {
        self.motion
    }

    pub fn icons(&self) -> &IconRegistry {
        &self.icons
    }

    pub fn toast_manager(&self) -> &ToastManager {
        &self.toast_manager
    }

    pub fn modal_manager(&self) -> &ModalManager {
        &self.modal_manager
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_theme_patch(mut self, patch: ThemePatch) -> Self {
        self.theme = self.theme.merged(&patch);
        self
    }

    pub fn with_motion(mut self, motion: MotionConfig) -> Self {
        self.motion = motion;
        self
    }

    pub fn with_icons(mut self, icons: IconRegistry) -> Self {
        self.icons = icons;
        self
    }

    pub fn with_toast_manager(mut self, manager: ToastManager) -> Self {
        self.toast_manager = manager;
        self
    }

    pub fn with_modal_manager(mut self, manager: ModalManager) -> Self {
        self.modal_manager = manager;
        self
    }
}

#[cfg(feature = "gpui-latest")]
#[derive(Clone)]
pub struct ProviderGlobal(pub CalmProvider);

#[cfg(feature = "gpui-latest")]
impl gpui::Global for ProviderGlobal {}

#[cfg(feature = "gpui-latest")]
impl CalmProvider {
    pub fn install(self, cx: &mut gpui::App) {
        cx.set_global(ProviderGlobal(self));
    }

    pub fn read(cx: &gpui::App) -> Self {
        cx.global::<ProviderGlobal>().0.clone()
    }
}
