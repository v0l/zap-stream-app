use crate::route::RouteServices;
use crate::theme::NEUTRAL_900;
use crate::widgets::NostrWidget;
use egui::{Button, Frame, Image, Margin, Rect, Response, Rounding, Sense, Shadow, Stroke, TextEdit, Ui, Vec2, Widget};
use log::info;

pub struct WriteChat {
    msg: String,
}

impl WriteChat {
    pub fn new() -> Self {
        Self {
            msg: String::new(),
        }
    }
}

impl NostrWidget for WriteChat {
    fn render(&mut self, ui: &mut Ui, services: &RouteServices<'_>) -> Response {
        let size = ui.available_size();
        let logo_bytes = include_bytes!("../resources/send-03.svg");
        Frame::none()
            .inner_margin(Margin::symmetric(12., 6.))
            .stroke(Stroke::new(1.0, NEUTRAL_900))
            .show(ui, |ui| {
                Frame::none()
                    .fill(NEUTRAL_900)
                    .rounding(Rounding::same(12.0))
                    .inner_margin(Margin::symmetric(12., 12.))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let editor = TextEdit::singleline(&mut self.msg)
                                .frame(false);
                            ui.add(editor);
                            if Image::from_bytes("send-03.svg", logo_bytes)
                                .sense(Sense::click())
                                .ui(ui)
                                .clicked() {
                                info!("Sending: {}", self.msg);
                                self.msg.clear();
                            }
                        });
                    })
            }).response
    }
}