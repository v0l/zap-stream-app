use crate::link::NostrLink;
use crate::note_util::{NoteUtil, OwnedNote};
use crate::route::{RouteAction, RouteServices, Routes};
use crate::stream_info::StreamInfo;
use crate::widgets::StreamPlayer;
use egui::{Color32, Label, Response, RichText, TextWrapMode, Ui, Widget};
use nostrdb::Note;
use std::ptr;

pub struct StreamPage<'a> {
    services: &'a mut RouteServices<'a>,
    link: &'a NostrLink,
    event: &'a Option<OwnedNote>,
}

impl<'a> StreamPage<'a> {
    pub fn new(services: &'a mut RouteServices<'a>, link: &'a NostrLink, event: &'a Option<OwnedNote>) -> Self {
        Self { services, link, event }
    }
}

impl<'a> Widget for StreamPage<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let event = if let Some(event) = self.event {
            Note::Owned {
                ptr: ptr::null_mut(),
                size: 0,
            }
        } else {
            let mut q = self.services.ndb.query(self.services.tx, &[
                self.link.try_into().unwrap()
            ], 1).unwrap();
            let [e] = q.try_into().unwrap();
            e.note
        };

        if let Some(stream) = event.stream() {
            if self.services.player.is_none() {
                self.services.navigate(Routes::Action(RouteAction::StartPlayer(stream)));
            }
        }
        StreamPlayer::new(self.services).ui(ui);
        let title = RichText::new(match event.get_tag_value("title") {
            Some(s) => s.variant().str().unwrap_or("Unknown"),
            None => "Unknown",
        })
            .size(16.)
            .color(Color32::WHITE);
        ui.add(Label::new(title).wrap_mode(TextWrapMode::Truncate))
    }
}