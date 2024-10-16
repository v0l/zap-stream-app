use crate::route::RouteServices;
use crate::widgets::stream_tile::StreamEvent;
use egui::{Frame, Margin, Response, Ui, Widget};
use nostrdb::Note;

pub struct StreamList<'a> {
    streams: &'a Vec<Note<'a>>,
    services: &'a RouteServices<'a>,
}

impl<'a> StreamList<'a> {
    pub fn new(streams: &'a Vec<Note<'a>>, services: &'a RouteServices) -> Self {
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
                    for event in self.streams {
                        ui.add(StreamEvent::new(event, self.services));
                    }
                })
            })
            .response
    }
}
