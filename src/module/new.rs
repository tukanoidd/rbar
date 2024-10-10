use std::{collections::HashMap, sync::Arc};

use iced::{Element, Renderer, Task, Theme};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{app::AppMsg, util::color_css_to_iced};

#[derive(Debug)]
pub struct RegistryModuleInfo<M>
where
    M: Module,
{
    module: M,
    info: GenericModuleInfo,
    widgets: HashMap<Uuid, RegistryModuleWidgetInfo<M>>,
}

#[derive(Debug)]
pub struct RegistryModuleWidgetInfo<M>
where
    M: Module,
{
    state: Arc<Mutex<<M::Widget as ModuleWidget<M>>::State>>,
    #[allow(clippy::type_complexity)]
    style: Option<Arc<<<M::Widget as ModuleWidget<M>>::Config as ModuleWidgetConfig>::Style>>,
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
pub struct ModuleWidgetGroupStyle {
    #[default(5.0)]
    padding: f32,
    #[default(5.0)]
    spacing: f32,
    border: Option<BorderStyle>,
    background_color: Option<csscolorparser::Color>,
}

#[derive(Debug, SmartDefault, Serialize, Deserialize)]
pub struct BorderStyle {
    width: Option<f32>,
    color: Option<csscolorparser::Color>,
    radius: Option<f32>,
}

impl<'a> From<&'a BorderStyle> for iced::Border {
    fn from(
        BorderStyle {
            width,
            color,
            radius,
        }: &'a BorderStyle,
    ) -> Self {
        let mut border = iced::Border::default();

        if let Some(width) = width {
            border = border.width(*width);
        }

        if let Some(color) = color {
            border = border.color(color_css_to_iced(color));
        }

        if let Some(radius) = radius {
            border.rounded(*radius);
        }

        border
    }
}

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
        config: <Self::Widget as ModuleWidget<Self>>::Config,
    ) -> <Self::Widget as ModuleWidget<Self>>::State;
}

pub trait ModuleConfig: std::fmt::Debug + Default + Serialize + for<'de> Deserialize<'de> {}

impl ModuleConfig for () {}

pub trait ModuleEvent: std::fmt::Debug + Clone {}

pub trait ModuleWidget<M>: std::fmt::Debug
where
    M: Module,
{
    type Config: ModuleWidgetConfig;
    type Event: ModuleWidgetEvent;
    type State: ModuleWidgetState;

    fn view<'a>(
        self,
        style: Option<Arc<<Self::Config as ModuleWidgetConfig>::Style>>,
        state: Arc<Mutex<Self::State>>,
    ) -> Element<'a, Self::Event, Theme, Renderer>;
    fn update(
        self,
        state: Arc<Mutex<Self::State>>,
        event: Self::Event,
    ) -> Option<ModuleWidgetUpdateOutput<M>>;
}

pub trait ModuleWidgetStyle: std::fmt::Debug + Serialize + for<'de> Deserialize<'de> {}

impl ModuleWidgetStyle for () {}

pub trait ModuleWidgetEvent: std::fmt::Debug + Clone {}

impl ModuleWidgetEvent for () {}

pub trait ModuleWidgetState: std::fmt::Debug {}

impl ModuleWidgetState for () {}

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

impl ModuleWidgetConfig for () {
    type Style = ();
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
                                ) -> Vec<(Uuid, RegistryModuleWidgetInfo<$name>)> {
                                        inds
                                            .into_iter()
                                            .map(|ind| widgets.remove(ind))
                                            .filter_map(move |c| match c {
                                                ModuleWidgetConfigTy::$name(StyledModuleWidgetConfig {
                                                    config,
                                                    style
                                                }) => {
                                                    let id = Uuid::new_v4();
                                                    ids.push(id);

                                                    let state = Arc::new(Mutex::new(module.widget_state(config)));
                                                    let style = style.map(Arc::new);

                                                    Some((
                                                        id,
                                                        RegistryModuleWidgetInfo {
                                                            state,
                                                            style
                                                        }
                                                    ))
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

                pub fn view<'a>(s: Arc<Mutex<Self>>) -> Element<'a, AppMsg, Theme, Renderer> {
                    let s = s.blocking_lock();
                    let ModuleWidgetsLayout {
                        left,
                        center,
                        right
                    } = &s.widgets_layout;
                    $(let [< $name:snake:lower >] = &s.[< $name:snake:lower >];)*

                    use iced::{*, widget::*};

                    let mut module_groups_row = row![]
                        .align_y(alignment::Vertical::Center);

                    let widgets = |row: &StyledModuleWidgetRow, align_x: alignment::Horizontal| match row.widget_ids.is_empty() {
                        true => {
                            let StyledModuleWidgetRow {
                                widget_ids,
                                style: ModuleWidgetGroupStyle {
                                    padding,
                                    spacing,
                                    border,
                                    background_color
                                },
                            } = row;
                            let mut row_widgets = row![]
                                .padding(*padding)
                                .spacing(*spacing)
                                .align_y(alignment::Vertical::Center)
                                .width(Length::Shrink);

                            let style = {
                                let mut style = container::Style::default();

                                if let Some(border) = border {
                                    style = style.border(border);
                                }

                                if let Some(bg_color) = background_color {
                                    style = style.background(color_css_to_iced(bg_color));
                                }

                                style
                            };

                            for widget_id in widget_ids {
                                $(if let Some(RegistryModuleInfo {
                                    widgets,
                                    ..
                                }) = [< $name:snake:lower >] {
                                    if let Some(RegistryModuleWidgetInfo {
                                        state,
                                        style
                                    }) = widgets.get(widget_id) {
                                        row_widgets = row_widgets.push(
                                            [< $name Widget >]
                                                .view(style.clone(), state.clone())
                                                .map(ModuleRegistryEvent::widget)
                                                .map(AppMsg::ModuleRegistry)
                                        );
                                    }
                                })*
                            }

                            Some(
                                container(row_widgets)
                                    .style(move |_| style)
                                    .width(Length::Fill)
                                    .align_y(alignment::Vertical::Center)
                                    .align_x(align_x)
                            )
                        },
                        false => None,
                    };

                    let left_widgets = widgets(left, alignment::Horizontal::Left);
                    let center_widgets = widgets(center, alignment::Horizontal::Center);
                    let right_widgets = widgets(right, alignment::Horizontal::Right);

                    if let Some(left_widgets) = left_widgets {
                        module_groups_row = module_groups_row.push(left_widgets);
                    }

                    if let Some(center_widgets) = center_widgets {
                        module_groups_row = module_groups_row.push(center_widgets);
                    }

                    if let Some(right_widgets) = right_widgets {
                        module_groups_row = module_groups_row.push(right_widgets);
                    }

                    module_groups_row.into()
                }
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum ModuleTy {
                $($name),*
            }

            #[derive(Default, Serialize, Deserialize)]
            pub struct ModuleConfigCollection {
                $([< $name:snake:lower >]: GenericModuleConfig<<$name as Module>::Config>),+
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
