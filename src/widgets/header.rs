use crate::link::NostrLink;
use crate::route::{RouteServices, RouteType};
use crate::theme::{self, BORDER, HEADER_HEIGHT, MUTED, PRIMARY, SURFACE_LOW, TEXT};
use crate::widgets::avatar::Avatar;
use crate::widgets::Button;
use egui::emath::Align;
use egui::{
    vec2, Color32, CursorIcon, Frame, Image, Layout, Margin, Response, Sense, Stroke, TextEdit,
    Ui, Widget,
};
use nostrdb::Transaction;

fn is_fallback(pk: &enostr::Pubkey) -> bool {
    pk.bytes() == notedeck::FALLBACK_PUBKEY().bytes()
}

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
        let wide = ui.available_width() >= theme::WIDE_BREAKPOINT;
        let r = Frame::new()
            .inner_margin(Margin::symmetric(16, 0))
            .show(ui, |ui| {
                let size = vec2(ui.available_width(), HEADER_HEIGHT);
                ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 10.;
                    let logo = Image::from_bytes(
                        "header_logo.svg",
                        include_bytes!("../resources/logo_amber.svg"),
                    )
                    .max_height(22.)
                    .sense(Sense::click())
                    .ui(ui)
                    .on_hover_and_drag_cursor(CursorIcon::PointingHand);
                    let word = ui
                        .add(
                            egui::Label::new(theme::display_bold_text("zap.stream", 19.).color(PRIMARY))
                                .sense(Sense::click()),
                        )
                        .on_hover_and_drag_cursor(CursorIcon::PointingHand);
                    if logo.clicked() || word.clicked() {
                        services.navigate(RouteType::HomePage);
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.set_height(HEADER_HEIGHT);
                        let acc = services.ctx.accounts.get_selected_account();
                        let logged_in = acc.key.secret_key.is_some()
                            || (!acc.key.pubkey.bytes().iter().all(|b| *b == 0)
                                && !is_fallback(&acc.key.pubkey));
                        if logged_in {
                            let pubkey = acc.key.pubkey;
                            let avatar = Avatar::pubkey(pubkey.bytes(), services.ctx.ndb, tx)
                                .size(32.)
                                .render(ui, services.ctx);
                            let profile = services.profile(pubkey.bytes());
                            let name = profile
                                .and_then(|p| p.display_name().or(p.name()))
                                .unwrap_or("Nostrich")
                                .to_string();
                            let mut action = None;
                            egui::Popup::menu(&avatar)
                                .id(egui::Id::new("account-menu"))
                                .show(|ui| {
                                    ui.set_min_width(220.0);
                                    ui.spacing_mut().item_spacing.y = 4.0;
                                    ui.label(theme::semibold(&name, 15.0));
                                    ui.label(
                                        theme::medium(
                                            format!("{}...", &pubkey.hex()[..12]),
                                            12.5,
                                        )
                                        .color(MUTED),
                                    );
                                    ui.separator();
                                    if Button::new().text(ui, "Profile").clicked() {
                                        action = Some("profile");
                                        ui.close();
                                    }
                                    if Button::new().text(ui, "Logout").clicked() {
                                        action = Some("logout");
                                        ui.close();
                                    }
                                });
                            match action {
                                Some("profile") => {
                                    services.navigate(RouteType::ProfilePage {
                                        link: NostrLink::profile(pubkey.bytes()),
                                    });
                                }
                                Some("logout") => {
                                    services.action(crate::route::RouteAction::Logout(pubkey));
                                }
                                _ => {}
                            }
                        } else {
                            let clicked = ui
                                .scope(|ui| {
                                    ui.set_height(34.0);
                                    Button::primary().text(ui, "Login").clicked()
                                })
                                .inner;
                            if clicked {
                                services.navigate(RouteType::LoginPage);
                            }
                        }

                        if wide {
                            let w = (ui.available_width() - 32.0).clamp(0.0, 410.0);
                            let qid = egui::Id::new("search-query");
                            let mut query: String =
                                ui.ctx().data(|d| d.get_temp(qid)).unwrap_or_default();
                            let before = query.clone();
                            Frame::new()
                                .fill(SURFACE_LOW)
                                .stroke(Stroke::new(1.0, BORDER))
                                .corner_radius(12)
                                .inner_margin(Margin::symmetric(12, 4))
                                .show(ui, |ui| {
                                    ui.set_width(w - 26.0);
                                    ui.set_height(18.0);
                                    ui.add(
                                        TextEdit::singleline(&mut query)
                                            .frame(egui::Frame::NONE)
                                            .hint_text(
                                                theme::semibold("Search", theme::FONT_SIZE)
                                                    .color(MUTED),
                                            )
                                            .desired_width(f32::INFINITY),
                                    );
                                });
                            if query != before {
                                ui.ctx().data_mut(|d| d.insert_temp(qid, query));
                            }
                        }
                    });
                });
            });
        let rect = r.response.rect;
        ui.painter().hline(
            rect.x_range(),
            rect.bottom(),
            Stroke::new(1.0, BORDER),
        );
        let _ = (vec2(0., 0.), Color32::TRANSPARENT, TEXT);
        r.response
    }
}
