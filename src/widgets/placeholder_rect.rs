use crate::theme::{NEUTRAL_800, ROUNDING_DEFAULT};
use egui::{Response, Sense, Ui, Widget};

pub struct PlaceholderRect;

impl Widget for PlaceholderRect {
    fn ui(self, ui: &mut Ui) -> Response {
        let img_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(img_size, Sense::click());
        painter.rect_filled(response.rect, ROUNDING_DEFAULT, NEUTRAL_800);
        response
    }
}
