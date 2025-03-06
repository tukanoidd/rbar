mod app;
mod cli;
mod config;
mod util;

use clap::Parser;
use directories::ProjectDirs;
use iced_layershell::{
    reexport::{KeyboardInteractivity, Layer},
    settings::{LayerShellSettings, Settings},
    Application,
};
use miette::IntoDiagnostic;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use app::{App, AppFlags};
use cli::Cli;
use config::Config;
use util::{audio::Audio, wayland::Wayland};

fn main() -> miette::Result<()> {
    let Cli { debug, trace } = Cli::parse();

    let wayland = Wayland::new()?;
    let audio = Audio::new()?;

    init_logging(debug, trace)?;

    let dirs = ProjectDirs::from("com", "tukanoidd", "rbar")
        .ok_or_else(|| miette::miette!("Failed to initialize project directories"))?;

    let config = Config::open_or_default(dirs.config_dir())?;

    let Settings {
        layer_settings:
            LayerShellSettings {
                margin,
                start_mode,
                events_transparent,
                ..
            },
        fonts,
        default_font,
        default_text_size,
        antialiasing,
        virtual_keyboard_support,
        ..
    } = Settings::<()>::default();

    App::run(Settings {
        id: Some("com.tukanoidd.rbar".into()),
        layer_settings: LayerShellSettings {
            anchor: config.main.side.into(),
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::None,
            exclusive_zone: config.main.height as i32,
            size: Some((config.main.width, config.main.height)),
            margin,
            start_mode,
            events_transparent,
        },
        flags: AppFlags {
            wayland,
            audio,
            config,
        },
        fonts,
        default_font,
        default_text_size,
        antialiasing,
        virtual_keyboard_support,
    })
    .into_diagnostic()?;

    Ok(())
}

fn init_logging(debug: bool, trace: bool) -> miette::Result<()> {
    let level = format!(
        "rbar={}",
        trace
            .then_some("trace")
            .or_else(|| (cfg!(debug_assertions) || debug).then_some("debug"))
            .unwrap_or("info")
    );

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(tracing_subscriber::EnvFilter::new(level))
        .try_init()
        .into_diagnostic()?;

    tracing::debug!("Logging initialized!");

    Ok(())
}
