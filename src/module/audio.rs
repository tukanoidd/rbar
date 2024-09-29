use std::{ffi::CString, io::BufReader, os::unix::net::UnixStream};

use iced::widget::text;
use miette::IntoDiagnostic;
use pulseaudio::protocol;

use super::{NoConfig, TModule};

#[derive(Debug)]
pub struct Audio {}

impl Audio {}

impl TModule for Audio {
    type Config = NoConfig;
    type Event = AudioEvent;

    fn new(config: Self::Config) -> Self {
        Self {}
    }

    fn update(&mut self, event: Self::Event) -> Option<crate::app::AppMsg> {
        match event {
            _ => {}
        }

        None
    }

    fn view(&self) -> iced::Element<'_, Self::Event, iced::Theme, iced::Renderer> {
        text("Audio").into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioEvent {}

#[derive(Debug)]
pub struct AudioInfo {
    sock: BufReader<UnixStream>,
    sinks: Vec<protocol::SinkInfo>,
}

impl AudioInfo {
    pub async fn init() -> miette::Result<Self> {
        let socket_path = pulseaudio::socket_path_from_env()
            .ok_or_else(|| miette::miette!("PulseAudio is not available"))?;
        let mut sock = BufReader::new(UnixStream::connect(socket_path).into_diagnostic()?);

        let cookie = tokio::fs::read(
            pulseaudio::cookie_path_from_env()
                .ok_or_else(|| miette::miette!("Failed to get cookie path"))?,
        )
        .await
        .into_diagnostic()?;
        let auth = protocol::AuthParams {
            version: protocol::MAX_VERSION,
            supports_shm: false,
            supports_memfd: false,
            cookie,
        };

        protocol::write_command_message(
            sock.get_mut(),
            0,
            protocol::Command::Auth(auth),
            protocol::MAX_VERSION,
        )
        .into_diagnostic()?;
        let (_, auth_info) =
            protocol::read_reply_message::<protocol::AuthReply>(&mut sock, protocol::MAX_VERSION)
                .into_diagnostic()?;
        let protocol_version = std::cmp::min(protocol::MAX_VERSION, auth_info.version);

        let mut props = protocol::Props::new();
        props.set(
            protocol::Prop::ApplicationName,
            CString::new("rbar").into_diagnostic()?,
        );
        protocol::write_command_message(
            sock.get_mut(),
            1,
            protocol::Command::SetClientName(props),
            protocol_version,
        )
        .into_diagnostic()?;

        let _ = protocol::read_reply_message::<protocol::SetClientNameReply>(
            &mut sock,
            protocol_version,
        )
        .into_diagnostic()?;

        // Finally, write a command to get the list of sinks. The reply contains the information we're after.
        protocol::write_command_message(
            sock.get_mut(),
            2,
            protocol::Command::GetSinkInfoList,
            protocol_version,
        )
        .into_diagnostic()?;

        let (_, sinks) =
            protocol::read_reply_message::<protocol::SinkInfoList>(&mut sock, protocol_version)
                .into_diagnostic()?;

        for info in &sinks {
            tracing::debug!("{:#?}", info);
        }

        Ok(Self { sock, sinks })
    }
}
