pub mod module;

use iced::{executor, Task, Theme};
use iced_layershell::to_layer_message;
use module::{MainModuleContainerMsg, MainModulesContainer};

use crate::{
    config::Config,
    util::{audio::Audio, wayland::Wayland},
};

pub struct App {
    wayland: Wayland,
    audio: Audio,

    main_container: MainModulesContainer,
}

impl iced_layershell::Application for App {
    type Executor = executor::Default;
    type Message = AppMsg;
    type Theme = Theme;
    type Flags = AppFlags;

    fn new(
        AppFlags {
            wayland,
            audio,

            config: Config { main, containers },
        }: Self::Flags,
    ) -> (Self, Task<Self::Message>) {
        let main_container = MainModulesContainer::new(main, containers);
        let mut res = Self {
            wayland,
            audio,

            main_container,
        };

        let task = res.main_container.initial_update();

        (res, task)
    }

    fn namespace(&self) -> String {
        "RBar".into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            AppMsg::Modules(msg) => self.main_container.update(msg),
            _ => Task::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        self.main_container.view()
    }

    fn theme(&self) -> Self::Theme {
        Theme::CatppuccinMocha
    }
}

pub struct AppFlags {
    pub wayland: Wayland,
    pub audio: Audio,

    pub config: Config,
}

#[to_layer_message]
#[derive(derive_more::Debug, Clone)]
pub enum AppMsg {
    Modules(MainModuleContainerMsg),
}
