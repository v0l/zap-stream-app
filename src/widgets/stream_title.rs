use crate::note_util::NoteUtil;
use crate::route::RouteServices;
use crate::stream_info::StreamInfo;
use crate::theme::MARGIN_DEFAULT;
use crate::widgets::zap::ZapButton;
use crate::widgets::Profile;
use egui::{Color32, Frame, Label, Response, RichText, TextWrapMode, Ui};
use nostrdb::Note;

pub struct StreamTitle<'a> {
    event: &'a Note<'a>,
}

impl<'a> StreamTitle<'a> {
    pub fn new(event: &'a Note<'a>) -> StreamTitle<'a> {
        StreamTitle { event }
    }
    pub fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        Frame::none()
            .outer_margin(MARGIN_DEFAULT)
            .show(ui, |ui| {
                let title = RichText::new(self.event.title().unwrap_or("Untitled"))
                    .size(20.)
                    .color(Color32::WHITE);
                ui.add(Label::new(title.strong()).wrap_mode(TextWrapMode::Wrap));

                ui.horizontal(|ui| {
                    Profile::new(self.event.host())
                        .size(32.)
                        .render(ui, services);
                    ZapButton::event(self.event).render(ui, services);
                });

                if let Some(summary) = self
                    .event
                    .get_tag_value("summary")
                    .and_then(|r| r.variant().str())
                {
                    if !summary.is_empty() {
                        let summary = RichText::new(summary).color(Color32::WHITE);
                        ui.add(Label::new(summary).wrap_mode(TextWrapMode::Wrap));
                    }
                }
            })
            .response
    }
}
