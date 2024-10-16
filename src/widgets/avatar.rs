use crate::route::RouteServices;
use crate::services::ndb_wrapper::SubWrapper;
use egui::{Color32, Image, Rect, Response, Rounding, Sense, Ui, Vec2, Widget};

pub struct Avatar<'a> {
    image: Option<Image<'a>>,
    sub: Option<SubWrapper>,
}

impl<'a> Avatar<'a> {
    pub fn new(img: Image<'a>) -> Self {
        Self {
            image: Some(img),
            sub: None,
        }
    }

    pub fn new_optional(img: Option<Image<'a>>) -> Self {
        Self {
            image: img,
            sub: None,
        }
    }

    pub fn pubkey(pk: &[u8; 32], svc: &RouteServices<'a>) -> Self {
        let (img, sub) = svc.ndb.fetch_profile(svc.tx, pk);
        Self {
            image: img
                .map_or(None, |p| p.picture())
                .map(|p| Image::from_uri(p)),
            sub,
        }
    }

    pub fn max_size(mut self, size: f32) -> Self {
        self.image = if let Some(i) = self.image {
            Some(i.max_height(size))
        } else {
            None
        };
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.image = if let Some(i) = self.image {
            Some(i.fit_to_exact_size(Vec2::new(size, size)))
        } else {
            None
        };
        self
    }
}

impl<'a> Widget for Avatar<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        match self.image {
            Some(img) => img.rounding(Rounding::same(ui.available_height())).ui(ui),
            None => {
                let h = ui.available_height();
                let rnd = Rounding::same(h);
                let (response, painter) = ui.allocate_painter(Vec2::new(h, h), Sense::click());
                painter.rect_filled(Rect::EVERYTHING, rnd, Color32::from_rgb(200, 200, 200));
                response
            }
        }
    }
}
