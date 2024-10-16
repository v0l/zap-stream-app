use crate::route::RouteServices;
use crate::services::ndb_wrapper::SubWrapper;
use crate::widgets::Avatar;
use egui::{Color32, Image, Label, Response, RichText, TextWrapMode, Ui, Widget};
use nostrdb::NdbProfile;

pub struct Profile<'a> {
    size: f32,
    pubkey: &'a [u8; 32],
    profile: Option<NdbProfile<'a>>,
    sub: Option<SubWrapper>,
}

impl<'a> Profile<'a> {
    pub fn new(pubkey: &'a [u8; 32], services: &'a RouteServices<'a>) -> Self {
        let (p, sub) = services.ndb.fetch_profile(services.tx, pubkey);
        Self {
            pubkey,
            size: 40.,
            profile: p,
            sub,
        }
    }

    pub fn size(self, size: f32) -> Self {
        Self { size, ..self }
    }
}

impl<'a> Widget for Profile<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.;

            let img = self
                .profile
                .map_or(None, |f| f.picture().map(|f| Image::from_uri(f)));
            ui.add(Avatar::new_optional(img).size(self.size));

            let name = self
                .profile
                .map_or("Nostrich", |f| f.name().map_or("Nostrich", |f| f));
            let name = RichText::new(name).size(13.).color(Color32::WHITE);
            ui.add(Label::new(name).wrap_mode(TextWrapMode::Truncate));
        })
        .response
    }
}
