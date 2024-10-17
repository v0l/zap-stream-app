use crate::route::RouteServices;
use crate::services::image_cache::ImageCache;
use crate::services::ndb_wrapper::SubWrapper;
use crate::widgets::Avatar;
use egui::{Color32, Label, Response, RichText, TextWrapMode, Ui, Widget};
use nostrdb::NdbProfile;

pub struct Profile<'a> {
    size: f32,
    pubkey: &'a [u8; 32],
    profile: Option<NdbProfile<'a>>,
    sub: Option<SubWrapper>,
    img_cache: &'a ImageCache,
}

impl<'a> Profile<'a> {
    pub fn new(pubkey: &'a [u8; 32], services: &'a RouteServices<'a>) -> Self {
        let (p, sub) = services.ndb.fetch_profile(services.tx, pubkey);

        Self {
            pubkey,
            size: 40.,
            profile: p,
            img_cache: services.img_cache,
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

            ui.add(Avatar::from_profile(self.profile, self.img_cache).size(self.size));

            let name = self
                .profile
                .map_or("Nostrich", |f| f.name().map_or("Nostrich", |f| f));
            let name = RichText::new(name).size(13.).color(Color32::WHITE);
            ui.add(Label::new(name).wrap_mode(TextWrapMode::Truncate));
        }).response
    }
}
