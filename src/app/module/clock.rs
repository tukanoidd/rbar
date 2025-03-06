use std::time::Duration;

use chrono::Local;
use iced::{
    widget::{button, text},
    Element, Length, Task,
};
use serde::Deserialize;
use smart_default::SmartDefault;
use uuid::Uuid;

use crate::app::AppMsg;

use super::{MainModuleContainerMsg, TModule};

#[derive(Debug)]
pub struct Clock {
    settings: ClockSettings,
}

impl TModule for Clock {
    type Settings = ClockSettings;
    type ViewDeps = ();
    type UpdateDeps = ();
    type Msg = UpdateClock;

    fn new(settings: Self::Settings) -> Self {
        Self { settings }
    }

    fn view(&self, _id: Uuid, _deps: Self::ViewDeps) -> Element<'_, AppMsg> {
        let now = Local::now();
        let now_str = now.format(&self.settings.format).to_string();

        button(text(now_str)).height(Length::Fill).into()
    }

    fn update(&mut self, id: Uuid, _msg: UpdateClock, _deps: Self::UpdateDeps) -> Task<AppMsg> {
        Task::perform(
            async move {
                tokio::time::sleep(Duration::from_millis(1)).await;
                MainModuleContainerMsg::new(id, UpdateClock)
            },
            AppMsg::Modules,
        )
    }
}

#[derive(Debug, SmartDefault, Clone, Deserialize)]
#[serde(default)]
pub struct ClockSettings {
    #[default = "%d-%m-%Y %H:%M:%S"]
    format: String,
}

#[derive(Debug, Clone)]
pub struct UpdateClock;
