use std::{sync::Arc, time::Duration};

use chrono::Local;
use iced::{Subscription, Task, Theme};
use iced_layershell::{settings::Settings, to_layer_message, Application};
use itertools::Itertools;
use miette::IntoDiagnostic;

use crate::{
    config::Config,
    module::{
        audio::{Audio, AudioInfo},
        battery::{Battery, BatteryEvent, BatteryInfo},
        clock::{Clock, ClockEvent},
        ModuleEvent, ModuleGetSet, ModuleGroups,
    },
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
    module_groups: ModuleGroups,

    battery_info: Option<Arc<BatteryInfo>>,
    audio_info: Option<Arc<AudioInfo>>,
}

impl App {
    async fn init(battery_in_config: bool, audio_in_config: bool) -> miette::Result<AppInit> {
        let battery_info = match battery_in_config {
            true => BatteryInfo::init().arc().some()?,
            false => None,
        };
        let audio_info = match audio_in_config {
            true => AudioInfo::init().await.arc().some()?,
            false => None,
        };

        Ok(AppInit {
            battery_info,
            audio_info,
        })
    }
}

macro_rules! task_wait_msg {
    ($ms:expr => $block:block, $msg:ident) => {
        iced::Task::perform(
            async {
                tokio::time::sleep(std::time::Duration::from_millis($ms)).await;
                $block
            },
            AppMsg::$msg,
        )
    };
    (d $duration:expr => $block:block, $msg:ident) => {
        iced::Task::perform(
            async move {
                tokio::time::sleep($duration).await;
                $block
            },
            AppMsg::$msg,
        )
    };
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
            audio_info: None,
        };

        let mut tasks = vec![];

        if ModuleGetSet::<Clock>::has(&res.module_groups) {
            tasks.push(Task::perform(async {}, |_| AppMsg::UpdateTime));
        }

        let battery_module_in_config = ModuleGetSet::<Battery>::has(&res.module_groups);
        let audio_module_in_config = ModuleGetSet::<Audio>::has(&res.module_groups);

        tasks.push(Task::perform(
            Self::init(battery_module_in_config, audio_module_in_config),
            |res| AppMsg::Init(res.map_err(|e| e.to_string())),
        ));

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
        let msgs = match message {
            AppMsg::Init(res) => match res {
                Ok(AppInit {
                    battery_info,
                    audio_info,
                }) => {
                    vec![AppMsg::RefreshBattery(Ok(battery_info))]
                }
                Err(err) => {
                    panic!("Failed to initialize the app: {err}");
                }
            },

            AppMsg::WaitAndMsg(duration, msg) => {
                let msg = Box::into_inner(msg);
                return Task::perform(tokio::time::sleep(duration), move |_| msg.clone());
            }
            AppMsg::WaitGetBatteryInfo(duration) => {
                return task_wait_msg!(d duration => {
                    BatteryInfo::init().arc().some().err_str()
                }, RefreshBattery)
            }

            AppMsg::UpdateTime => {
                self.module_groups
                    .set_event(ClockEvent::SetTime(Local::now()));
                self.module_groups.update().collect()
            }

            AppMsg::RefreshBattery(info) => match info {
                Ok(info) => {
                    self.battery_info = info;
                    vec![AppMsg::UpdateBattery]
                }
                Err(err) => {
                    tracing::error!("Failed to refresh battery info: {err}");
                    vec![AppMsg::wait_ms_get_battery_info(500)]
                }
            },
            AppMsg::UpdateBattery => {
                let upd = AppMsg::wait_s_get_battery_info(30);

                match &self.battery_info {
                    None => vec![upd],
                    Some(battery_info) => {
                        let data = battery_info.data().collect_vec();
                        self.module_groups.set_event(BatteryEvent::SetData(data));
                        self.module_groups.update().chain([upd]).collect()
                    }
                }
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
            false => Task::batch(msgs.into_iter().dedup().map(|msg| self.update(msg))),
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
    battery_info: Option<Arc<BatteryInfo>>,
    audio_info: Option<Arc<AudioInfo>>,
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

    WaitAndMsg(Duration, Box<AppMsg>),
    WaitGetBatteryInfo(Duration),

    RefreshBattery(Result<Option<Arc<BatteryInfo>>, String>),
    UpdateBattery,

    Module(ModuleEvent),
}

impl<T> From<T> for AppMsg
where
    T: Into<ModuleEvent>,
{
    fn from(value: T) -> Self {
        Self::Module(value.into())
    }
}

impl AppMsg {
    pub fn wait_ms_msg(ms: u64, msg: impl Into<Self>) -> Self {
        Self::WaitAndMsg(Duration::from_millis(ms), Box::new(msg.into()))
    }

    pub fn wait_ms_get_battery_info(ms: u64) -> Self {
        Self::WaitGetBatteryInfo(Duration::from_millis(ms))
    }

    pub fn wait_s_get_battery_info(s: u64) -> Self {
        Self::WaitGetBatteryInfo(Duration::from_secs(s))
    }
}
