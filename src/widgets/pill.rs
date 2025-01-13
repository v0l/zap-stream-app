use crate::theme::{FONT_SIZE, NEUTRAL_800};
use eframe::epaint::Margin;
use egui::{Color32, Frame, Response, RichText, Ui, Widget};

pub struct Pill {
    text: String,
    color: Color32,
}

impl Pill {
    pub fn new(text: &str) -> Self {
        Self {
            text: String::from(text),
            color: NEUTRAL_800,
        }
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }
}

impl Widget for Pill {
    fn ui(self, ui: &mut Ui) -> Response {
        Frame::default()
            .inner_margin(Margin::symmetric(5.0, 3.0))
            .rounding(5.0)
            .fill(self.color)
            .show(ui, |ui| {
                ui.label(RichText::new(&self.text).size(FONT_SIZE));
            })
            .response
    }
}
