use crate::feedback::ToastManager;
use crate::overlay::ModalManager;
use crate::theme::Theme;
use std::sync::Arc;

#[derive(Default)]
pub struct CalmProvider {
    theme: Arc<Theme>,
    toast_manager: ToastManager,
    modal_manager: ModalManager,
}

impl gpui::Global for CalmProvider {}

impl CalmProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_theme(mut self, configure: impl FnOnce(Arc<Theme>) -> Theme) -> Self {
        self.theme = configure(self.theme).into();
        self
    }

    pub fn init(self, cx: &mut gpui::App) {
        cx.set_global(self);
    }

    pub fn theme(cx: &gpui::App) -> Arc<Theme> {
        cx.global::<CalmProvider>().theme.clone()
    }

    pub fn toast(cx: &gpui::App) -> ToastManager {
        cx.global::<CalmProvider>().toast_manager.clone()
    }

    pub fn modal(cx: &gpui::App) -> ModalManager {
        cx.global::<CalmProvider>().modal_manager.clone()
    }
}
