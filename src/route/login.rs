use crate::route::{RouteAction, RouteServices, Routes};
use crate::widgets::{Button, NostrWidget};
use egui::{Color32, Response, RichText, Ui};
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
    fn render(&mut self, ui: &mut Ui, services: &RouteServices<'_>) -> Response {
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 8.;

            ui.label(RichText::new("Login").size(32.));
            ui.label("Pubkey");
            ui.text_edit_singleline(&mut self.key);
            if Button::new().show(ui, |ui| ui.label("Login")).clicked() {
                if let Ok(pk) = hex::decode(&self.key) {
                    if let Ok(pk) = pk.as_slice().try_into() {
                        services.action(RouteAction::LoginPubkey(pk));
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
    }
}
