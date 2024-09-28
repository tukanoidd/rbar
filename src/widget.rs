pub mod button;
pub mod custom;

use button::{ButtonEvent, ButtonWidget};
use custom::CustomWidget;
use iced::widget::{self, Component};
use serde::{Deserialize, Serialize};

use crate::app::AppMsg;

#[derive(Serialize, Deserialize)]
pub struct BarWidget {
    #[serde(rename = "type")]
    ty: WidgetType,
    styling: Option<WidgetStyling>,
}

impl Component<AppMsg> for BarWidget {
    type State = ();
    type Event = BarWidgetEvent;

    fn update(&mut self, _state: &mut Self::State, _event: Self::Event) -> Option<AppMsg> {
        None
    }

    fn view(
        &self,
        _state: &Self::State,
    ) -> iced_style::core::Element<'_, Self::Event, iced_style::Theme, iced_renderer::Renderer>
    {
        widget::container::<Self::Event, _, _>(match &self.ty {
            WidgetType::Button(button) => button.view(&()).map(Into::into),
            WidgetType::Custom(custom) => custom.view(&()).map(Into::into),
        })
        .into()
    }
}

pub struct BarWidgetEvent;

macro_rules! from_widget_event {
    ($($widget_event:ty),+) => {$(
        impl From<$widget_event> for BarWidgetEvent {
            fn from(_: $widget_event) -> Self {
                Self
            }
        }
    )*};
}

from_widget_event![ButtonEvent, ()];

#[derive(Serialize, Deserialize)]
pub enum WidgetType {
    Button(ButtonWidget),
    Custom(CustomWidget),
}

#[derive(Serialize, Deserialize)]
pub struct WidgetStyling {}
