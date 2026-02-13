use crate::components::{AppShellWindowConfig, OverlayMaterialCapabilities};
use crate::motion::MotionConfig;
use crate::provider::CalmProvider;
use crate::theme::{Theme, ThemePatch};

type LaunchHook = Box<dyn FnOnce(&mut gpui::App, &CalmProvider) + 'static>;

pub struct AppShellWindowRuntime {
    config: AppShellWindowConfig,
}

impl AppShellWindowRuntime {
    pub fn new(config: AppShellWindowConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &AppShellWindowConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: AppShellWindowConfig) {
        self.config = config;
    }

    pub fn configure_window_options(
        &self,
        configure: impl FnOnce(gpui::WindowOptions) -> gpui::WindowOptions,
    ) -> gpui::WindowOptions {
        let user_options = configure(gpui::WindowOptions::default());
        self.config.apply_to_window_options(user_options)
    }

    pub fn open_window<V>(
        &self,
        cx: &mut gpui::App,
        configure: impl FnOnce(gpui::WindowOptions) -> gpui::WindowOptions,
        build_root_view: impl FnOnce(&mut gpui::Window, &mut gpui::App) -> gpui::Entity<V>,
    ) -> Result<gpui::WindowHandle<V>, String>
    where
        V: 'static + gpui::Render,
    {
        cx.open_window(self.configure_window_options(configure), build_root_view)
            .map_err(|err| err.to_string())
    }

    pub fn open_window_with_options<V>(
        &self,
        cx: &mut gpui::App,
        options: gpui::WindowOptions,
        build_root_view: impl FnOnce(&mut gpui::Window, &mut gpui::App) -> gpui::Entity<V>,
    ) -> Result<gpui::WindowHandle<V>, String>
    where
        V: 'static + gpui::Render,
    {
        cx.open_window(
            self.config.apply_to_window_options(options),
            build_root_view,
        )
        .map_err(|err| err.to_string())
    }
}

pub struct CalmApplication {
    application: gpui::Application,
    provider: CalmProvider,
    app_shell_window_config: AppShellWindowConfig,
    launch_hooks: Vec<LaunchHook>,
}

impl Default for CalmApplication {
    fn default() -> Self {
        Self::new()
    }
}

impl CalmApplication {
    fn default_provider() -> CalmProvider {
        CalmProvider::new().with_overlay_capability_probe(
            |window: &gpui::Window, _cx: &gpui::App| {
                let mut capabilities = OverlayMaterialCapabilities {
                    window_system: window.supports_window_material(),
                    region_system: window.supports_region_material(),
                    renderer_blur: window.supports_renderer_backdrop_blur(),
                }
                .with_env_overrides();

                if !capabilities.window_system
                    && !capabilities.region_system
                    && !capabilities.renderer_blur
                {
                    capabilities = OverlayMaterialCapabilities::detect_runtime();
                }

                capabilities
            },
        )
    }

    pub fn new() -> Self {
        Self {
            application: gpui::Application::new(),
            provider: Self::default_provider(),
            app_shell_window_config: AppShellWindowConfig::default(),
            launch_hooks: Vec::new(),
        }
    }

    pub fn headless() -> Self {
        Self {
            application: gpui::Application::headless(),
            provider: Self::default_provider(),
            app_shell_window_config: AppShellWindowConfig::default(),
            launch_hooks: Vec::new(),
        }
    }

    pub fn from_application(application: gpui::Application) -> Self {
        Self {
            application,
            provider: Self::default_provider(),
            app_shell_window_config: AppShellWindowConfig::default(),
            launch_hooks: Vec::new(),
        }
    }

    pub fn application(&self) -> &gpui::Application {
        &self.application
    }

    pub fn configure_application(
        mut self,
        configure: impl FnOnce(gpui::Application) -> gpui::Application,
    ) -> Self {
        self.application = configure(self.application);
        self
    }

    pub fn with_assets(mut self, asset_source: impl gpui::AssetSource) -> Self {
        self.application = self.application.with_assets(asset_source);
        self
    }

    pub fn with_quit_mode(mut self, mode: gpui::QuitMode) -> Self {
        self.application = self.application.with_quit_mode(mode);
        self
    }

    pub fn on_open_urls(&mut self, callback: impl FnMut(Vec<String>) + 'static) -> &mut Self {
        self.application.on_open_urls(callback);
        self
    }

    pub fn on_reopen(&mut self, callback: impl FnMut(&mut gpui::App) + 'static) -> &mut Self {
        self.application.on_reopen(callback);
        self
    }

    pub fn with_provider(mut self, provider: CalmProvider) -> Self {
        self.provider = provider;
        self
    }

    pub fn with_app_shell_window_config(mut self, config: AppShellWindowConfig) -> Self {
        self.app_shell_window_config = config;
        self
    }

    pub fn set_theme(mut self, theme: Theme) -> Self {
        self.provider = self.provider.set_theme(theme);
        self
    }

    pub fn patch_theme(mut self, patch: ThemePatch) -> Self {
        self.provider = self.provider.patch_theme(patch);
        self
    }

    pub fn with_motion(mut self, motion: MotionConfig) -> Self {
        self.provider = self.provider.with_motion(motion);
        self
    }

    pub fn before_launch(
        mut self,
        hook: impl FnOnce(&mut gpui::App, &CalmProvider) + 'static,
    ) -> Self {
        self.launch_hooks.push(Box::new(hook));
        self
    }

    pub fn run<F>(self, on_finish_launching: F)
    where
        F: 'static + FnOnce(&mut gpui::App),
    {
        let provider = self.provider;
        let launch_hooks = self.launch_hooks;
        self.application.run(move |cx| {
            provider.clone().install(cx);

            for hook in launch_hooks {
                hook(cx, &provider);
            }

            on_finish_launching(cx);
        });
    }

    pub fn run_with_app_shell<F>(self, on_finish_launching: F)
    where
        F: 'static + FnOnce(&mut gpui::App, &mut AppShellWindowRuntime),
    {
        let provider = self.provider;
        let launch_hooks = self.launch_hooks;
        let app_shell_window_config = self.app_shell_window_config;
        self.application.run(move |cx| {
            provider.clone().install(cx);

            for hook in launch_hooks {
                hook(cx, &provider);
            }

            let mut runtime = AppShellWindowRuntime::new(app_shell_window_config);
            on_finish_launching(cx, &mut runtime);
        });
    }
}
