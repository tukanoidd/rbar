use std::path::Path;

use derive_more::From;
use iced_layershell::reexport::Anchor;
use miette::Diagnostic;
use nickel_lang_core::{eval::cache::lazy::CBNCache, program::Program};
use serde::Deserialize;
use smart_default::SmartDefault;
use thiserror::Error;

use crate::{
    app::module::{clock::ClockSettings, ModuleContainersInfo},
    util::color::ColorWrap,
};

#[derive(SmartDefault, Deserialize)]
#[serde(default)]
pub struct Config {
    pub main: MainContainerSettings,
    pub containers: ModuleContainersSettings,
}

impl Config {
    pub fn open_or_default(config_dir: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let config_dir = config_dir.as_ref();
        tracing::debug!("Opening config inside: {config_dir:?}");

        if !config_dir.exists() {
            tracing::debug!("Config dir {config_dir:?} doesn't exist, creating...");
            std::fs::create_dir_all(config_dir)?;
        }

        let config_path = config_dir.join("config.ncl");

        match config_path.exists() {
            true => {
                tracing::trace!("Evaluating config at {config_path:?}");

                let mut program =
                    Program::<CBNCache>::new_from_file(config_path, std::io::stdout())?;

                let deser_res = program
                    .eval_full()
                    .map_err(|e| format!("{e:?}"))
                    .map_err(ConfigError::Nickel)?;

                tracing::trace!("Config evaluated, converting to JSON...");
                let deser_json = serde_json::to_string(&deser_res)?;

                tracing::trace!("Conversion to JSON complete, deserializing into a Rust type...");
                let config = serde_json::from_str(&deser_json)?;
                tracing::trace!("Deserialization complete!");

                Ok(config)
            }
            false => {
                tracing::warn!("Config doesn't exist, using default fallback option...");
                Ok(Self::default())
            }
        }
    }
}

#[derive(SmartDefault, Deserialize)]
#[serde(default)]
pub struct MainContainerSettings {
    #[default = 50]
    pub height: u32,
    #[default = 50]
    pub width: u32,

    #[default = 5]
    pub padding: u16,

    pub side: Side,
}

#[derive(Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Left,
    Right,
    #[default]
    Top,
    Bottom,
}

impl From<Side> for Anchor {
    fn from(value: Side) -> Self {
        match value {
            Side::Left => Anchor::Left | Anchor::Top | Anchor::Bottom,
            Side::Right => Anchor::Right | Anchor::Top | Anchor::Bottom,
            Side::Top => Anchor::Top | Anchor::Right | Anchor::Left,
            Side::Bottom => Anchor::Bottom | Anchor::Right | Anchor::Left,
        }
    }
}

pub type ModuleContainersSettings = ModuleContainersInfo<ModuleContainerSettings>;

#[derive(SmartDefault, Deserialize)]
#[serde(default)]
pub struct ModuleContainerSettings {
    pub modules: Vec<ModuleSettings>,

    #[default = 5]
    pub padding: u16,
    pub background_color: Option<ColorWrap>,
}

macro_rules! default_serde_getters {
    ($($name:ident: $ty:ty = $val:expr),+ $(,)?) => {
        paste::paste! {
            $(fn [< default_ $name >]() -> $ty { $val }),+
        }
    };
}

#[derive(Deserialize)]
pub struct ModuleSettings {
    #[serde(default = "ModuleSettings::default_padding")]
    pub padding: u16,
    #[serde(flatten)]
    pub ty: ModuleTypeSettings,
}

impl ModuleSettings {
    default_serde_getters![padding: u16 = 5];
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ModuleTypeSettings {
    Clock(ClockSettings),
}

#[derive(Debug, Error, Diagnostic)]
pub enum ConfigError {
    #[error("[Config] [std::io] {0}")]
    IO(#[from] std::io::Error),

    #[error("[Config] [nickel_lang_core] {0}")]
    Nickel(String),
    #[error("[Config] [serde_json] {0}")]
    Json(#[from] serde_json::Error),
}
