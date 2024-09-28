use iced::widget::{container, rich_text, row, span};
use iced_fonts::{nerd, Nerd};
use miette::IntoDiagnostic;
use starship_battery::{Battery as SBattery, Manager, State};

use crate::app::AppMsg;

use super::{NoConfig, NoEvent, TModule};

pub type BatteryEvent = NoEvent;

pub struct Battery(Vec<BatteryData>);

impl Battery {
    pub fn set(&mut self, data: Vec<BatteryData>) {
        self.0 = data;
    }
}

impl TModule for Battery {
    type Config = NoConfig;
    type Event = BatteryEvent;

    fn new(_config: Self::Config) -> Self {
        Self(vec![])
    }

    fn update(&mut self, _event: Self::Event) -> Option<AppMsg> {
        // match event {
        //     _ => {}
        // }

        None
    }

    fn view(&self) -> iced::Element<'_, Self::Event, iced::Theme, iced::Renderer> {
        container(
            row(self.0.iter().map(|BatteryData { level, icon }| {
                rich_text![span(nerd::icon_to_string(*icon)), span(format!("{level}%"))].into()
            }))
            .spacing(5),
        )
        .style(container::rounded_box)
        .into()
    }
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

#[derive(Clone, Copy)]
pub struct BatteryData {
    level: u8,
    icon: Nerd,
}
