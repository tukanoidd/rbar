use clap::Parser;

/// A wayland bar
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub debug: bool,
    #[arg(short, long)]
    pub trace: bool,
}
