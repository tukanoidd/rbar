use chrono::{DateTime, Local};
use derive_more::derive::Display;
use iced::widget::{button, text};
use serde::{Deserialize, Serialize};

use crate::app::AppMsg;

use super::ModuleRegistryEvent;

crate::module!(Clock {
    module: {
        fields: { time: chrono::DateTime<chrono::Local> },
        types: {
            Config: {},
            InitInput: {},
            InitOutput: {},
            CycleInput: {},
            Event: { UpdateTime },
        },
        methods: {
            new: { Self { time: Local::now() } },
            init: { ClockInitOutput {} },
            cycle [event]: {
                #[allow(unreachable_patterns)]
                match event {
                    ClockEvent::UpdateTime => {
                        *time = chrono::Local::now();
                        Some(AppMsg::module_registry(
                            ModuleRegistryEvent::widget(ClockWidgetEvent::SetTime(*time))
                        ))
                    },
                    _ => None
                }
            },
            widget_state: { ClockWidgetState {
                format,
                time: *time
            } },
        },
    },
    widget: {
        fields: {
            format: ClockFormat,
            time: DateTime<Local>
        },
        types: {
            Config: {
                types: {
                    Style: {},
                },
                fields: { format: ClockFormat },
            },
            Event: { SwitchFormat, SetTime(DateTime<Local>) },
        },
        methods: {
            view [style]: {
                button(text(time.format(format.chrono_format()).to_string()))
                .on_press(Self::Event::SwitchFormat)
                .into()
            },
            update [event]: {
                match event {
                    ClockWidgetEvent::SwitchFormat => {
                        format.switch();
                    },
                    ClockWidgetEvent::SetTime(new_time) => {
                        *time = new_time;
                    },
                }

                None
            },
        },
    }
});

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
