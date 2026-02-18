use crate::feedback::ToastManager;
use crate::overlay::ModalManager;
use crate::theme::Theme;
#[cfg(feature = "i18n")]
use crate::{I18nManager, Locale};
use std::sync::Arc;

#[derive(Default)]
pub struct CalmProvider {
    theme: Arc<Theme>,
    toast_manager: ToastManager,
    modal_manager: ModalManager,
    #[cfg(feature = "i18n")]
    i18n: I18nManager,
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

    #[cfg(feature = "i18n")]
    pub fn set_i18n_locale(self, locale: impl Into<Locale>) -> Self {
        self.i18n.set_locale(locale);
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

    #[cfg(feature = "i18n")]
    pub fn i18n(cx: &gpui::App) -> I18nManager {
        cx.global::<CalmProvider>().i18n.clone()
    }
}
