use crate::services::Services;
use crate::widgets::stream::StreamEvent;
use egui::{Frame, Margin, Response, Ui, Widget};
use egui_extras::Column;
use nostr_sdk::Event;

pub struct StreamList<'a> {
    streams: &'a Vec<Event>,
    services: &'a Services,
}

impl<'a> StreamList<'a> {
    pub fn new(streams: &'a Vec<Event>, services: &'a Services) -> Self {
        Self { streams, services }
    }
}

impl Widget for StreamList<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        Frame::none()
            .inner_margin(Margin::symmetric(16., 8.))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(0., 20.0);
                    for event in self.streams.iter().take(5) {
                        ui.add(StreamEvent::new(event, self.services));
                    }
                })
            }).response
    }
}