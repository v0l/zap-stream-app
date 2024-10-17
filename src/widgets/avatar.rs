use crate::services::image_cache::ImageCache;
use crate::services::ndb_wrapper::SubWrapper;
use egui::{Color32, Image, Pos2, Response, Rounding, Sense, Ui, Vec2, Widget};
use nostrdb::NdbProfile;

pub struct Avatar<'a> {
    image: Option<Image<'a>>,
    sub: Option<SubWrapper>,
    size: Option<f32>,
}

impl<'a> Avatar<'a> {
    pub fn new(img: Image<'a>) -> Self {
        Self {
            image: Some(img),
            sub: None,
            size: None,
        }
    }

    pub fn new_optional(img: Option<Image<'a>>) -> Self {
        Self {
            image: img,
            sub: None,
            size: None,
        }
    }

    pub fn from_profile(p: Option<NdbProfile<'a>>, svc: &'a ImageCache) -> Self {
        let img = p
            .map_or(None, |f| f.picture().map(|f| svc.load(f)));
        Self {
            image: img,
            sub: None,
            size: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }
}

impl<'a> Widget for Avatar<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size_v = self.size.unwrap_or(40.);
        let size = Vec2::new(size_v, size_v);
        match self.image {
            Some(img) => img.fit_to_exact_size(size).rounding(Rounding::same(size_v)).ui(ui),
            None => {
                let (response, painter) = ui.allocate_painter(size, Sense::click());
                painter.circle_filled(Pos2::new(size_v / 2., size_v / 2.), size_v / 2., Color32::from_rgb(200, 200, 200));
                response
            }
        }
    }
}
