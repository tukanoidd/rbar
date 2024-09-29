use iced::widget::{rich_text, row, span};
use iced_fonts::{nerd, Nerd};
use miette::IntoDiagnostic;
use starship_battery::{Battery as SBattery, Manager, State};

use crate::app::AppMsg;

use super::{NoConfig, TModule};

#[derive(Debug)]
pub struct Battery(Vec<BatteryData>);

impl TModule for Battery {
    type Config = NoConfig;
    type Event = BatteryEvent;

    fn new(_config: Self::Config) -> Self {
        Self(vec![])
    }

    fn update(&mut self, event: Self::Event) -> Option<AppMsg> {
        match event {
            BatteryEvent::SetData(data) => self.0 = data,
        }

        None
    }

    fn view(&self) -> iced::Element<'_, Self::Event, iced::Theme, iced::Renderer> {
        row(self.0.iter().map(|BatteryData { level, icon }| {
            rich_text![
                span(nerd::icon_to_string(*icon)).font(iced_fonts::NERD_FONT),
                span(format!(" {level}%"))
            ]
            .into()
        }))
        .spacing(5)
        .into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatteryEvent {
    SetData(Vec<BatteryData>),
}

#[derive(Debug)]
pub struct BatteryInfo {
    batteries: Vec<SBattery>,
}

impl BatteryInfo {
    pub fn init() -> miette::Result<Self> {
        let manager = Manager::new().into_diagnostic()?;
        let batteries = manager
            .batteries()
            .into_diagnostic()?
            .collect::<Result<Vec<_>, _>>()
            .into_diagnostic()?;

        Ok(Self { batteries })
    }

    pub fn data(&self) -> impl Iterator<Item = BatteryData> + '_ {
        self.batteries.iter().map(|battery| {
            let level = (battery.state_of_charge().value * 100.0).round() as u8;
            let icon = match battery.state() {
                State::Unknown | State::Discharging => match level {
                    0..10 => Nerd::BatteryOutline,
                    10..20 => Nerd::BatteryOnezero,
                    20..30 => Nerd::BatteryTwozero,
                    30..40 => Nerd::BatteryThreezero,
                    40..50 => Nerd::BatteryFourzero,
                    50..60 => Nerd::BatteryFivezero,
                    60..70 => Nerd::BatterySixzero,
                    70..80 => Nerd::BatterySevenzero,
                    80..90 => Nerd::BatteryEightzero,
                    90..100 => Nerd::BatteryNinezero,
                    _ => Nerd::Battery,
                },
                State::Charging => match level {
                    0..10 => Nerd::BatteryChargingOutline,
                    10..20 => Nerd::BatteryChargingOnezero,
                    20..30 => Nerd::BatteryChargingTwozero,
                    30..40 => Nerd::BatteryChargingThreezero,
                    40..50 => Nerd::BatteryChargingFourzero,
                    50..60 => Nerd::BatteryChargingFivezero,
                    60..70 => Nerd::BatteryChargingSixzero,
                    70..80 => Nerd::BatteryChargingSevenzero,
                    80..90 => Nerd::BatteryChargingEightzero,
                    90..100 => Nerd::BatteryChargingNinezero,
                    _ => Nerd::BatteryCharging,
                },
                State::Empty => todo!(),
                State::Full => todo!(),
            };

            BatteryData { level, icon }
        })
    }
}

impl PartialEq for BatteryInfo {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BatteryData {
    level: u8,
    icon: Nerd,
}

macro_rules! battery_icons_eq {
    ($icon:expr, $other_icon:expr => $($name:ident),+) => {
        paste::paste! {
            match ($icon, $other_icon) {
                (Nerd::Battery, Nerd::Battery)
                | (Nerd::BatteryCharging, Nerd::BatteryCharging)
                $(
                    | (Nerd::[< Battery $name >], Nerd::[< Battery $name >])
                    | (
                        Nerd::[< BatteryCharging $name >],
                        Nerd::[< BatteryCharging $name >]
                    ) => true,
                 )+
                _ => false
            }

        }
    }
}

impl PartialEq for BatteryData {
    fn eq(&self, other: &Self) -> bool {
        let Self { level, icon } = self;
        let Self {
            level: other_level,
            icon: other_icon,
        } = other;

        (level == other_level)
            && (battery_icons_eq!(
            icon,
            other_icon =>
                Outline,
                Onezero, Twozero, Threezero,
                Fourzero, Fivezero, Sixzero,
                Sevenzero, Eightzero, Ninezero
            ))
    }
}
