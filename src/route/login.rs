use crate::login::LoginKind;
use crate::route::{RouteServices, Routes};
use crate::widgets::{Button, NativeTextInput, NostrWidget};
use egui::{Color32, Frame, Margin, Response, RichText, Ui};
use nostr_sdk::util::hex;

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

impl NostrWidget for LoginPage {
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_>) -> Response {
        Frame::none()
            .inner_margin(Margin::same(12.))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 8.;

                    ui.label(RichText::new("Login").size(32.));
                    let mut input = NativeTextInput::new(&mut self.key).with_hint_text("npub/nsec");
                    input.render(ui, services);

                    if Button::new().show(ui, |ui| ui.label("Login")).clicked() {
                        if let Ok((hrp, key)) = bech32::decode(&self.key) {
                            match hrp.to_lowercase().as_str() {
                                "nsec" => {
                                    services.login.login(LoginKind::PrivateKey {
                                        key: key.as_slice().try_into().unwrap(),
                                    });
                                    services.navigate(Routes::HomePage);
                                }
                                "npub" | "nprofile" => {
                                    services.login.login(LoginKind::PublicKey {
                                        key: key.as_slice().try_into().unwrap(),
                                    });
                                    services.navigate(Routes::HomePage);
                                }
                                _ => {}
                            }
                        } else if let Ok(pk) = hex::decode(&self.key) {
                            if let Ok(pk) = pk.as_slice().try_into() {
                                services.login.login(LoginKind::PublicKey { key: pk });
                                services.navigate(Routes::HomePage);
                                return;
                            }
                        }
                        self.error = Some("Invalid pubkey".to_string());
                    }
                    if let Some(e) = &self.error {
                        ui.label(RichText::new(e).color(Color32::RED));
                    }
                })
                .response
            })
            .inner
    }
}
