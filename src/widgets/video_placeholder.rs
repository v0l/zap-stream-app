use egui::{Color32, Response, Rounding, Sense, Ui, Vec2, Widget};

pub struct VideoPlaceholder;

impl Widget for VideoPlaceholder {
    fn ui(self, ui: &mut Ui) -> Response {
        let w = ui.available_width();
        let h = (w / 16.0) * 9.0;
        let img_size = Vec2::new(w, h);

        let (response, painter) = ui.allocate_painter(img_size, Sense::click());
        painter.rect_filled(
            response.rect,
            Rounding::same(12.),
            Color32::from_rgb(200, 200, 200),
        );
        response
    }
}
