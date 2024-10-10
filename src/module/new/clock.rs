use chrono::{DateTime, Local};
use derive_more::derive::Display;
use iced::widget::{button, text};
use macros::{module_widget, Module};
use serde::{Deserialize, Serialize};

use crate::app::AppMsg;

use super::ModuleRegistryEvent;

#[derive(Debug, Module)]
#[module(
    widget = "ClockWidget", 
    type_fields(event(name = UpdateTime)),
    methods(
        new = "Self { time: Local::now() }",
        init = "()",
        cycle = "{
            #[allow(unreachable_patterns)]
            match event {
                ClockEvent::UpdateTime => {
                    self.time = chrono::Local::now();
                    Some(AppMsg::module_registry(
                        ModuleRegistryEvent::widget(ClockWidgetEvent::SetTime(self.time))
                    ))
                },
                _ => None
            }
        }",
        widget_state = "ClockWidgetState {
            format: config.format,
            time: self.time
        }"
    )
)]
pub struct Clock {
    time: chrono::DateTime<chrono::Local>,
}

#[module_widget(
    module = Clock,
    type_fields(
        config(name = format, ty = "ClockFormat"),

        event(name = SwitchFormat),
        event(name = SetTime, field(name = time, ty = "DateTime<Local>"))
    ),
    methods(
        view = "
            button(text(
                state.time.format(state.format.chrono_format()).to_string()
            ))
                .on_press(Self::Event::SwitchFormat)
                .into()
        ",
        update = "{
            match event {
                ClockWidgetEvent::SwitchFormat => {
                    state.format.switch();
                },
                ClockWidgetEvent::SetTime(new_time) => {
                    state.time = new_time;
                },
            }

            None
        }"
    )
)]
#[derive(Debug)]
pub struct ClockWidget {
    format: ClockFormat,
    time: DateTime<Local>,
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
