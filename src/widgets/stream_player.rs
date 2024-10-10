use crate::route::RouteServices;
use crate::widgets::VideoPlaceholder;
use egui::{Response, Ui, Vec2, Widget};

pub struct StreamPlayer<'a> {
    services: &'a mut RouteServices<'a>,
}

impl<'a> StreamPlayer<'a> {
    pub fn new(services: &'a mut RouteServices<'a>) -> Self {
        Self { services }
    }
}

impl<'a> Widget for StreamPlayer<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let w = ui.available_width();
        let h = w / 16. * 9.;
        let size = Vec2::new(w, h);

        if let Some(p) = self.services.player.as_mut() {
            p.ui(ui, size)
        } else {
            VideoPlaceholder.ui(ui)
        }
    }
}