use crate::route::RouteServices;
use crate::widgets::Profile;
use eframe::epaint::Vec2;
use egui::{Response, Ui, Widget};
use nostrdb::Note;

pub struct ChatMessage<'a> {
    ev: &'a Note<'a>,
    services: &'a RouteServices<'a>,
}

impl<'a> ChatMessage<'a> {
    pub fn new(ev: &'a Note<'a>, services: &'a RouteServices<'a>) -> ChatMessage<'a> {
        ChatMessage { ev, services }
    }
}

impl<'a> Widget for ChatMessage<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8., 2.);
            let author = self.ev.pubkey();
            Profile::new(author, self.services)
                .size(24.)
                .ui(ui);

            let content = self.ev.content();
            ui.label(content);
        }).response
    }
}