use std::sync::Arc;

use iced::{
    widget::{center, text},
    Subscription, Task, Theme,
};
use iced_layershell::{settings::Settings, to_layer_message, Application};
use miette::IntoDiagnostic;
use tokio::sync::Mutex;

use crate::{
    config::Config,
    module::new::{ModuleRegistry, ModuleRegistryEvent, ModulesConfig},
    util::ResultExt,
};

pub fn run(config: Config) -> miette::Result<()> {
    let settings = Settings {
        id: Some("com.tukanoidd.rbar".into()),
        layer_settings: config.layer_shell_settings(),
        flags: config,
        fonts: vec![
            iced_fonts::BOOTSTRAP_FONT_BYTES.into(),
            iced_fonts::NERD_FONT_BYTES.into(),
            iced_fonts::REQUIRED_FONT_BYTES.into(),
        ],
        ..Default::default()
    };

    App::run(settings).into_diagnostic()
}

struct App {
    module_registry: Option<Arc<Mutex<ModuleRegistry>>>,
}

impl App {
    async fn init(modules_config: ModulesConfig) -> miette::Result<AppInit> {
        let module_registry = ModuleRegistry::new(modules_config)
            .await
            .tokio_mutex()
            .arc()?;

        Ok(AppInit { module_registry })
    }
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = AppMsg;
    type Theme = Theme;
    type Flags = Config;

    fn new(config: Self::Flags) -> (Self, Task<Self::Message>) {
        let res = Self {
            module_registry: None,
        };

        let mut tasks = vec![];

        tasks.push(Task::perform(Self::init(config.modules), |res| {
            AppMsg::Init(res.map_err(|e| e.to_string()))
        }));

        let command = Task::batch(tasks);

        (res, command)
    }

    fn namespace(&self) -> String {
        "rbar".into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        let tasks = match message {
            AppMsg::Init(res) => match res {
                Ok(AppInit { module_registry }) => {
                    let tasks = module_registry.blocking_lock().after_init_tasks();
                    tasks
                }
                Err(err) => {
                    panic!("Failed to initialize the app: {err}");
                }
            },
            AppMsg::NotifyError(err) => {
                tracing::error!("{err}");
                vec![]
            }

            AppMsg::ModuleRegistry(ev) => match self.module_registry.clone() {
                Some(module_registry) => {
                    vec![Task::perform(
                        async move {
                            let mut module_registry = module_registry.lock().await;
                            module_registry.event(ev).await.err_str()
                        },
                        |res| match res {
                            Ok(msg) => msg,
                            Err(err) => AppMsg::NotifyError(err),
                        },
                    )]
                }
                None => vec![],
            },

            AppMsg::AnchorChange(_)
            | AppMsg::AnchorSizeChange(_, _)
            | AppMsg::LayerChange(_)
            | AppMsg::MarginChange(_)
            | AppMsg::SizeChange(_)
            | AppMsg::VirtualKeyboardPressed { .. } => vec![],
        };

        Task::batch(tasks)
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        match self.module_registry.clone() {
            Some(module_registry) => ModuleRegistry::view(module_registry),
            None => center(text("Initializing Modules...")).into(),
        }
    }

    fn theme(&self) -> Self::Theme {
        Theme::CatppuccinMocha
    }
}

#[derive(Debug, Clone)]
pub struct AppInit {
    module_registry: Arc<Mutex<ModuleRegistry>>,
}

impl PartialEq for AppInit {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum AppMsg {
    Init(Result<AppInit, String>),
    NotifyError(String),
    ModuleRegistry(ModuleRegistryEvent),
}

impl AppMsg {
    pub fn module_registry(e: impl Into<ModuleRegistryEvent>) -> Self {
        Self::ModuleRegistry(e.into())
    }
}
