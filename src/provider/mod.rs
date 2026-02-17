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
    toast_manager: Option<ToastManager>,
    modal_manager: Option<ModalManager>,
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

    pub fn enable_toast_manager(self) -> Self {
        self.set_toast_manager(ToastManager::default())
    }

    pub fn set_modal_manager(mut self, manager: ModalManager) -> Self {
        self.modal_manager = Some(manager);
        self
    }

    pub fn enable_modal_manager(self) -> Self {
        self.set_modal_manager(ModalManager::default())
    }

    pub fn init(self, cx: &mut gpui::App) {
        if cx.has_global::<ProviderGlobal>() {
            let global = cx.global_mut::<ProviderGlobal>();
            if let Some(theme) = self.theme {
                global.theme = Arc::new(theme);
            }
            if let Some(manager) = self.toast_manager {
                global.toast_manager = Some(manager);
            }
            if let Some(manager) = self.modal_manager {
                global.modal_manager = Some(manager);
            }
            return;
        }

        cx.set_global(ProviderGlobal {
            theme: Arc::new(self.theme.unwrap_or_default()),
            toast_manager: self.toast_manager,
            modal_manager: self.modal_manager,
        });
    }

    pub fn theme(cx: &gpui::App) -> Arc<Theme> {
        cx.try_global::<ProviderGlobal>()
            .map(|global| global.theme.clone())
            .unwrap_or_else(|| Arc::new(Theme::default()))
    }

    pub fn toast(cx: &gpui::App) -> Option<ToastManager> {
        cx.try_global::<ProviderGlobal>()
            .and_then(|global| global.toast_manager.clone())
    }

    pub fn modal(cx: &gpui::App) -> Option<ModalManager> {
        cx.try_global::<ProviderGlobal>()
            .and_then(|global| global.modal_manager.clone())
    }
}
