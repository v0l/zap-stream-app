use crate::route::{RouteServices, RouteType};
use crate::widgets::avatar::Avatar;
use crate::widgets::{Button, NostrWidget};
use eframe::emath::Align;
use eframe::epaint::Vec2;
use egui::{CursorIcon, Frame, Layout, Margin, Response, Sense, Ui, Widget};

pub struct Header;

impl Header {
    pub fn new() -> Self {
        Self {}
    }
}

impl NostrWidget for Header {
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        let logo_bytes = include_bytes!("../resources/logo.svg");
        Frame::none()
            .outer_margin(Margin::symmetric(16., 8.))
            .show(ui, |ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), 32.),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.style_mut().spacing.item_spacing.x = 16.;
                        if services
                            .image_bytes("logo.svg", logo_bytes)
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
                                Avatar::pubkey(&acc.pubkey, services.ctx.ndb, &services.tx).render(ui, services.ctx.img_cache);
                            } else if Button::new().show(ui, |ui| ui.label("Login")).clicked() {
                                services.navigate(RouteType::LoginPage);
                            }
                        });
                    },
                )
            })
            .response
    }

    fn update(&mut self, _services: &mut RouteServices<'_, '_>) -> anyhow::Result<()> {
        Ok(())
    }
}
