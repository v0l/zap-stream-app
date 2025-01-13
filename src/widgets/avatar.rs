use crate::route::image_from_cache;
use egui::{vec2, Color32, Pos2, Response, Rounding, Sense, Ui, Vec2, Widget};
use nostrdb::{Ndb, NdbProfile, Transaction};
use notedeck::ImageCache;

pub struct Avatar {
    image: Option<String>,
    size: Option<f32>,
}

impl Avatar {
    pub fn new_optional(img: Option<&str>) -> Self {
        Self {
            image: img.map(String::from),
            size: None,
        }
    }

    pub fn pubkey(pk: &[u8; 32], ndb: &Ndb, tx: &Transaction) -> Self {
        let picture = ndb
            .get_profile_by_pubkey(tx, pk)
            .map(|p| p.record().profile().map(|p| p.picture()).unwrap_or(None))
            .unwrap_or(None);
        Self {
            image: picture.map(|s| s.to_string()),
            size: None,
        }
    }

    pub fn from_profile(p: &Option<NdbProfile<'_>>) -> Self {
        let img = p.map(|f| f.picture()).unwrap_or(None);
        Self {
            image: img.map(String::from),
            size: None,
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

    pub fn render(self, ui: &mut Ui, img_cache: &mut ImageCache) -> Response {
        let size_v = self.size.unwrap_or(40.);
        let size = Vec2::new(size_v, size_v);
        if !ui.is_rect_visible(ui.cursor()) {
            return Self::placeholder(ui, size_v);
        }
        match &self.image {
            Some(img) => image_from_cache(img_cache, ui, img)
                .max_size(size)
                .fit_to_exact_size(size)
                .rounding(Rounding::same(size_v))
                .sense(Sense::click())
                .ui(ui),
            None => Self::placeholder(ui, size_v),
        }
    }
}
