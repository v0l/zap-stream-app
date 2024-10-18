use crate::route::RouteServices;
use crate::services::image_cache::ImageCache;
use crate::services::ndb_wrapper::SubWrapper;
use crate::widgets::{Avatar, Username};
use egui::{Response, Ui, Widget};
use nostrdb::NdbProfile;
use crate::theme::FONT_SIZE;

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

            ui.add(Avatar::from_profile(&self.profile, self.img_cache).size(self.size));
            ui.add(Username::new(&self.profile, FONT_SIZE))
        }).response
    }
}
