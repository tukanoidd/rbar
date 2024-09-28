pub mod battery;
pub mod clock;

use std::hash::Hash;

use bon::Builder;
use derive_more::derive::{Display, From};
use iced::{
    alignment::Vertical,
    widget::{row, Space},
    Element, Length, Renderer, Theme,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::app::AppMsg;

pub trait TModule {
    type Config: TModuleConfig;
    type Event: Clone;

    fn new(config: Self::Config) -> Self;
    fn update(&mut self, event: Self::Event) -> Option<AppMsg>;
    fn view(&self) -> Element<'_, Self::Event, Theme, Renderer>;
}

pub trait TModuleConfig: Default + Hash + Serialize + for<'de> Deserialize<'de> {}

impl<C> TModuleConfig for C where C: Default + Hash + Serialize + for<'de> Deserialize<'de> {}

#[derive(Default, Display, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NoConfig;

#[derive(Debug, Clone, PartialEq)]
pub struct NoEvent;

#[derive(Builder)]
pub struct ModuleInfo<M>
where
    M: TModule,
{
    pub module: M,
    pub event: Option<M::Event>,
}

impl<M> ModuleInfo<M>
where
    M: TModule,
{
    pub fn new(module: M) -> Self {
        Self {
            module,
            event: None,
        }
    }

    pub fn new_conf(c: M::Config) -> Self {
        Self::new(M::new(c))
    }

    pub fn update(&mut self) -> Option<AppMsg> {
        self.event.take().and_then(|e| self.module.update(e))
    }

    pub fn view(&self) -> Element<'_, M::Event, Theme, Renderer> {
        self.module.view()
    }
}

pub struct Modules {
    modules: Vec<Module>,
    position: ModulePosition,
}

impl Modules {
    pub fn left(mut self) -> Self {
        self.position = ModulePosition::Left;
        self
    }

    pub fn right(mut self) -> Self {
        self.position = ModulePosition::Right;
        self
    }

    pub fn view(&self) -> Element<'_, AppMsg, Theme, Renderer> {
        let mut children = self.modules.iter().map(Module::view).collect_vec();

        match self.position {
            ModulePosition::Left => children.push(Space::with_width(Length::Fill).into()),
            ModulePosition::Right => children.insert(0, Space::with_width(Length::Fill).into()),
            _ => {}
        }

        row(children)
            .spacing(10)
            .align_y(Vertical::Center)
            .height(Length::Fill)
            .width(Length::Shrink)
            .into()
    }

    pub fn update(&mut self) -> impl Iterator<Item = AppMsg> + '_ {
        self.modules.iter_mut().filter_map(|m| m.update())
    }
}

impl<I> From<I> for Modules
where
    I: IntoIterator<Item = ModuleConfig>,
{
    fn from(value: I) -> Self {
        Self {
            modules: value.into_iter().map(From::from).collect(),
            position: ModulePosition::Center,
        }
    }
}

pub enum ModulePosition {
    Left,
    Center,
    Right,
}

pub struct ModuleGroups {
    pub left: Modules,
    pub center: Modules,
    pub right: Modules,
}

impl ModuleGroups {
    pub fn view(&self) -> Element<'_, AppMsg, Theme, Renderer> {
        row![
            self.left.view(),
            Space::with_width(Length::Fill),
            self.center.view(),
            Space::with_width(Length::Fill),
            self.right.view()
        ]
        .align_y(Vertical::Center)
        .height(Length::Fill)
        .into()
    }

    pub fn update(&mut self) -> impl Iterator<Item = AppMsg> + '_ {
        self.left
            .update()
            .chain(self.center.update())
            .chain(self.right.update())
    }
}

impl<M1, M2, M3> From<(M1, M2, M3)> for ModuleGroups
where
    M1: Into<Modules>,
    M2: Into<Modules>,
    M3: Into<Modules>,
{
    fn from((left, center, right): (M1, M2, M3)) -> Self {
        Self {
            left: left.into().left(),
            center: center.into(),
            right: right.into().right(),
        }
    }
}

pub trait ModuleGetSet<M>
where
    M: TModule + 'static,
{
    fn get(&self) -> impl Iterator<Item = &ModuleInfo<M>>;
    fn get_mut(&mut self) -> impl Iterator<Item = &mut ModuleInfo<M>>;

    fn set_event(&mut self, event: M::Event) {
        self.get_mut().for_each(|m| m.event = Some(event.clone()))
    }
}

macro_rules! modules {
    ($($name:ident),+) => {
        paste::paste! {
            $(use [< $name:snake:lower >]::{$name, [< $name Event >]};)+

            #[derive(From)]
            pub enum Module {
                $($name(ModuleInfo<$name>)),+
            }

            impl Module {
                pub fn view(&self) -> Element<'_, AppMsg, Theme, Renderer> {
                    match self {
                        $(Module::$name(m) => m.view().map(|e| AppMsg::Module(e.into()))),+
                    }
                }

                pub fn update(&mut self) -> Option<AppMsg> {
                    match self {
                        $(Module::$name(m) => m.update()),+
                    }
                }
            }

            #[derive(Debug, Clone, PartialEq, From)]
            pub enum ModuleEvent {
                $($name([< $name Event >])),+
            }

            #[derive(Display, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
            pub enum ModuleConfig {
                $($name(<$name as TModule>::Config)),+
            }

            impl From<ModuleConfig> for Module {
                fn from(config: ModuleConfig) -> Self {
                    match config {
                        $(ModuleConfig::$name(c) => Self::$name(ModuleInfo::new_conf(c))),+
                    }
                }
            }

            $(
                impl ModuleGetSet<$name> for Modules {
                    fn get(&self) -> impl Iterator<Item = &ModuleInfo<$name>> {
                        self.modules.iter().filter_map(|m| match m {
                            Module::$name(m) => Some(m),
                            _ => None
                        })
                    }

                    fn get_mut(&mut self) -> impl Iterator<Item = &mut ModuleInfo<$name>> {
                        self.modules.iter_mut().filter_map(|m| match m {
                            Module::$name(m) => Some(m),
                            _ => None
                        })
                    }
                }

                impl ModuleGetSet<$name> for ModuleGroups {
                    fn get(&self) -> impl Iterator<Item = &ModuleInfo<$name>> {
                        ModuleGetSet::<$name>::get(&self.left)
                            .chain(ModuleGetSet::<$name>::get(&self.center))
                            .chain(ModuleGetSet::<$name>::get(&self.right))
                    }

                    fn get_mut(&mut self) -> impl Iterator<Item = &mut ModuleInfo<$name>> {
                        ModuleGetSet::<$name>::get_mut(&mut self.left)
                            .chain(ModuleGetSet::<$name>::get_mut(&mut self.center))
                            .chain(ModuleGetSet::<$name>::get_mut(&mut self.right))
                    }
                }
            )+

            impl ModuleGroups {
                pub fn set_event(&mut self, event: ModuleEvent) {
                    match event {
                        $(ModuleEvent::$name(e) => ModuleGetSet::<$name>::set_event(self, e)),+
                    }
                }
            }
        }
    }
}

modules![Clock, Battery];
