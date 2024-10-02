use chrono::{DateTime, Local};
use derive_more::derive::Display;
use iced::{
    widget::{button, text},
    Element, Renderer, Theme,
};
use serde::{Deserialize, Serialize};

use crate::app::AppMsg;

pub trait Module: Sized {
    type Config: ModuleConfig;
    type InitInput;
    type InitOutput;
    type CycleInput;
    type CycleOutput;
    type Event;

    type Widget: ModuleWidget<Self>;

    fn new(config: Self::Config) -> miette::Result<Self>
    where
        Self: Sized;
    async fn init(&mut self, input: Self::InitInput) -> miette::Result<Self::InitOutput>;
    async fn cycle(
        &mut self,
        input: Self::CycleInput,
        event: Self::Event,
    ) -> miette::Result<Self::CycleOutput>;

    fn widget(&self, config: <Self::Widget as ModuleWidget<Self>>::Config) -> Self::Widget;
}

pub trait ModuleConfig: Serialize + for<'de> Deserialize<'de> {}

pub trait ModuleWidget<M>
where
    M: Module,
{
    type Config: ModuleWidgetConfig;
    type Event: ModuleWidgetEvent;

    fn view(&self) -> Element<'_, Self::Event, Theme, Renderer>;
    fn update(&mut self, event: Self::Event) -> Option<ModuleWidgetUpdateOutput<M>>;
}

pub trait ModuleWidgetConfig: Serialize + for<'de> Deserialize<'de> {}
pub trait ModuleWidgetEvent: Clone {}

pub enum ModuleWidgetUpdateOutput<M>
where
    M: Module,
{
    App(AppMsg),
    Widget(<M::Widget as ModuleWidget<M>>::Event),
    Module(<M as Module>::Event),
}

impl<M> ModuleWidgetUpdateOutput<M>
where
    M: Module,
{
    pub fn app(msg: impl Into<AppMsg>) -> Self {
        Self::App(msg.into())
    }

    pub fn widget(event: impl Into<<M::Widget as ModuleWidget<M>>::Event>) -> Self {
        Self::Widget(event.into())
    }

    pub fn module(event: impl Into<<M as Module>::Event>) -> Self {
        Self::Module(event.into())
    }
}

macro_rules! modules {
    ($(
        $name:ident {
            module: {
                fields: {
                    $($field:ident: $field_ty:ty),*
                    $(,)?
                },
                types: {
                    Config: {
                        $($config_field:ident: $config_field_ty:ty),*
                        $(,)?
                    },
                    InitInput: {
                        $($init_input_field:ident: $init_input_field_ty:ty),*
                        $(,)?
                    },
                    InitOutput: {
                        $($init_output_field:ident: $init_output_field_ty:ty),*
                        $(,)?
                    },
                    CycleInput: {
                        $($cycle_input_field:ident: $cycle_input_field_ty:ty),*
                        $(,)?
                    },
                    CycleOutput: {
                        $($cycle_output_field:ident: $cycle_output_field_ty:ty),*
                        $(,)?
                    },
                    Event: {
                        $(
                            $event_variant:ident
                            $({
                                $($event_variant_field:ident: $event_variant_field_ty:ty),+
                            })?
                            $(($event_single_variant_ty:ty))?
                        ),*
                        $(,)?
                    },
                    $(,)?
                },
                methods: {
                    new: $module_new_block:block,
                    init: $module_init_block:block,
                    cycle [$cycle_event:ident]: $module_cycle_block:block,
                    widget: $module_widget_block:block
                    $(,)?
                }
                $(,)?
            },
            widget: {
                fields: {
                    $($widget_field:ident: $widget_field_ty:ty),*
                    $(,)?
                },
                types: {
                    Config: {
                        $($widget_config_field:ident: $widget_config_field_ty:ty),*
                        $(,)?
                    },
                    Event: {
                        $(
                            $widget_event_variant:ident
                            $({
                                $($widget_event_variant_field:ident: $widget_event_variant_field_ty:ty),+
                                $(,)?
                            })?
                            $(($widget_event_variant_single_field_ty:ty))?
                        ),*
                        $(,)?
                    }
                    $(,)?
                },
                methods: {
                    view: $widget_view_block:block,
                    update [$widget_update_event:ident]: $widget_update_block:block
                    $(,)?
                }
                $(,)?
            }
            $(,)?
        }
    ),+) => {
        paste::paste! {$(
            pub struct $name {
                $($field: $field_ty),*
            }

            impl Module for $name {
                type Config = [< $name Config >];
                type InitInput = [< $name InitInput >];
                type InitOutput = [< $name InitOutput >];
                type CycleInput = [< $name CycleInput >];
                type CycleOutput = [< $name CycleOutput >];
                type Event = [< $name Event >];

                type Widget = [< $name Widget >];

                fn new(Self::Config { $($config_field),* }: Self::Config) -> miette::Result<Self>
                where
                    Self: Sized
                {
                    Ok($module_new_block)
                }

                async fn init(
                    &mut self,
                    Self::InitInput {
                        $($init_input_field),*
                    }: Self::InitInput
                ) -> miette::Result<Self::InitOutput> {
                    let Self { $($field),* } = self;
                    Ok($module_init_block)
                }

                async fn cycle(
                    &mut self,
                    Self::CycleInput {
                        $($cycle_input_field),*
                    }: Self::CycleInput,
                    $cycle_event: Self::Event
                ) -> miette::Result<Self::CycleOutput> {
                    let Self { $($field),* } = self;
                    Ok($module_cycle_block)
                }

                fn widget(
                    &self,
                    [< $name WidgetConfig >] {
                        $($widget_config_field),*
                    }: [< $name WidgetConfig >]
                ) -> Self::Widget {
                    let Self { $($field),* } = self;
                    $module_widget_block
                }
            }

            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct [< $name Config >] {
                $($config_field: $config_field_ty),*
            }

            impl ModuleConfig for [< $name Config >] {}

            pub struct [< $name InitInput >] {
                $($init_input_field: $init_input_field_ty),*
            }

            pub struct [< $name InitOutput >] {
                $($init_output_field: $init_output_field_ty),*
            }

            pub struct [< $name CycleInput >] {
                $($cycle_input_field: $cycle_input_field_ty),*
            }

            pub struct [< $name CycleOutput >] {
                $($cycle_output_field: $cycle_output_field_ty),*
            }

            #[derive(Clone)]
            pub enum [< $name Event >] {
                $(
                    $event_variant
                    $({ $($event_variant_field: $event_variant_field_ty),* })?
                    $(($event_single_variant_ty))?
                ),*
            }

            pub struct [< $name Widget >] {
                $($widget_field: $widget_field_ty),*
            }

            impl ModuleWidget<$name> for [< $name Widget >] {
                type Config = [< $name WidgetConfig >];
                type Event = [< $name WidgetEvent >];

                fn view(&self) -> iced::Element<'_, Self::Event, iced::Theme, iced::Renderer> {
                    let Self { $($widget_field),* } = self;
                    $widget_view_block
                }

                fn update(&mut self, $widget_update_event: Self::Event) -> Option<ModuleWidgetUpdateOutput<$name>> {
                    let Self { $($widget_field),* } = self;
                    $widget_update_block
                }
            }

            #[derive(serde::Serialize, serde::Deserialize)]
            pub struct [< $name WidgetConfig >] {
                $($widget_config_field: $widget_config_field_ty),*
            }

            impl ModuleWidgetConfig for [< $name WidgetConfig >] {}

            #[derive(Clone)]
            pub enum [< $name WidgetEvent >] {
                $($widget_event_variant
                    $({
                        $($widget_event_variant_field: $widget_event_variant_field_ty),+
                    })?
                    $(($widget_event_variant_single_field_ty))?
                ),*
            }

            impl ModuleWidgetEvent for [< $name WidgetEvent >] {}
        )+}
    };
}

modules![Clock {
    module: {
        fields: { time: chrono::DateTime<chrono::Local> },
        types: {
            Config: {},
            InitInput: {},
            InitOutput: {},
            CycleInput: {},
            CycleOutput: {},
            Event: { UpdateTime },
        },
        methods: {
            new: { Self { time: Local::now() } },
            init: { ClockInitOutput {} },
            cycle [event]: {
                match event {
                    ClockEvent::UpdateTime => *time = chrono::Local::now(),
                }

                ClockCycleOutput {}
            },
            widget: { ClockWidget {
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
            Config: { format: ClockFormat },
            Event: { SwitchFormat },
        },
        methods: {
            view: {
                button(text(time.format(format.chrono_format()).to_string()))
                .on_press(Self::Event::SwitchFormat)
                .into()
            },
            update [event]: {
                match event {
                    ClockWidgetEvent::SwitchFormat => {
                        format.switch();
                    },
                }

                None
            },
        },
    }
}];

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
