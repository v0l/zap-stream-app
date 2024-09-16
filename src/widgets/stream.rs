use crate::services::Services;
use crate::widgets::avatar::Avatar;
use eframe::epaint::Vec2;
use egui::{Color32, Image, Rect, Response, RichText, Rounding, Sense, Ui, Widget};
use nostr_sdk::{Alphabet, Event, PublicKey, SingleLetterTag, TagKind};

pub struct StreamEvent<'a> {
    event: &'a Event,
    picture: Option<Image<'a>>,
    services: &'a Services,
}

impl<'a> StreamEvent<'a> {
    pub fn new(event: &'a Event, services: &'a Services) -> Self {
        let image = event.get_tag_content(TagKind::Image);
        let cover = match image {
            Some(i) => Some(Image::from_uri(i)),
            None => None,
        };
        Self { event, picture: cover, services }
    }
}
impl Widget for StreamEvent<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            ui.style_mut()
                .spacing.item_spacing = Vec2::new(12., 16.);

            let host = match self.event.tags.iter().find(|t| t.kind() == TagKind::SingleLetter(SingleLetterTag::lowercase(Alphabet::P)) && t.as_vec()[3] == "host") {
                Some(t) => PublicKey::from_hex(t.as_vec().get(1).unwrap()).unwrap(),
                None => self.event.author()
            };
            match self.picture {
                Some(picture) => picture.rounding(Rounding::same(12.)).ui(ui),
                None => {
                    let w = ui.available_width();
                    let h = (w / 16.0) * 9.0;
                    let (response, painter) = ui.allocate_painter(Vec2::new(w, h), Sense::hover());
                    painter.rect_filled(Rect::EVERYTHING, Rounding::same(12.), Color32::from_rgb(200, 200, 200));
                    response
                }
            };
            ui.horizontal(|ui| {
                ui.add(Avatar::public_key(self.services, &host).size(40.));
                ui.label(RichText::new(self.event.get_tag_content(TagKind::Title).unwrap_or("Unknown"))
                    .size(16.)
                    .color(Color32::WHITE)
                );
            })
        }).response
    }
}