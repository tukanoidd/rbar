use std::{io::Write, path::PathBuf};

use directories::ProjectDirs;
use iced_layershell::{reexport::Anchor, settings::LayerShellSettings};
use itertools::Itertools;
use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::module::{clock::ClockFormat, ModuleConfig};

#[derive(SmartDefault, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    #[default = true]
    pub top: bool,
    #[default((900, 50))]
    pub size: (u32, u32),

    pub left: Vec<ModuleConfig>,
    #[default(vec![ModuleConfig::Clock(Default::default())])]
    pub center: Vec<ModuleConfig>,
    pub right: Vec<ModuleConfig>,
}

impl Config {
    pub fn open(project_dirs: &ProjectDirs, path: Option<PathBuf>) -> miette::Result<Self> {
        let config_dir = project_dirs.config_dir();
        let path = match path {
            Some(path) => path,
            None => {
                if !config_dir.exists() {
                    tracing::warn!("Config dir {config_dir:?} doesn't exist, creating...");
                    std::fs::create_dir_all(config_dir).into_diagnostic()?;
                }

                config_dir.join("config.ron")
            }
        };

        match path.exists() {
            true => {
                let config: Config =
                    ron::from_str(&std::fs::read_to_string(path).into_diagnostic()?)
                        .into_diagnostic()?;

                let duplicates = config
                    .left
                    .iter()
                    .chain(config.center.iter())
                    .chain(config.right.iter())
                    .duplicates()
                    .map(ToString::to_string)
                    .collect_vec();

                if !duplicates.is_empty() {
                    return Err(miette::miette!("rbar doesn't support more than one instance of a module running (for now), please remove the duplicates: [{}]", duplicates.join(", ")))              ;
                }

                Ok(config)
            }
            false => {
                tracing::warn!("Config file {path:?} doesn't exist, creating default...");

                let config = Config::default();
                let config_str = ron::to_string(&config).into_diagnostic()?;

                let mut file = std::fs::File::create(path).into_diagnostic()?;
                file.write_all(config_str.as_bytes()).into_diagnostic()?;

                Ok(config)
            }
        }
    }

    pub fn layer_shell_settings(&self) -> LayerShellSettings {
        let Self { top, size, .. } = self;

        LayerShellSettings {
            anchor: (match top {
                true => Anchor::Top,
                false => Anchor::Bottom,
            }) | Anchor::Left
                | Anchor::Right,
            layer: iced_layershell::reexport::Layer::Top,
            exclusive_zone: size.1 as i32,
            size: Some(*size),
            keyboard_interactivity: iced_layershell::reexport::KeyboardInteractivity::None,
            ..Default::default()
        }
    }
}

#[derive(SmartDefault, Debug, Serialize, Deserialize)]
pub struct ClockConfig {
    pub format: ClockFormat,
}
