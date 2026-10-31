use crate::route::{RouteServices, RouteType};
use crate::theme;
use crate::widgets::{Button, NostrWidget};
use egui::{Color32, Frame, Id, Margin, Response, RichText, Sense, Stroke, TextEdit, Ui};
use nostr::prelude::hex;
use nostr::SecretKey;

pub struct LoginPage {
    key: String,
    error: Option<String>,
}

impl LoginPage {
    pub fn new() -> Self {
        Self {
            key: String::new(),
            error: None,
        }
    }
}

impl LoginPage {
    fn add_and_select(&self, services: &mut RouteServices<'_, '_>, kp: enostr::Keypair) {
        let pubkey = kp.pubkey;
        if let Some(resp) = services.ctx.accounts.add_account(kp) {
            resp.unk_id_action
                .process_action(services.ctx.unknown_ids, services.ctx.ndb, services.tx);
        }
        services.ctx.select_account(&pubkey);
        services.navigate(RouteType::HomePage);
    }
}

impl LoginPage {
    fn try_login(&mut self, services: &mut RouteServices<'_, '_>) {
        let input = self.key.trim().to_string();
        if let Ok((hrp, key)) = bech32::decode(&input) {
            match hrp.to_lowercase().as_str() {
                "nsec" => {
                    if let Ok(sk) = SecretKey::from_slice(key.as_slice()) {
                        self.add_and_select(services, enostr::Keypair::from_secret(sk));
                        return;
                    }
                }
                "npub" | "nprofile" => {
                    if let Ok(pk) = key.as_slice().try_into() {
                        self.add_and_select(
                            services,
                            enostr::Keypair::only_pubkey(enostr::Pubkey::new(pk)),
                        );
                        return;
                    }
                }
                _ => {}
            }
        } else if let Ok(pk) = hex::decode(&input) {
            if let Ok(pk) = pk.as_slice().try_into() {
                self.add_and_select(
                    services,
                    enostr::Keypair::only_pubkey(enostr::Pubkey::new(pk)),
                );
                return;
            }
        }
        self.error = Some("Invalid key".to_string());
    }

    fn option_row(ui: &mut Ui, title: &str, subtitle: &str) -> bool {
        let r = Frame::new()
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(12)
            .inner_margin(Margin::symmetric(16, 12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.label(theme::semibold(title, 16.0).color(theme::TEXT));
                    ui.label(theme::medium(subtitle, 14.0).color(theme::MUTED));
                });
            })
            .response
            .interact(Sense::click())
            .on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
        r.clicked()
    }
}

impl NostrWidget for LoginPage {
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        let screen = ui.ctx().content_rect();
        let backdrop = Color32::from_rgba_unmultiplied(0, 0, 0, 178);
        let mut close = false;
        egui::Area::new(Id::new("login-backdrop"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .show(ui.ctx(), |ui| {
                let (rect, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
                ui.painter().rect_filled(rect, 0.0, backdrop);
                if resp.clicked() {
                    close = true;
                }
            });

        let width = (screen.width() - 32.0).min(440.0);
        egui::Area::new(Id::new("login-card"))
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ui.ctx(), |ui| {
                Frame::new()
                    .fill(theme::SURFACE_LOW)
                    .stroke(Stroke::new(1.0, theme::SURFACE))
                    .corner_radius(16)
                    .inner_margin(Margin::same(24))
                    .show(ui, |ui| {
                        ui.set_width(width - 48.0);
                        ui.spacing_mut().item_spacing.y = 14.0;
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.add(egui::Label::new(theme::display_bold_text(
                                "Welcome to zap.stream",
                                26.0,
                            )));
                            ui.add(
                                egui::Label::new(
                                    RichText::new(
                                        "Watch, stream and get paid in sats. No email or password needed.",
                                    )
                                    .size(15.0)
                                    .color(theme::TEXT_SOFT),
                                )
                                .wrap_mode(egui::TextWrapMode::Wrap),
                            );
                        });
                        ui.add_space(6.0);
                        ui.label(theme::medium("npub, nsec or hex key", 14.0).color(theme::MUTED));
                        let secret = self.key.starts_with("nsec");
                        Frame::new()
                            .fill(theme::SURFACE)
                            .corner_radius(12)
                            .inner_margin(Margin::symmetric(12, 10))
                            .show(ui, |ui| {
                                ui.add(
                                    TextEdit::singleline(&mut self.key)
                                        .frame(Frame::NONE)
                                        .password(secret)
                                        .hint_text("npub1... or nsec1...")
                                        .desired_width(f32::INFINITY),
                                );
                            });
                        let go = Button::primary()
                            .padding(Margin::symmetric(12, 12))
                            .radius(255)
                            .full_width()
                            .text(ui, "Login")
                            .clicked();
                        if go || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            self.try_login(services);
                        }
                        if let Some(e) = &self.error {
                            ui.label(RichText::new(e).color(Color32::from_rgb(255, 99, 99)));
                        }
                        let _ = Self::option_row;
                    });
            });

        if close {
            services.navigate(RouteType::HomePage);
        }
        ui.response()
    }

    fn update(&mut self, _services: &mut RouteServices<'_, '_>) -> anyhow::Result<()> {
        Ok(())
    }
}
