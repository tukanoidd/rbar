use std::{ffi::CString, io::BufReader, os::unix::net::UnixStream, sync::Arc};

use iced::widget::{rich_text, span};
use iced_fonts::Nerd;
use miette::IntoDiagnostic;
use pulseaudio::protocol;
use tokio::sync::Mutex;

use super::{NoConfig, TModule};

#[derive(Debug)]
pub struct Audio {
    default: Option<usize>,
    data: Vec<AudioData>,
}

impl TModule for Audio {
    type Config = NoConfig;
    type Event = AudioEvent;

    fn new(_config: Self::Config) -> Self {
        Self {
            default: None,
            data: vec![],
        }
    }

    fn update(&mut self, event: Self::Event) -> Option<crate::app::AppMsg> {
        match event {
            AudioEvent::SetData(info) => {
                let info = info.blocking_lock();

                self.default = info.default_device_index();
                self.data = info.devices.iter().map(AudioData::new).collect();

                tracing::debug!("{self:#?}");
            }
        }

        None
    }

    fn view(&self) -> iced::Element<'_, Self::Event, iced::Theme, iced::Renderer> {
        let default_device = match self.default.and_then(|i| self.data.get(i)) {
            Some(data) => data,
            None => &AudioData::unknown(),
        };

        rich_text![
            span(iced_fonts::nerd::icon_to_string(default_device.icon)).font(iced_fonts::NERD_FONT),
            span(format!(" {}%", default_device.volume))
        ]
        .into()
    }
}

#[derive(Debug, Clone)]
pub enum AudioEvent {
    SetData(Arc<Mutex<AudioInfo>>),
}

impl PartialEq for AudioEvent {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct AudioData {
    icon: Nerd,
    volume: u8,
}

impl AudioData {
    fn new(device_info: &AudioDevice) -> Self {
        Self {
            icon: device_info.icon(),
            volume: device_info.volume,
        }
    }

    fn unknown() -> Self {
        Self {
            icon: Nerd::VolumeOff,
            volume: 0,
        }
    }
}

#[derive(Debug)]
pub struct AudioInfo {
    sock: BufReader<UnixStream>,
    server_info: AudioServerInfo,
    devices: Vec<AudioDevice>,
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

        protocol::write_command_message(
            sock.get_mut(),
            2,
            protocol::Command::GetServerInfo,
            protocol_version,
        )
        .into_diagnostic()?;

        let (_, server_info) = protocol::read_reply_message::<protocol::command::ServerInfo>(
            &mut sock,
            protocol_version,
        )
        .into_diagnostic()?;
        let server_info = AudioServerInfo::new(server_info)?;

        protocol::write_command_message(
            sock.get_mut(),
            3,
            protocol::Command::GetSinkInfoList,
            protocol_version,
        )
        .into_diagnostic()?;

        let (_, sinks) =
            protocol::read_reply_message::<protocol::SinkInfoList>(&mut sock, protocol_version)
                .into_diagnostic()?;

        let devices = sinks
            .into_iter()
            .map(AudioDevice::new)
            .collect::<miette::Result<Vec<_>>>()?;

        Ok(Self {
            sock,
            server_info,
            devices,
        })
    }

    fn default_device_index(&self) -> Option<usize> {
        self.devices
            .iter()
            .position(|d| d.name == self.server_info.default_device)
    }
}

#[derive(Debug)]
pub struct AudioServerInfo {
    server: protocol::ServerInfo,
    default_device: String,
}

impl AudioServerInfo {
    fn new(server: protocol::ServerInfo) -> miette::Result<Self> {
        let default_device = server
            .default_sink_name
            .as_ref()
            .ok_or_else(|| miette::miette!("Failed to get default audio device"))?
            .to_str()
            .into_diagnostic()?
            .to_string();

        Ok(Self {
            server,
            default_device,
        })
    }
}

#[derive(Debug)]
pub struct AudioDevice {
    sink: protocol::SinkInfo,

    name: String,
    description: String,

    muted: bool,
    volume: u8,
}

impl AudioDevice {
    fn new(sink: protocol::SinkInfo) -> miette::Result<Self> {
        let name = sink.name.to_str().into_diagnostic()?.to_string();
        let description = match &sink.description {
            Some(str) => Some(str.to_str().into_diagnostic()?.to_string()),
            None => None,
        }
        .unwrap_or_else(|| "Unknown".into());

        let muted = sink.muted;
        let volume = (sink.base_volume.to_linear() * 100.0).round() as u8;

        tracing::debug!("{name} ({description}) [{muted} {volume}]");

        Ok(Self {
            sink,

            name,
            description,

            muted,
            volume,
        })
    }

    fn icon(&self) -> Nerd {
        if self.muted {
            return Nerd::VolumeMute;
        }

        match self.volume {
            0 => Nerd::VolumeOff,
            1..=33 => Nerd::VolumeLow,
            34..=66 => Nerd::VolumeMedium,
            _ => Nerd::VolumeHigh,
        }
    }
}
