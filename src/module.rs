pub mod audio;
pub mod battery;
pub mod new;

use std::hash::Hash;

use bon::Builder;
use derive_more::derive::Display;
use iced::{Element, Renderer, Theme};
use serde::{Deserialize, Serialize};

use crate::app::AppMsg;

pub trait TModule: std::fmt::Debug {
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

#[derive(Debug, Builder)]
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
