use crate::app::AppMsg;

use super::{NoConfig, TModule};

pub struct Battery {
    level: u8,
}

impl TModule for Battery {
    type Config = NoConfig;
    type Event = BatteryEvent;

    fn new(config: Self::Config) -> Self {
        todo!()
    }

    fn update(&mut self, event: Self::Event) -> Option<AppMsg> {
        todo!()
    }

    fn view(&self) -> iced::Element<'_, Self::Event, iced::Theme, iced::Renderer> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatteryEvent {}
