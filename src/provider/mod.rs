use crate::feedback::ToastManager;
use crate::overlay::ModalManager;
use crate::theme::Theme;
use std::sync::Arc;

#[derive(Default)]
pub struct CalmProvider {
    theme: Option<Theme>,
    toast_manager: Option<ToastManager>,
    modal_manager: Option<ModalManager>,
}

#[derive(Clone)]
struct ProviderGlobal {
    theme: Arc<Theme>,
    toast_manager: ToastManager,
    modal_manager: ModalManager,
}

impl gpui::Global for ProviderGlobal {}

impl CalmProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_theme(mut self, configure: impl FnOnce(Theme) -> Theme) -> Self {
        let current = self.theme.take().unwrap_or_default();
        self.theme = Some(configure(current));
        self
    }

    pub fn set_toast_manager(mut self, manager: ToastManager) -> Self {
        self.toast_manager = Some(manager);
        self
    }

    pub fn set_modal_manager(mut self, manager: ModalManager) -> Self {
        self.modal_manager = Some(manager);
        self
    }

    pub fn init(self, cx: &mut gpui::App) {
        if cx.has_global::<ProviderGlobal>() {
            let global = cx.global_mut::<ProviderGlobal>();
            if let Some(theme) = self.theme {
                global.theme = Arc::new(theme);
            }
            if let Some(manager) = self.toast_manager {
                global.toast_manager = manager;
            }
            if let Some(manager) = self.modal_manager {
                global.modal_manager = manager;
            }
            return;
        }

        cx.set_global(ProviderGlobal {
            theme: Arc::new(self.theme.unwrap_or_default()),
            toast_manager: self.toast_manager.unwrap_or_default(),
            modal_manager: self.modal_manager.unwrap_or_default(),
        });
    }

    pub fn theme(cx: &gpui::App) -> Arc<Theme> {
        cx.try_global::<ProviderGlobal>()
            .map(|global| global.theme.clone())
            .unwrap_or_else(|| Arc::new(Theme::default()))
    }

    pub fn toast(cx: &gpui::App) -> ToastManager {
        cx.try_global::<ProviderGlobal>()
            .map(|global| global.toast_manager.clone())
            .unwrap_or_default()
    }

    pub fn modal(cx: &gpui::App) -> ModalManager {
        cx.try_global::<ProviderGlobal>()
            .map(|global| global.modal_manager.clone())
            .unwrap_or_default()
    }
}
