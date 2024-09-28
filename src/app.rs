use std::{sync::Arc, time::Duration};

use chrono::Local;
use iced::{Task, Theme};
use iced_layershell::{settings::Settings, to_layer_message, Application};
use itertools::Itertools;
use miette::IntoDiagnostic;
use tokio::time::sleep;

use crate::{
    config::Config,
    module::{
        battery::{Battery, BatteryInfo},
        clock::Clock,
        ModuleEvent, ModuleGetSet, ModuleGroups, ModuleInfo,
    },
};

struct App {
    module_groups: ModuleGroups,

    battery_info: Option<Arc<BatteryInfo>>,
}

impl App {
    async fn init() -> miette::Result<AppInit> {
        let battery_info = BatteryInfo::init()?;

        Ok(AppInit {
            battery_info: Arc::new(battery_info),
        })
    }
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = AppMsg;
    type Theme = Theme;
    type Flags = Config;

    fn new(config: Self::Flags) -> (Self, Task<Self::Message>) {
        let res = Self {
            module_groups: (config.left, config.center, config.right).into(),

            battery_info: None,
        };
        let command = Task::batch([
            Task::perform(async {}, |_| AppMsg::UpdateTime),
            Task::perform(Self::init(), |res| {
                AppMsg::Init(res.map_err(|e| e.to_string()))
            }),
        ]);

        (res, command)
    }

    fn namespace(&self) -> String {
        "rbar".into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        let msgs = match message {
            AppMsg::Init(res) => match res {
                Ok(AppInit { battery_info }) => {
                    vec![AppMsg::RefreshBattery(Ok(battery_info))]
                }
                Err(err) => {
                    panic!("Failed to initialize the app: {err}");
                }
            },

            AppMsg::UpdateTime => {
                let clocks: Vec<&mut ModuleInfo<Clock>> =
                    ModuleGetSet::<Clock>::get_mut(&mut self.module_groups).collect();

                if !clocks.is_empty() {
                    clocks.into_iter().for_each(|clock| {
                        clock.module.time = Local::now();
                    });

                    return Task::perform(
                        async {
                            sleep(tokio::time::Duration::from_millis(300)).await;
                        },
                        |_| AppMsg::UpdateTime,
                    );
                }

                vec![]
            }
            AppMsg::RefreshBattery(info) => match info {
                Ok(info) => {
                    self.battery_info = Some(info);

                    vec![AppMsg::UpdateBattery]
                }
                Err(err) => {
                    tracing::error!("Failed to refresh battery info: {err}");

                    return Task::perform(
                        async {
                            sleep(Duration::from_millis(500)).await;
                            BatteryInfo::init().map(Arc::new).map_err(|e| e.to_string())
                        },
                        AppMsg::RefreshBattery,
                    );
                }
            },
            AppMsg::UpdateBattery => {
                if let Some(battery_info) = &self.battery_info {
                    let batteries: Vec<&mut ModuleInfo<Battery>> =
                        ModuleGetSet::<Battery>::get_mut(&mut self.module_groups).collect();

                    if !batteries.is_empty() {
                        let data = battery_info.data().collect_vec();

                        batteries.into_iter().for_each(|battery| {
                            battery.module.set(data.clone());
                        });
                    }
                }

                return Task::perform(
                    async {
                        sleep(Duration::from_millis(1000)).await;
                        BatteryInfo::init().map(Arc::new).map_err(|e| e.to_string())
                    },
                    AppMsg::RefreshBattery,
                );
            }
            AppMsg::Module(ev) => {
                self.module_groups.set_event(ev);
                self.module_groups.update().dedup().collect()
            }

            AppMsg::AnchorChange(_)
            | AppMsg::LayerChange(_)
            | AppMsg::MarginChange(_)
            | AppMsg::SizeChange(_)
            | AppMsg::VirtualKeyboardPressed { .. } => vec![],
        };

        match msgs.is_empty() {
            true => Task::none(),
            false => Task::batch(msgs.into_iter().map(|msg| self.update(msg))),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        self.module_groups.view()
    }

    fn theme(&self) -> Self::Theme {
        Theme::CatppuccinMocha
    }
}

#[derive(Debug, Clone)]
pub struct AppInit {
    battery_info: Arc<BatteryInfo>,
}

impl PartialEq for AppInit {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[to_layer_message]
#[derive(Debug, Clone, PartialEq)]
pub enum AppMsg {
    Init(Result<AppInit, String>),

    UpdateTime,

    RefreshBattery(Result<Arc<BatteryInfo>, String>),
    UpdateBattery,

    Module(ModuleEvent),
}

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
