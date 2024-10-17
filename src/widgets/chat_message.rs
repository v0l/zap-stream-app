use crate::route::RouteServices;
use crate::services::ndb_wrapper::SubWrapper;
use crate::stream_info::StreamInfo;
use crate::theme::{NEUTRAL_500, PRIMARY};
use crate::widgets::Avatar;
use eframe::epaint::text::TextWrapMode;
use egui::text::LayoutJob;
use egui::{Align, Color32, Label, Response, TextFormat, Ui, Widget};
use nostrdb::{NdbProfile, Note};

pub struct ChatMessage<'a> {
    stream: &'a Note<'a>,
    ev: &'a Note<'a>,
    services: &'a RouteServices<'a>,
    profile: (Option<NdbProfile<'a>>, Option<SubWrapper>),
}

impl<'a> ChatMessage<'a> {
    pub fn new(stream: &'a Note<'a>, ev: &'a Note<'a>, services: &'a RouteServices<'a>) -> ChatMessage<'a> {
        ChatMessage { stream, ev, services, profile: services.ndb.fetch_profile(services.tx, ev.pubkey()) }
    }
}

impl<'a> Widget for ChatMessage<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal_wrapped(|ui| {
            let mut job = LayoutJob::default();

            let is_host = self.stream.host().eq(self.ev.pubkey());
            let profile = self.services.ndb.get_profile_by_pubkey(self.services.tx, self.ev.pubkey())
                .map_or(None, |p| p.record().profile());

            let name = profile
                .map_or("Nostrich", |f| f.name().map_or("Nostrich", |f| f));

            let name_color = if is_host {
                PRIMARY
            } else {
                NEUTRAL_500
            };

            let mut format = TextFormat::default();
            format.line_height = Some(24.0);
            format.valign = Align::Center;

            format.color = name_color;
            job.append(name, 0.0, format.clone());
            format.color = Color32::WHITE;
            job.append(self.ev.content(), 5.0, format.clone());

            ui.add(Avatar::from_profile(profile ,self.services.img_cache).size(24.));
            ui.add(Label::new(job)
                .wrap_mode(TextWrapMode::Wrap)
            );
        }).response
    }
}
