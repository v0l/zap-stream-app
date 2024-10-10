use egui::{Color32, Image, Rect, Response, Rounding, Sense, Ui, Vec2, Widget};
use crate::services::profile::ProfileService;

pub struct Avatar<'a> {
    image: Option<Image<'a>>,
}

impl<'a> Avatar<'a> {
    pub fn new(img: Image<'a>) -> Self {
        Self { image: Some(img) }
    }

    pub fn public_key(svc: &'a ProfileService, pk: &[u8; 32]) -> Self {
        if let Some(meta) = svc.get_profile(pk) {
            if let Some(img) = &meta.picture {
                return Self { image: Some(Image::from_uri(img.clone())) };
            }
        }
        Self { image: None }
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
            Some(img) => {
                img.rounding(Rounding::same(ui.available_height())).ui(ui)
            }
            None => {
                let (response, painter) = ui.allocate_painter(Vec2::new(32., 32.), Sense::hover());
                painter.rect_filled(Rect::EVERYTHING, Rounding::same(32.), Color32::from_rgb(200, 200, 200));
                response
            }
        }
    }
}