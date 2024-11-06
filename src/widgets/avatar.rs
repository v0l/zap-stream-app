use crate::route::RouteServices;
use crate::services::ndb_wrapper::SubWrapper;
use egui::{vec2, Color32, Pos2, Response, Rounding, Sense, Ui, Vec2, Widget};
use nostrdb::NdbProfile;

pub struct Avatar<'a> {
    image: Option<&'a str>,
    sub: Option<SubWrapper>,
    size: Option<f32>,
    services: &'a RouteServices<'a>,
}

impl<'a> Avatar<'a> {
    pub fn new_optional(img: Option<&'a str>, services: &'a RouteServices<'a>) -> Self {
        Self {
            image: img,
            sub: None,
            size: None,
            services,
        }
    }

    pub fn from_profile(p: &'a Option<NdbProfile<'a>>, services: &'a RouteServices<'a>) -> Self {
        let img = p.map(|f| f.picture()).unwrap_or(None);
        Self {
            image: img,
            sub: None,
            size: None,
            services,
        }
    }

    pub fn pubkey(pk: &[u8; 32], services: &'a RouteServices<'a>) -> Self {
        let (p, sub) = services.ndb.fetch_profile(services.tx, pk);
        Self {
            image: p.map(|f| f.picture()).unwrap_or(None),
            sub,
            size: None,
            services,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    fn placeholder(ui: &mut Ui, size: f32) -> Response {
        let (response, painter) = ui.allocate_painter(vec2(size, size), Sense::click());
        painter.circle_filled(
            Pos2::new(size / 2., size / 2.),
            size / 2.,
            Color32::from_rgb(200, 200, 200),
        );
        response
    }
}

impl<'a> Widget for Avatar<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size_v = self.size.unwrap_or(40.);
        let size = Vec2::new(size_v, size_v);
        if !ui.is_visible() {
            return Self::placeholder(ui, size_v);
        }
        match self
            .image
            .as_ref()
            .map(|i| self.services.img_cache.load(*i))
        {
            Some(img) => img
                .fit_to_exact_size(size)
                .rounding(Rounding::same(size_v))
                .ui(ui),
            None => Self::placeholder(ui, size_v),
        }
    }
}
