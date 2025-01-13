use crate::route::RouteServices;
use crate::theme::FONT_SIZE;
use crate::widgets::{Avatar, Username};
use egui::{Response, Ui};
use nostrdb::NdbProfile;

pub struct Profile<'a> {
    size: f32,
    pubkey: &'a [u8; 32],
    profile: &'a Option<NdbProfile<'a>>,
}

impl<'a> Profile<'a> {
    pub fn new(pubkey: &'a [u8; 32]) -> Self {
        Self {
            pubkey,
            size: 40.,
            profile: &None,
        }
    }

    pub fn from_profile(pubkey: &'a [u8; 32], profile: &'a Option<NdbProfile<'a>>) -> Self {
        Self {
            pubkey,
            profile,
            size: 40.,
        }
    }

    pub fn size(self, size: f32) -> Self {
        Self { size, ..self }
    }

    pub fn render(self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.;

            let profile = if let Some(profile) = self.profile {
                Some(*profile)
            } else {
                services.profile(self.pubkey)
            };
            Avatar::from_profile(&profile)
                .size(self.size)
                .render(ui, services.ctx.img_cache);
            ui.add(Username::new(&profile, FONT_SIZE))
        })
        .response
    }
}
