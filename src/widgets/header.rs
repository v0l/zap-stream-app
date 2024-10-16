use crate::route::{RouteServices, Routes};
use crate::widgets::avatar::Avatar;
use crate::widgets::NostrWidget;
use eframe::emath::Align;
use eframe::epaint::Vec2;
use egui::{Frame, Image, Layout, Margin, Response, Sense, Ui, Widget};

pub struct Header;

impl Header {
    pub fn new() -> Self {
        Self {}
    }
}

impl NostrWidget for Header {
    fn render(&mut self, ui: &mut Ui, services: &RouteServices<'_>) -> Response {
        let logo_bytes = include_bytes!("../resources/logo.svg");
        Frame::none()
            .outer_margin(Margin::symmetric(16., 8.))
            .show(ui, |ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), 32.),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.style_mut().spacing.item_spacing.x = 16.;
                        if Image::from_bytes("logo.svg", logo_bytes)
                            .max_height(22.62)
                            .sense(Sense::click())
                            .ui(ui)
                            .clicked()
                        {
                            services.navigate(Routes::HomePage);
                        }
                        if let Some(pk) = services.login {
                            ui.add(Avatar::pubkey(pk, services));
                        }
                    },
                )
            })
            .response
    }
}
