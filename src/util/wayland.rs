use miette::Diagnostic;
use smithay_client_toolkit::reexports::client::{ConnectError, Connection};
use thiserror::Error;

pub struct Wayland {
    connection: Connection,
}

impl Wayland {
    pub fn new() -> Result<Self, WaylandError> {
        tracing::debug!("Initializing wayland connection...");

        let connection = Connection::connect_to_env()?;

        tracing::debug!("Wayland connection initialized...");

        Ok(Self { connection })
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum WaylandError {
    #[error("[Wayland] [wayland_client::conn]")]
    Connection(#[from] ConnectError),
}
