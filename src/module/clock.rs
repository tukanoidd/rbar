use chrono::{DateTime, Local};
use derive_more::derive::Display;
use iced::{
    widget::{button, text},
    Element, Renderer, Theme,
};
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::app::AppMsg;

use super::TModule;

#[derive(SmartDefault)]
pub struct Clock {
    #[default(Local::now())]
    pub time: DateTime<Local>,
    format: ClockFormat,
}

impl TModule for Clock {
    type Config = ClockFormat;
    type Event = ClockEvent;

    fn new(format: ClockFormat) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    fn update(&mut self, event: Self::Event) -> Option<AppMsg> {
        match event {
            ClockEvent::SwitchFormat => self.format.switch(),
        }

        None
    }

    fn view(&self) -> Element<'_, Self::Event, Theme, Renderer> {
        button(text(
            self.time.format(self.format.chrono_format()).to_string(),
        ))
        .on_press(ClockEvent::SwitchFormat)
        .into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClockEvent {
    SwitchFormat,
}

#[allow(non_camel_case_types)]
#[derive(Default, Display, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClockFormat {
    HH_MM,
    HH_MM_SS,
    DD_MM_YY_HH_MM,
    DD_MM_YYYY_HH_MM,
    DD_MM_YY_HH_MM_SS,
    #[default]
    DD_MM_YYYY_HH_MM_SS,
}

impl ClockFormat {
    fn switch(&mut self) {
        *self = match self {
            ClockFormat::HH_MM => Self::HH_MM_SS,
            ClockFormat::HH_MM_SS => Self::DD_MM_YY_HH_MM,
            ClockFormat::DD_MM_YY_HH_MM => Self::DD_MM_YYYY_HH_MM,
            ClockFormat::DD_MM_YYYY_HH_MM => Self::DD_MM_YY_HH_MM_SS,
            ClockFormat::DD_MM_YY_HH_MM_SS => Self::DD_MM_YYYY_HH_MM_SS,
            ClockFormat::DD_MM_YYYY_HH_MM_SS => Self::HH_MM,
        };
    }

    fn chrono_format(&self) -> &'static str {
        match self {
            ClockFormat::HH_MM => "%H:%M",
            ClockFormat::HH_MM_SS => "%H:%M:%S",
            ClockFormat::DD_MM_YY_HH_MM => "%d/%m/%y %H:%M",
            ClockFormat::DD_MM_YYYY_HH_MM => "%d/%m/%Y %H:%M",
            ClockFormat::DD_MM_YY_HH_MM_SS => "%d/%m/%y %H:%M:%S",
            ClockFormat::DD_MM_YYYY_HH_MM_SS => "%d/%m/%Y %H:%M:%S",
        }
    }
}
