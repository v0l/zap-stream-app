use crate::services::Services;
use crate::widgets::avatar::Avatar;
use eframe::epaint::Vec2;
use egui::{Color32, Image, Label, Rect, Response, RichText, Rounding, Sense, TextWrapMode, Ui, Widget};
use log::info;
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
            let w = ui.available_width();
            let h = (w / 16.0) * 9.0;
            let img_size = Vec2::new(w, h);

            let img = match self.picture {
                Some(picture) => picture.fit_to_exact_size(img_size).rounding(Rounding::same(12.)).sense(Sense::click()).ui(ui),
                None => {
                    let (response, painter) = ui.allocate_painter(img_size, Sense::click());
                    painter.rect_filled(Rect::EVERYTHING, Rounding::same(12.), Color32::from_rgb(200, 200, 200));
                    response
                }
            };
            if img.clicked() {
                info!("Navigating to {}", self.event.id);
            }
            ui.horizontal(|ui| {
                ui.add(Avatar::public_key(self.services, &host).size(40.));
                let title = RichText::new(self.event.get_tag_content(TagKind::Title).unwrap_or("Unknown"))
                    .size(16.)
                    .color(Color32::WHITE);
                ui.add(Label::new(title).wrap_mode(TextWrapMode::Truncate));
            })
        }).response
    }
}