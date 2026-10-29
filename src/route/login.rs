use crate::route::{RouteServices, RouteType};
use crate::widgets::{Button, NativeTextInput, NostrWidget};
use egui::{Color32, Frame, Margin, Response, RichText, Ui};
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

impl NostrWidget for LoginPage {
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        Frame::new()
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 8.;

                    ui.label(RichText::new("Login").size(32.));
                    let input = NativeTextInput::new(&mut self.key).with_hint_text("npub/nsec");
                    ui.add(input);

                    if Button::new().show(ui, |ui| ui.label("Login")).clicked() {
                        if let Ok((hrp, key)) = bech32::decode(&self.key) {
                            match hrp.to_lowercase().as_str() {
                                "nsec" => {
                                    if let Ok(sk) = SecretKey::from_slice(key.as_slice()) {
                                        let kp = enostr::Keypair::from_secret(sk);
                                        self.add_and_select(services, kp);
                                        return;
                                    }
                                }
                                "npub" | "nprofile" => {
                                    if let Ok(pk) = key.as_slice().try_into() {
                                        let kp = enostr::Keypair::only_pubkey(enostr::Pubkey::new(pk));
                                        self.add_and_select(services, kp);
                                        return;
                                    }
                                }
                                _ => {}
                            }
                        } else if let Ok(pk) = hex::decode(&self.key) {
                            if let Ok(pk) = pk.as_slice().try_into() {
                                let kp = enostr::Keypair::only_pubkey(enostr::Pubkey::new(pk));
                                self.add_and_select(services, kp);
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

    fn update(&mut self, _services: &mut RouteServices<'_, '_>) -> anyhow::Result<()> {
        Ok(())
    }
}
