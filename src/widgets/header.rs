use crate::route::{RouteServices, Routes};
use crate::widgets::avatar::Avatar;
use eframe::emath::Align;
use eframe::epaint::Vec2;
use egui::{Frame, Image, Layout, Margin, Response, Sense, Ui, Widget};
use nostr_sdk::util::hex;

pub struct Header<'a> {
    services: &'a RouteServices<'a>,
}

impl<'a> Header<'a> {
    pub fn new(services: &'a RouteServices) -> Self {
        Self { services }
    }
}

impl Widget for Header<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let login: [u8; 32] = hex::decode("63fe6318dc58583cfe16810f86dd09e18bfd76aabc24a0081ce2856f330504ed").unwrap().try_into().unwrap();
        let logo_bytes = include_bytes!("../logo.svg");
        Frame::none()
            .outer_margin(Margin::symmetric(16., 8.))
            .show(ui, |ui| {
                ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), 32.), Layout::left_to_right(Align::Center), |ui| {
                    ui.style_mut()
                        .spacing.item_spacing.x = 16.;
                    if Image::from_bytes("logo.svg", logo_bytes)
                        .max_height(22.62)
                        .sense(Sense::click())
                        .ui(ui).clicked() {
                        self.services.navigate(Routes::HomePage);
                    }
                    ui.add(Avatar::public_key(&self.services.profile, &login));
                })
            }).response
    }
}