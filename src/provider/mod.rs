use crate::theme::Theme;
use std::sync::Arc;

#[derive(Default)]
pub struct CalmProvider {
    theme: Option<Theme>,
}

#[derive(Clone)]
struct ProviderGlobal {
    theme: Arc<Theme>,
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

    pub fn init(self, cx: &mut gpui::App) {
        match (cx.has_global::<ProviderGlobal>(), self.theme) {
            (true, Some(theme)) => {
                cx.global_mut::<ProviderGlobal>().theme = Arc::new(theme);
            }
            (true, None) => {}
            (false, Some(theme)) => {
                cx.set_global(ProviderGlobal {
                    theme: Arc::new(theme),
                });
            }
            (false, None) => {
                cx.set_global(ProviderGlobal {
                    theme: Arc::new(Theme::default()),
                });
            }
        }
    }

    pub fn theme(cx: &gpui::App) -> Arc<Theme> {
        cx.try_global::<ProviderGlobal>()
            .map(|global| global.theme.clone())
            .unwrap_or_else(|| Arc::new(Theme::default()))
    }
}
