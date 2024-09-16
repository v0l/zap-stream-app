use crate::services::Services;
use crate::widgets::avatar::Avatar;
use eframe::emath::Align;
use eframe::epaint::Vec2;
use egui::{Frame, Image, Layout, Margin, Response, Ui, Widget};
use nostr_sdk::PublicKey;

pub struct Header<'a> {
    services: &'a Services,
}

impl<'a> Header<'a> {
    pub fn new(services: &'a Services) -> Self {
        Self { services }
    }
}

impl Widget for Header<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let login = PublicKey::from_hex("63fe6318dc58583cfe16810f86dd09e18bfd76aabc24a0081ce2856f330504ed").unwrap();
        let logo_bytes = include_bytes!("../logo.svg");
        Frame::none()
            .outer_margin(Margin::symmetric(16., 8.))
            .show(ui, |ui| {
                ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 32.), Layout::left_to_right(Align::Center), |ui| {
                    ui.style_mut()
                        .spacing.item_spacing.x = 16.;
                    Image::from_bytes("logo.svg", logo_bytes)
                        .max_height(22.62).ui(ui);
                    ui.add(Avatar::public_key(&self.services, &login));
                })
            }).response
    }
}