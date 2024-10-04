use std::{collections::HashMap, sync::Arc};

use iced::{Element, Renderer, Task, Theme};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::app::AppMsg;

#[derive(Debug)]
pub struct RegistryModuleInfo<M>
where
    M: Module,
{
    module: M,
    info: GenericModuleInfo,
    widgets: HashMap<Uuid, <M::Widget as ModuleWidget<M>>::State>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct ModulesConfig {
    configs: ModuleConfigCollection,
    widgets: ModuleWidgetGroupConfigs,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GenericModuleConfig<C>
where
    C: ModuleConfig,
{
    #[serde(flatten, default)]
    info: GenericModuleInfo,
    #[serde(flatten, default, bound = "C: for<'c> Deserialize<'c>")]
    config: C,
}

#[derive(Debug, SmartDefault, Serialize, Deserialize)]
#[serde(default)]
pub struct GenericModuleInfo {
    #[default(500)]
    delay_ms: u64,
}

#[derive(SmartDefault, Serialize, Deserialize)]
pub struct ModuleWidgetGroupConfigs {
    left: ModuleWidgetGroupConfig,
    #[default(ModuleWidgetGroupConfig {
        widgets: vec![
            StyledModuleWidgetConfig {
                config: ClockWidgetConfig::default(),
                style: None
            }.into(),
        ],
        style: Default::default()
    })]
    center: ModuleWidgetGroupConfig,
    right: ModuleWidgetGroupConfig,
}

#[derive(SmartDefault, Serialize, Deserialize)]
pub struct ModuleWidgetGroupConfig {
    widgets: Vec<ModuleWidgetConfigTy>,
    style: ModuleWidgetGroupStyle,
}

#[derive(Debug, SmartDefault, Serialize, Deserialize)]
pub struct ModuleWidgetGroupStyle {}

#[derive(Debug)]
struct ModuleWidgetsLayout {
    left: StyledModuleWidgetRow,
    center: StyledModuleWidgetRow,
    right: StyledModuleWidgetRow,
}

#[derive(Debug)]
struct StyledModuleWidgetRow {
    widget_ids: Vec<Uuid>,
    style: ModuleWidgetGroupStyle,
}

pub trait Module: std::fmt::Debug + Sized {
    type Config: ModuleConfig;
    type InitInput;
    type InitOutput;
    type CycleInput;
    type Event: ModuleEvent;

    type Widget: ModuleWidget<Self>;

    fn new(config: Self::Config) -> miette::Result<Self>
    where
        Self: Sized;
    async fn init(&mut self, input: Self::InitInput) -> miette::Result<Self::InitOutput>;
    async fn cycle(
        &mut self,
        registry: &mut ModuleRegistry,
        input: Self::CycleInput,
        event: Self::Event,
    ) -> miette::Result<Option<AppMsg>>;

    fn widget_state(
        &self,
        config: StyledModuleWidgetConfig<<Self::Widget as ModuleWidget<Self>>::Config>,
    ) -> <Self::Widget as ModuleWidget<Self>>::State;
}

pub trait ModuleConfig: std::fmt::Debug + Default + Serialize + for<'de> Deserialize<'de> {}

pub trait ModuleEvent: std::fmt::Debug + Clone {}

pub trait ModuleWidget<M>: std::fmt::Debug
where
    M: Module,
{
    type Config: ModuleWidgetConfig;
    type Event: ModuleWidgetEvent;
    type State: ModuleWidgetState;

    fn view(self, state: &Self::State) -> Element<'_, Self::Event, Theme, Renderer>;
    fn update(
        self,
        state: &mut Self::State,
        event: Self::Event,
    ) -> Option<ModuleWidgetUpdateOutput<M>>;
}

pub trait ModuleWidgetStyle: std::fmt::Debug + Serialize + for<'de> Deserialize<'de> {}
pub trait ModuleWidgetEvent: std::fmt::Debug + Clone {}
pub trait ModuleWidgetState: std::fmt::Debug {}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct StyledModuleWidgetConfig<C>
where
    C: ModuleWidgetConfig,
{
    #[serde(default, flatten, bound = "C: for<'c> Deserialize<'c>")]
    pub config: C,
    #[serde(default)]
    pub style: Option<C::Style>,
}

pub trait ModuleWidgetConfig:
    std::fmt::Debug + Default + Serialize + for<'de> Deserialize<'de>
{
    type Style: ModuleWidgetStyle;
}

pub enum ModuleWidgetUpdateOutput<M>
where
    M: Module,
{
    Widget(<M::Widget as ModuleWidget<M>>::Event),
    Module(<M as Module>::Event),
}

impl<M> ModuleWidgetUpdateOutput<M>
where
    M: Module,
{
    pub fn widget(event: impl Into<<M::Widget as ModuleWidget<M>>::Event>) -> Self {
        Self::Widget(event.into())
    }

    pub fn module(event: impl Into<<M as Module>::Event>) -> Self {
        Self::Module(event.into())
    }
}

macro_rules! modules {
    ($($name:ident {
        update_msg: $update_msg:ident,
        after_init_msg: [$($after_init_msg:expr),*]
        $(,)?
    }),+) => {
        paste::paste! {
            $(
                pub mod [< $name:snake:lower >];
                use [< $name:snake:lower >]::*;
            )+

            #[derive(Debug)]
            pub struct ModuleRegistry {
                widgets_layout: ModuleWidgetsLayout,
                $([< $name:snake:lower >]: Option<RegistryModuleInfo<$name>>),+
            }

            impl ModuleRegistry {
                pub async fn new(ModulesConfig {
                    configs: ModuleConfigCollection {
                        $([< $name:snake:lower >]: [< $name:snake:lower _config >]),*
                    },
                    widgets: ModuleWidgetGroupConfigs {
                        left: ModuleWidgetGroupConfig {
                            widgets: mut left_widgets,
                            style: left_style
                        },
                        center: ModuleWidgetGroupConfig {
                            widgets: mut center_widgets,
                            style: center_style
                        },
                        right: ModuleWidgetGroupConfig {
                            widgets: mut right_widgets,
                            style: right_style
                        },
                    },
                }: ModulesConfig) -> miette::Result<Self> {
                    let module_tys = left_widgets.iter()
                        .chain(center_widgets.iter())
                        .chain(right_widgets.iter())
                        .map(|c: &ModuleWidgetConfigTy| match c {
                            $(ModuleWidgetConfigTy::$name(_) => ModuleTy::$name),*
                        })
                        .dedup()
                        .collect_vec();

                    let mut left_layout = vec![];
                    let mut center_layout = vec![];
                    let mut right_layout = vec![];

                    $(
                        let [< $name:snake:lower >] = match module_tys.contains(&ModuleTy::$name) {
                            true => {
                                let GenericModuleConfig { info, config } = [< $name:snake:lower _config >];
                                let module = $name::new(config)?;
                                let widgets = {
                                    let inds = |widgets: &Vec<ModuleWidgetConfigTy>| widgets
                                        .iter().
                                        enumerate()
                                        .filter_map(|(ind, c)| match c {
                                            ModuleWidgetConfigTy::$name(_) => Some(ind),
                                            _ => None
                                        })
                                        .collect_vec();

                                    let left_inds = inds(&left_widgets);
                                    let center_inds = inds(&center_widgets);
                                    let right_inds = inds(&right_widgets);

                                    fn widget_states<'a>(
                                        module: &$name,
                                        inds: Vec<usize>,
                                        widgets: &'a mut Vec<ModuleWidgetConfigTy>,
                                        ids: &'a mut Vec<Uuid>
                                ) -> Vec<(Uuid, [< $name WidgetState >])> {
                                        inds
                                            .into_iter()
                                            .map(|ind| widgets.remove(ind))
                                            .filter_map(move |c| match c {
                                                ModuleWidgetConfigTy::$name(c) => {
                                                    let id = Uuid::new_v4();
                                                    ids.push(id);

                                                    Some((id, module.widget_state(c)))
                                                },
                                                _ => None
                                            }).collect()
                                    }

                                    widget_states(&module, left_inds, &mut left_widgets, &mut left_layout)
                                        .into_iter()
                                        .chain(widget_states(&module, center_inds, &mut center_widgets, &mut center_layout).into_iter())
                                        .chain(widget_states(&module, right_inds, &mut right_widgets, &mut right_layout).into_iter())
                                        .collect()
                                };

                                Some(RegistryModuleInfo { module, info, widgets })
                            },
                            false => None,
                        };
                    )*

                    let widgets_layout = ModuleWidgetsLayout {
                        left: StyledModuleWidgetRow {
                            widget_ids: left_layout,
                            style: left_style,
                        },
                        center: StyledModuleWidgetRow {
                            widget_ids: center_layout,
                            style: center_style,
                        },
                        right: StyledModuleWidgetRow {
                            widget_ids: right_layout,
                            style: right_style,
                        },
                    };

                    Ok(Self {
                        widgets_layout,
                        $([< $name:snake:lower >]),*
                    })
                }

                pub fn after_init_tasks(&self) -> Vec<Task<AppMsg>> {
                    let Self {
                        $([< $name:snake:lower >],)*
                        ..
                    } = self;

                    let mut tasks = vec![];

                    $(
                        if let Some(RegistryModuleInfo {
                            info,
                            ..
                        }) = [< $name:snake:lower >] {
                            let delay_ms = info.delay_ms;
                            tasks.push(Task::perform(async move {
                                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                            }, |_| {
                                AppMsg::ModuleRegistry(ModuleRegistryEvent::module([< $name Event >]::$update_msg))
                            }));

                            let mut other_tasks = [$($after_init_msg),*]
                                .into_iter()
                                .map(move |msg: AppMsg| Task::perform(async move { msg }, |m| m))
                                .collect_vec();

                            tasks.append(&mut other_tasks);
                        }
                    )*

                    tasks
                }
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum ModuleTy {
                $($name),*
            }

            #[derive(Default, Serialize, Deserialize)]
            pub struct ModuleConfigCollection {
                $([< $name:snake:lower >]: GenericModuleConfig<[< $name Config >]>),+
            }

            #[derive(Debug, derive_more::From, Clone)]
            pub enum ModuleEventTy {
                $($name([< $name Event >])),+
            }

            #[derive(derive_more::From, Serialize, Deserialize)]
            pub enum ModuleWidgetConfigTy {
                $($name(StyledModuleWidgetConfig<[< $name WidgetConfig >]>)),+
            }

            #[derive(Debug, derive_more::From, Clone)]
            pub enum ModuleWidgetEventTy {
                $($name([< $name WidgetEvent >])),+
            }
        }
    };
}

impl ModuleRegistry {
    pub async fn event(&mut self, event: ModuleRegistryEvent) -> miette::Result<AppMsg> {
        todo!()
    }

    pub fn view<'a>(s: Arc<Mutex<Self>>) -> Element<'a, AppMsg, Theme, Renderer> {
        todo!()
    }
}

#[derive(Debug, Clone, derive_more::From)]
pub enum ModuleRegistryEvent {
    Module(ModuleEventTy),
    Widget(ModuleWidgetEventTy),
}

impl ModuleRegistryEvent {
    pub fn module(e: impl Into<ModuleEventTy>) -> Self {
        Self::Module(e.into())
    }

    pub fn widget(e: impl Into<ModuleWidgetEventTy>) -> Self {
        Self::Widget(e.into())
    }
}

modules![Clock {
    update_msg: UpdateTime,
    after_init_msg: []
}];

#[macro_export]
macro_rules! module {
    (path) => {$crate::module::new};
    ($name:ident {
        module: {
            fields: {
                $($field:ident: $field_ty:ty),*
                $(,)?
            },
            types: {
                Config: {
                    $(
                        $(#[default = $config_field_default:expr])?
                        $config_field:ident: $config_field_ty:ty
                    ),*
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
                widget_state: $module_widget_state_block:block
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
                    types: {
                        Style: {
                            $($style_field:ident: $style_field_ty:ty),*
                            $(,)?
                        }
                        $(,)?
                    },
                    fields: {
                        $(
                            $(#[default = $widget_config_field_default:expr])?
                            $widget_config_field:ident: $widget_config_field_ty:ty
                        ),*
                        $(,)?
                    }
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
    }) => {
        paste::paste! {
            #[derive(Debug)]
            pub struct $name {
                $($field: $field_ty),*
            }

            impl $crate::module::new::Module for $name {
                type Config = [< $name Config >];
                type InitInput = [< $name InitInput >];
                type InitOutput = [< $name InitOutput >];
                type CycleInput = [< $name CycleInput >];
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
                    registry: &mut $crate::module::new::ModuleRegistry,
                    Self::CycleInput {
                        $($cycle_input_field),*
                    }: Self::CycleInput,
                    $cycle_event: Self::Event
                ) -> miette::Result<Option<$crate::app::AppMsg>> {
                    let Self { $($field),* } = self;
                    Ok($module_cycle_block)
                }

                fn widget_state(
                    &self,
                    $crate::module::new::StyledModuleWidgetConfig {
                        config: [< $name WidgetConfig >] {
                            $($widget_config_field),*
                        },
                        style
                    }: $crate::module::new::StyledModuleWidgetConfig<[< $name WidgetConfig >]>
                ) -> <Self::Widget as $crate::module::new::ModuleWidget<Self>>::State {
                    let Self { $($field),* } = self;
                    $module_widget_state_block
                }
            }

            #[derive(Debug, smart_default::SmartDefault, serde::Serialize, serde::Deserialize)]
            pub struct [< $name Config >] {
                $(
                    $(#[default = $config_field_default])?
                    $config_field: $config_field_ty
                ),*
            }

            impl $crate::module::new::ModuleConfig for [< $name Config >] {}

            pub struct [< $name InitInput >] {
                $($init_input_field: $init_input_field_ty),*
            }

            pub struct [< $name InitOutput >] {
                $($init_output_field: $init_output_field_ty),*
            }

            pub struct [< $name CycleInput >] {
                $($cycle_input_field: $cycle_input_field_ty),*
            }

            #[derive(Debug, Clone)]
            pub enum [< $name Event >] {
                $(
                    $event_variant
                    $({ $($event_variant_field: $event_variant_field_ty),* })?
                    $(($event_single_variant_ty))?
                ),*
            }

            impl $crate::module::new::ModuleEvent for [< $name Event >] {}

            #[derive(Debug)]
            pub struct [< $name Widget >];

            #[derive(Debug)]
            pub struct [< $name WidgetState >] {
                $($widget_field: $widget_field_ty),*
            }

            impl $crate::module::new::ModuleWidgetState for [< $name WidgetState >] {}

            impl $crate::module::new::ModuleWidget<$name> for [< $name Widget >] {
                type Config = [< $name WidgetConfig >];
                type Event = [< $name WidgetEvent >];
                type State = [< $name WidgetState >];

                fn view(
                    self,
                    Self::State { $($widget_field),*}: &Self::State
                ) -> iced::Element<'_, Self::Event, iced::Theme, iced::Renderer> {
                    $widget_view_block
                }

                fn update(
                    self,
                    Self::State { $($widget_field),* }: &mut Self::State,
                    $widget_update_event: Self::Event
                ) -> Option<$crate::module::new::ModuleWidgetUpdateOutput<$name>> {
                    $widget_update_block
                }
            }

            #[derive(Debug, smart_default::SmartDefault, serde::Serialize, serde::Deserialize)]
            pub struct [< $name WidgetConfig >] {
                $(
                    $(#[default = $widget_config_field_default])?
                    $widget_config_field: $widget_config_field_ty
                ),*
            }

            impl $crate::module::new::ModuleWidgetConfig for [< $name WidgetConfig >] {
                type Style = [< $name WidgetStyle >];
            }

            #[derive(Debug, Serialize, Deserialize)]
            pub struct [< $name WidgetStyle >] {
                $($style_field: $style_field_ty),*
            }

            impl $crate::module::new::ModuleWidgetStyle for [< $name WidgetStyle >] {}

            #[derive(Debug, Clone)]
            pub enum [< $name WidgetEvent >] {
                $($widget_event_variant
                    $({
                        $($widget_event_variant_field: $widget_event_variant_field_ty),+
                    })?
                    $(($widget_event_variant_single_field_ty))?
                ),*
            }

            impl $crate::module::new::ModuleWidgetEvent for [< $name WidgetEvent >] {}
        }
    };
}
