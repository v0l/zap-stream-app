use crate::link::NostrLink;
use crate::note_util::NoteUtil;
use crate::route::{RouteServices, Routes};
use crate::widgets::avatar::Avatar;
use crate::widgets::VideoPlaceholder;
use eframe::epaint::Vec2;
use egui::{Color32, Image, Label, Response, RichText, Rounding, Sense, TextWrapMode, Ui, Widget};
use nostrdb::{NdbStrVariant, Note};
use crate::stream_info::StreamInfo;

pub struct StreamEvent<'a> {
    event: &'a Note<'a>,
    picture: Option<Image<'a>>,
    services: &'a RouteServices<'a>,
}

impl<'a> StreamEvent<'a> {
    pub fn new(event: &'a Note<'a>, services: &'a RouteServices) -> Self {
        let image = event.get_tag_value("image");
        let cover = match image {
            Some(i) => match i.variant().str() {
                Some(i) => Some(services.img_cache.load(i)),
                None => None,
            },
            None => None,
        };
        Self {
            event,
            picture: cover,
            services,
        }
    }
}
impl Widget for StreamEvent<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.style_mut().spacing.item_spacing = Vec2::new(12., 16.);

            let host = self.event.host();
            let w = ui.available_width();
            let h = (w / 16.0) * 9.0;
            let img_size = Vec2::new(w, h);

            let img = match self.picture {
                Some(picture) => picture
                    .fit_to_exact_size(img_size)
                    .rounding(Rounding::same(12.))
                    .sense(Sense::click())
                    .ui(ui),
                None => VideoPlaceholder.ui(ui),
            };
            if img.clicked() {
                self.services.navigate(Routes::Event {
                    link: NostrLink::from_note(&self.event),
                    event: None,
                });
            }
            ui.horizontal(|ui| {
                ui.add(Avatar::pubkey(&host, self.services).size(40.));
                let title = RichText::new(self.event.title().unwrap_or("Untitled"))
                    .size(16.)
                    .color(Color32::WHITE);
                ui.add(Label::new(title).wrap_mode(TextWrapMode::Truncate));
            })
        })
            .response
    }
}
