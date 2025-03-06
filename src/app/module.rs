pub mod audio;
pub mod clock;

use std::fmt::Debug;

use clock::{Clock, UpdateClock};
use derive_more::From;
use iced::{
    widget::{container, row},
    Alignment, Color, Element, Length, Task,
};
use serde::Deserialize;
use smart_default::SmartDefault;
use uuid::Uuid;

use crate::config::{
    MainContainerSettings, ModuleContainerSettings, ModuleContainersSettings, ModuleSettings,
    ModuleTypeSettings,
};

use super::AppMsg;

pub struct MainModulesContainer {
    settings: MainContainerSettings,

    module_registry: ModuleRegistry,
}

impl MainModulesContainer {
    pub fn new(settings: MainContainerSettings, containers: ModuleContainersSettings) -> Self {
        let module_registry = ModuleRegistry::new(containers);

        Self {
            settings,
            module_registry,
        }
    }

    pub fn view(&self) -> Element<'_, AppMsg> {
        container(row![
            Self::module_container(&self.module_registry.left, Alignment::Start),
            Self::module_container(&self.module_registry.center, Alignment::Center),
            Self::module_container(&self.module_registry.right, Alignment::End),
        ])
        .align_x(Alignment::Center)
        .padding(self.settings.padding)
        .into()
    }

    fn module_container(
        ModuleContainerInfo { styling, modules }: &ModuleContainerInfo,
        align_x: Alignment,
    ) -> Element<'_, AppMsg> {
        container(row(modules.iter().map(|module| module.view())))
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .align_x(align_x)
            .padding(styling.padding)
            .into()
    }

    pub fn update(
        &mut self,
        MainModuleContainerMsg { id, msg }: MainModuleContainerMsg,
    ) -> Task<AppMsg> {
        let module = self.module_registry.get_mut(&id);

        match module {
            Some(ModuleInfo { id, module }) => match (module, msg) {
                (Module::Clock(clock), ModuleMsg::Clock(msg)) => return clock.update(*id, msg, ()),
            },
            None => tracing::warn!("Module with id {id} was not found!"),
        }

        Task::none()
    }

    pub fn initial_update(&mut self) -> Task<AppMsg> {
        Task::batch(
            self.module_registry
                .left
                .modules
                .iter_mut()
                .chain(self.module_registry.center.modules.iter_mut())
                .chain(self.module_registry.right.modules.iter_mut())
                .map(|ModuleInfo { id, module }| match module {
                    Module::Clock(clock) => clock.update(*id, UpdateClock, ()),
                }),
        )
    }
}

#[derive(SmartDefault, Deserialize)]
#[serde(default)]
pub struct ModuleContainersInfo<T>
where
    T: Default,
{
    pub left: T,
    pub center: T,
    pub right: T,
}

type ModuleRegistry = ModuleContainersInfo<ModuleContainerInfo>;

impl ModuleRegistry {
    fn new(
        ModuleContainersSettings {
            left,
            center,
            right,
        }: ModuleContainersSettings,
    ) -> Self {
        tracing::debug!("Setting up registry...");

        let left = Self::info_from_settings(left);
        let center = Self::info_from_settings(center);
        let right = Self::info_from_settings(right);

        tracing::debug!("Registry setup!");

        tracing::trace!("Left: {left:#?}");
        tracing::trace!("Center: {center:#?}");
        tracing::trace!("Right: {right:#?}");

        Self {
            left,
            center,
            right,
        }
    }

    fn info_from_settings(
        ModuleContainerSettings {
            modules,
            padding,
            background_color,
        }: ModuleContainerSettings,
    ) -> ModuleContainerInfo {
        ModuleContainerInfo {
            styling: ModuleContainerStyling {
                padding,
                background_color: background_color.map(Into::into),
            },
            modules: Self::modules_from_settings(modules),
        }
    }

    fn modules_from_settings(modules: Vec<ModuleSettings>) -> Vec<ModuleInfo> {
        modules
            .iter()
            .flat_map(|ModuleSettings { ty, .. }: &ModuleSettings| match ty {
                ModuleTypeSettings::Clock(clock_settings) => {
                    Some(ModuleInfo::new(Clock::new(clock_settings.clone())))
                }
            })
            .collect()
    }

    fn get_mut(&mut self, id: &Uuid) -> Option<&mut ModuleInfo> {
        self.left
            .modules
            .iter_mut()
            .chain(self.center.modules.iter_mut())
            .chain(self.right.modules.iter_mut())
            .find(|module| module.id == *id)
    }
}

#[derive(Debug, Default)]
struct ModuleContainerInfo {
    styling: ModuleContainerStyling,
    modules: Vec<ModuleInfo>,
}

#[derive(Debug, SmartDefault)]
struct ModuleContainerStyling {
    #[default = 5]
    padding: u16,
    background_color: Option<Color>,
}

#[derive(Debug)]
struct ModuleInfo {
    id: Uuid,
    module: Module,
}

impl ModuleInfo {
    fn new(module: impl Into<Module>) -> Self {
        Self {
            id: Uuid::new_v4(),
            module: module.into(),
        }
    }

    fn view(&self) -> Element<'_, AppMsg> {
        match &self.module {
            Module::Clock(clock) => clock.view(self.id, ()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MainModuleContainerMsg {
    id: Uuid,
    msg: ModuleMsg,
}

impl MainModuleContainerMsg {
    pub fn new(id: Uuid, msg: impl Into<ModuleMsg>) -> Self {
        Self {
            id,
            msg: msg.into(),
        }
    }
}

#[derive(Debug, From)]
pub enum Module {
    Clock(Clock),
}

#[derive(Debug, Clone, From)]
pub enum ModuleMsg {
    Clock(UpdateClock),
}

pub trait TModule<'a> {
    type Settings: Debug + Default + Clone + for<'de> Deserialize<'de>;
    type ViewDeps;
    type UpdateDeps;
    type Msg: Debug + Clone;

    fn new(settings: Self::Settings) -> Self;
    fn view(&'a self, id: Uuid, deps: Self::ViewDeps) -> Element<'a, AppMsg>;
    fn update(&'a mut self, id: Uuid, msg: Self::Msg, deps: Self::UpdateDeps) -> Task<AppMsg>;
}
