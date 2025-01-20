use crate::link::NostrLink;
use crate::route::{RouteServices, RouteType};
use crate::widgets::avatar::Avatar;
use crate::widgets::Button;
use eframe::emath::Align;
use eframe::epaint::Vec2;
use egui::{CursorIcon, Frame, Image, Layout, Margin, Response, Sense, Ui, Widget};
use nostrdb::Transaction;

pub struct Header;

impl Header {
    pub fn new() -> Self {
        Self {}
    }
    pub fn render(
        &mut self,
        ui: &mut Ui,
        services: &mut RouteServices<'_, '_>,
        tx: &Transaction,
    ) -> Response {
        let logo_bytes = include_bytes!("../resources/logo.svg");
        Frame::none()
            .outer_margin(Margin::symmetric(16., 8.))
            .show(ui, |ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), 32.),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.style_mut().spacing.item_spacing.x = 16.;
                        if Image::from_bytes("header_logo.svg", logo_bytes)
                            .max_height(24.)
                            .sense(Sense::click())
                            .ui(ui)
                            .on_hover_and_drag_cursor(CursorIcon::PointingHand)
                            .clicked()
                        {
                            services.navigate(RouteType::HomePage);
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if let Some(acc) = services.ctx.accounts.get_selected_account() {
                                if Avatar::pubkey(&acc.pubkey, services.ctx.ndb, tx)
                                    .render(ui, services.ctx.img_cache)
                                    .clicked()
                                {
                                    services.navigate(RouteType::ProfilePage {
                                        link: NostrLink::profile(acc.pubkey.bytes()),
                                    })
                                }
                            } else if Button::new().show(ui, |ui| ui.label("Login")).clicked() {
                                services.navigate(RouteType::LoginPage);
                            }
                        });
                    },
                )
            })
            .response
    }
}
