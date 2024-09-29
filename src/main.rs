#![feature(box_into_inner)]

mod app;
mod config;
mod module;
mod util;

use std::path::PathBuf;

use clap::Parser;
use config::Config;
use directories::ProjectDirs;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Rusty Bar
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    config: Option<PathBuf>,
    #[arg(long)]
    debug: bool,
    #[arg(long)]
    trace: bool,
}

fn main() -> miette::Result<()> {
    let Cli {
        config,
        debug,
        trace,
    } = Cli::parse();

    init_logging(debug, trace);

    let project_dirs = ProjectDirs::from("com", "tukanoidd", "rbar")
        .ok_or_else(|| miette::miette!("Failed to initialize ProjectDirs"))?;

    let config = Config::open(&project_dirs, config)?;

    app::run(config)
}

fn init_logging(debug: bool, trace: bool) {
    let level = trace
        .then_some("trace")
        .or_else(|| (debug || cfg!(debug_assertions)).then_some("debug"))
        .unwrap_or("info");

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(tracing_subscriber::filter::EnvFilter::new(format!(
            "rbar={level}"
        )))
        .init();
}
