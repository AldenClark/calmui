use crate::feedback::ToastManager;
use crate::icon::IconRegistry;
use crate::motion::MotionConfig;
use crate::overlay::ModalManager;
use crate::theme::{Theme, ThemePatch};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct CalmProvider {
    theme: Arc<Theme>,
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
        self.theme.as_ref()
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

    pub fn set_theme(mut self, theme: Theme) -> Self {
        self.theme = Arc::new(theme);
        self
    }

    pub fn patch_theme(mut self, patch: ThemePatch) -> Self {
        self.theme = Arc::new(self.theme.as_ref().merged(&patch));
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

#[derive(Clone)]
pub struct ProviderGlobal(pub CalmProvider);

impl gpui::Global for ProviderGlobal {}

impl CalmProvider {
    pub fn install(self, cx: &mut gpui::App) {
        cx.set_global(ProviderGlobal(self));
    }

    pub fn read(cx: &gpui::App) -> Self {
        cx.global::<ProviderGlobal>().0.clone()
    }

    pub fn try_read(cx: &gpui::App) -> Option<Self> {
        cx.try_global::<ProviderGlobal>()
            .map(|global| global.0.clone())
    }

    pub fn read_or_default(cx: &gpui::App) -> Self {
        Self::try_read(cx).unwrap_or_default()
    }

    pub fn theme_arc_or_default(cx: &gpui::App) -> Arc<Theme> {
        Self::try_read(cx)
            .map(|provider| provider.theme)
            .unwrap_or_else(|| Arc::new(Theme::default()))
    }

    pub fn motion_or(cx: &gpui::App, fallback: MotionConfig) -> MotionConfig {
        Self::try_read(cx)
            .map(|provider| provider.motion)
            .unwrap_or(fallback)
    }

    pub fn icons_or(cx: &gpui::App, fallback: IconRegistry) -> IconRegistry {
        Self::try_read(cx)
            .map(|provider| provider.icons)
            .unwrap_or(fallback)
    }

    pub fn set_theme_global(cx: &mut gpui::App, theme: Theme) {
        if cx.has_global::<ProviderGlobal>() {
            cx.global_mut::<ProviderGlobal>().0.theme = Arc::new(theme);
        } else {
            Self::new().set_theme(theme).install(cx);
        }
    }

    pub fn patch_theme_global(cx: &mut gpui::App, patch: ThemePatch) {
        if cx.has_global::<ProviderGlobal>() {
            let current = cx.global::<ProviderGlobal>().0.theme.as_ref().clone();
            cx.global_mut::<ProviderGlobal>().0.theme = Arc::new(current.merged(&patch));
        } else {
            Self::new().patch_theme(patch).install(cx);
        }
    }
}
