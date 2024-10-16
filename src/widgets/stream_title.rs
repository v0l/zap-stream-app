use crate::note_util::NoteUtil;
use crate::route::RouteServices;
use crate::stream_info::StreamInfo;
use crate::widgets::{NostrWidget, Profile};
use egui::{Color32, Frame, Label, Margin, Response, RichText, TextWrapMode, Ui, Widget};
use nostrdb::Note;

pub struct StreamTitle<'a> {
    event: &'a Note<'a>,
}

impl<'a> StreamTitle<'a> {
    pub fn new(event: &'a Note<'a>) -> StreamTitle {
        StreamTitle {
            event
        }
    }
}

impl<'a> NostrWidget for StreamTitle<'a> {
    fn render(&mut self, ui: &mut Ui, services: &RouteServices<'_>) -> Response {
        Frame::none()
            .outer_margin(Margin::symmetric(12., 8.))
            .show(ui, |ui| {
                ui.style_mut().spacing.item_spacing.y = 8.;
                let title = RichText::new(self.event.title().unwrap_or("Untitled"))
                    .size(20.)
                    .color(Color32::WHITE);
                ui.add(Label::new(title.strong()).wrap_mode(TextWrapMode::Truncate));

                Profile::new(self.event.host(), services)
                    .size(32.)
                    .ui(ui);

                if let Some(summary) = self.event.get_tag_value("summary").map_or(None, |r| r.variant().str()) {
                    let summary = RichText::new(summary)
                        .color(Color32::WHITE);
                    ui.add(Label::new(summary).wrap_mode(TextWrapMode::Truncate));
                }
            }).response
    }
}