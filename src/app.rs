use chrono::Local;
use iced::{Task, Theme};
use iced_layershell::{settings::Settings, to_layer_message, Application};
use itertools::Itertools;
use miette::IntoDiagnostic;

use crate::{
    config::Config,
    module::{clock::Clock, ModuleEvent, ModuleGetSet, ModuleGroups, ModuleInfo},
};

struct App {
    module_groups: ModuleGroups,
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = AppMsg;
    type Theme = Theme;
    type Flags = Config;

    fn new(config: Self::Flags) -> (Self, Task<Self::Message>) {
        let res = Self {
            module_groups: (config.left, config.center, config.right).into(),
        };
        let command = Task::batch([Task::perform(async {}, |_| AppMsg::UpdateTime)]);

        (res, command)
    }

    fn namespace(&self) -> String {
        "rbar".into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        let msgs = match message {
            AppMsg::UpdateTime => {
                let clocks: Vec<&mut ModuleInfo<Clock>> =
                    ModuleGetSet::<Clock>::get_mut(&mut self.module_groups).collect();

                if !clocks.is_empty() {
                    clocks.into_iter().for_each(|clock| {
                        clock.module.time = Local::now();
                    });

                    return Task::perform(
                        async {
                            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                        },
                        |_| AppMsg::UpdateTime,
                    );
                }

                vec![]
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

#[to_layer_message]
#[derive(Debug, Clone, PartialEq)]
pub enum AppMsg {
    UpdateTime,

    Module(ModuleEvent),
}

pub fn run(config: Config) -> miette::Result<()> {
    let settings = Settings {
        id: Some("com.tukanoidd.rbar".into()),
        layer_settings: config.layer_shell_settings(),
        flags: config,
        ..Default::default()
    };

    App::run(settings).into_diagnostic()
}
