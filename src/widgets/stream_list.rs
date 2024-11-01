use crate::note_store::NoteStore;
use crate::route::RouteServices;
use crate::stream_info::StreamInfo;
use crate::widgets::stream_tile::StreamEvent;
use egui::{Frame, Margin, Response, Ui, Widget};
use itertools::Itertools;

pub struct StreamList<'a> {
    streams: &'a NoteStore<'a>,
    services: &'a RouteServices<'a>,
}

impl<'a> StreamList<'a> {
    pub fn new(streams: &'a NoteStore<'a>, services: &'a RouteServices) -> Self {
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
                    for event in self.streams.iter().sorted_by(|a, b| {
                        a.status()
                            .cmp(&b.status())
                            .then(a.starts().cmp(&b.starts()).reverse())
                    }) {
                        ui.add(StreamEvent::new(event, self.services));
                    }
                })
            })
            .response
    }
}
