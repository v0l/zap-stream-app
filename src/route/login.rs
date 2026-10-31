use crate::route::{RouteServices, RouteType};
use crate::theme;
use crate::widgets::{Button, NostrWidget};
use egui::{Color32, Frame, Id, Margin, Response, RichText, Sense, Stroke, TextEdit, Ui};
use nostr::prelude::hex;
use nostr::SecretKey;

#[derive(Clone, Copy, PartialEq)]
enum Step {
    Welcome,
    PasteKey,
    Profile,
    Address,
    SaveKey,
}

pub struct LoginPage {
    key: String,
    error: Option<String>,
    step: Step,
    name: String,
    address: String,
    keys: Option<nostr::Keys>,
    saved: bool,
    copied: bool,
}

impl LoginPage {
    pub fn new() -> Self {
        let mut page = Self {
            key: String::new(),
            error: None,
            step: Step::Welcome,
            name: String::new(),
            address: String::new(),
            keys: None,
            saved: false,
            copied: false,
        };
        if let Ok(step) = std::env::var("ZAP_STREAM_LOGIN_STEP") {
            page.keys = Some(nostr::Keys::generate());
            page.name = "Test Streamer".to_string();
            if let Some(k) = &page.keys {
                page.address = format!("{}@zap.stream", k.public_key().to_hex());
            }
            page.step = match step.as_str() {
                "paste" => Step::PasteKey,
                "profile" => Step::Profile,
                "address" => Step::Address,
                "save" => Step::SaveKey,
                _ => Step::Welcome,
            };
        }
        page
    }
}

impl LoginPage {
    fn add_and_select(&self, services: &mut RouteServices<'_, '_>, kp: enostr::Keypair) {
        let pubkey = kp.pubkey;
        if let Some(resp) = services.ctx.accounts.add_account(kp) {
            resp.unk_id_action
                .process_action(services.ctx.unknown_ids, services.ctx.ndb, services.tx);
        }
        services.action(crate::route::RouteAction::SelectAccount(pubkey));
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

    fn round_icon(ui: &mut Ui, glyph: &str) -> bool {
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(30.0, 30.0), Sense::click());
        let painter = ui.painter();
        painter.circle_filled(rect.center(), 15.0, theme::SURFACE);
        let stroke = Stroke::new(1.8, theme::TEXT);
        let c = rect.center();
        if glyph == "x" {
            let d = 4.5;
            painter.line_segment([c + egui::vec2(-d, -d), c + egui::vec2(d, d)], stroke);
            painter.line_segment([c + egui::vec2(-d, d), c + egui::vec2(d, -d)], stroke);
        } else {
            painter.line_segment([c + egui::vec2(2.5, -5.5), c + egui::vec2(-2.5, 0.0)], stroke);
            painter.line_segment([c + egui::vec2(-2.5, 0.0), c + egui::vec2(2.5, 5.5)], stroke);
        }
        resp.on_hover_and_drag_cursor(egui::CursorIcon::PointingHand)
            .clicked()
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

impl LoginPage {
    fn title(ui: &mut Ui, text: &str) {
        ui.add(egui::Label::new(theme::display_bold_text(text, 26.0)));
    }

    fn body(ui: &mut Ui, text: &str) {
        ui.add(
            egui::Label::new(RichText::new(text).size(15.0).color(theme::TEXT_SOFT))
                .wrap_mode(egui::TextWrapMode::Wrap),
        );
    }

    fn field(ui: &mut Ui, value: &mut String, hint: &str, mono: bool) {
        Frame::new()
            .fill(theme::SURFACE)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(12)
            .inner_margin(Margin::symmetric(12, 10))
            .show(ui, |ui| {
                let mut edit = TextEdit::singleline(value)
                    .frame(Frame::NONE)
                    .hint_text(hint)
                    .desired_width(f32::INFINITY);
                if mono {
                    edit = edit.font(egui::TextStyle::Monospace);
                }
                ui.add(edit);
            });
    }

    fn label(ui: &mut Ui, text: &str) {
        ui.label(theme::medium(text, 14.0).color(theme::MUTED));
    }

    fn primary(ui: &mut Ui, text: &str, enabled: bool) -> bool {
        Button::primary()
            .padding(Margin::symmetric(12, 12))
            .radius(255)
            .full_width()
            .disabled(!enabled)
            .text(ui, text)
            .clicked()
            && enabled
    }

    fn shown_key(&self) -> String {
        use nostr::ToBech32;
        self.keys
            .as_ref()
            .and_then(|k| k.secret_key().to_bech32().ok())
            .unwrap_or_default()
    }

    fn masked(nsec: &str) -> String {
        if nsec.len() > 24 {
            format!("{}...{}", &nsec[..10], &nsec[nsec.len() - 14..])
        } else {
            nsec.to_string()
        }
    }

    fn finish(&mut self, services: &mut RouteServices<'_, '_>) {
        let Some(keys) = self.keys.clone() else {
            return;
        };
        let kp = enostr::Keypair::from_secret(keys.secret_key().clone());
        self.add_and_select(services, kp);

        let mut meta = nostr::Metadata::new().name(self.name.trim()).display_name(self.name.trim());
        if !self.address.trim().is_empty() {
            meta = meta.lud16(self.address.trim());
        }
        if let Ok(ev) = nostr::EventBuilder::metadata(&meta).sign_with_keys(&keys) {
            services.broadcast_event(ev);
        }
    }

    fn step_welcome(&mut self, ui: &mut Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            Self::title(ui, "Welcome to zap.stream");
            Self::body(
                ui,
                "Watch, stream and get paid in sats. No email or password needed.",
            );
        });
        ui.add_space(6.0);
        if Self::primary(ui, "Create an account", true) {
            self.keys = Some(nostr::Keys::generate());
            self.step = Step::Profile;
        }
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.label(theme::medium("Already on nostr?", 14.0).color(theme::MUTED));
        });
        let _ = Self::option_row(
            ui,
            "Nostr extension",
            "No extension found in this app",
        );
        if Self::option_row(ui, "Private key", "Paste your nsec. Less secure.") {
            self.step = Step::PasteKey;
        }
    }

    fn step_paste(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) {
        Self::title(ui, "Log in with a key");
        Self::body(ui, "Paste your nsec, or an npub to browse as read only.");
        Self::label(ui, "npub, nsec or hex key");
        Self::field(ui, &mut self.key, "nsec1... or npub1...", true);
        let go = Self::primary(ui, "Login", !self.key.trim().is_empty());
        if go || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.try_login(services);
        }
        if let Some(e) = &self.error {
            ui.label(RichText::new(e).color(Color32::from_rgb(255, 99, 99)));
        }
    }

    fn step_profile(&mut self, ui: &mut Ui) {
        Self::title(ui, "Set up your profile");
        Self::body(
            ui,
            "This is how viewers will see you. You can change it later.",
        );
        ui.add_space(4.0);
        Self::label(ui, "Display name");
        Self::field(ui, &mut self.name, "Satoshi", false);
        if Self::primary(ui, "Continue", !self.name.trim().is_empty()) {
            if self.address.is_empty() {
                if let Some(k) = &self.keys {
                    self.address = format!("{}@zap.stream", k.public_key().to_hex());
                }
            }
            self.step = Step::Address;
        }
    }

    fn step_address(&mut self, ui: &mut Ui) {
        Self::title(ui, "Get paid by viewers");
        Self::body(
            ui,
            "Zaps from viewers go to this lightning address. We made you one, or paste your own.",
        );
        Self::label(ui, "Lightning address");
        Self::field(ui, &mut self.address, "you@wallet.com", true);
        Self::body(ui, "You can always replace it with your own address later.");
        if Self::primary(ui, "Continue", true) {
            self.step = Step::SaveKey;
        }
    }

    fn step_save(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) {
        Self::title(ui, "Save your key");
        Self::body(
            ui,
            "This key is your account. There is no password reset, so if you lose it, the account is gone for good.",
        );
        let nsec = self.shown_key();
        let shown = Self::masked(&nsec);
        let copy = Frame::new()
            .fill(theme::SURFACE)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(12)
            .inner_margin(Margin::symmetric(16, 12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(shown)
                            .monospace()
                            .size(14.0)
                            .color(theme::TEXT),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            theme::medium(if self.copied { "copied" } else { "copy" }, 13.0)
                                .color(theme::MUTED),
                        );
                    });
                });
            })
            .response
            .interact(Sense::click())
            .on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
        if copy.clicked() {
            services.ctx.clipboard.set_text(nsec);
            self.copied = true;
        }
        ui.checkbox(
            &mut self.saved,
            RichText::new("I saved my key somewhere safe, like a password manager")
                .size(14.0)
                .color(theme::TEXT_SOFT),
        );
        if Self::primary(ui, "Finish", self.saved) {
            self.finish(services);
        }
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

                        let top = ui.horizontal(|ui| {
                            let mut back = false;
                            let mut x = false;
                            if self.step != Step::Welcome && self.step != Step::SaveKey {
                                back = Self::round_icon(ui, "back");
                            }
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| x = Self::round_icon(ui, "x"),
                            );
                            (back, x)
                        });
                        let (back, x) = top.inner;
                        if x {
                            close = true;
                        }
                        if back {
                            self.step = match self.step {
                                Step::PasteKey | Step::Profile => Step::Welcome,
                                Step::Address => Step::Profile,
                                other => other,
                            };
                        }

                        match self.step {
                            Step::Welcome => self.step_welcome(ui),
                            Step::PasteKey => self.step_paste(ui, services),
                            Step::Profile => self.step_profile(ui),
                            Step::Address => self.step_address(ui),
                            Step::SaveKey => self.step_save(ui, services),
                        }
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
