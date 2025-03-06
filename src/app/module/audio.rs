use std::sync::Arc;

use iced::{futures::lock::Mutex, widget::text, Task};

use crate::app::AppMsg;

use super::TModule;

pub struct Audio {}

impl<'a> TModule<'a> for Audio {
    type Settings = ();
    type ViewDeps = &'a Audio;
    type UpdateDeps = &'a mut Audio;
    type Msg = AudioMsg;

    fn new(settings: Self::Settings) -> Self {
        Self {}
    }

    fn view(&self, id: uuid::Uuid, deps: Self::ViewDeps) -> iced::Element<'_, AppMsg> {
        let cards = deps.cards.iter().flat_map();

        text("todo").into()
    }

    fn update(&mut self, id: uuid::Uuid, msg: Self::Msg, deps: Self::UpdateDeps) -> Task<AppMsg> {
        // match msg {
        //     _ => {}
        // }

        Task::none()
    }
}

#[derive(Debug, Clone)]
pub enum AudioMsg {}
