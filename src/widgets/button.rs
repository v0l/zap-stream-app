use crate::theme::{MARGIN_DEFAULT, NEUTRAL_800, ROUNDING_DEFAULT};
use egui::{Color32, CursorIcon, Frame, Response, Sense, Ui, WidgetText};

pub struct Button {
    color: Color32,
    disabled: bool,
}

impl Button {
    pub fn new() -> Self {
        Self {
            color: NEUTRAL_800,
            disabled: false,
        }
    }

    pub fn with_color(mut self, color: impl Into<Color32>) -> Self {
        self.color = color.into();
        self
    }

    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }

    pub fn simple(ui: &mut Ui, content: &str) -> Response {
        Button::new().show(ui, |ui| ui.label(content))
    }

    pub fn text(self, ui: &mut Ui, text: impl Into<WidgetText>) -> Response {
        self.show(ui, |ui| ui.label(text))
    }

    pub fn show<F>(self, ui: &mut Ui, add_contents: F) -> Response
    where
        F: FnOnce(&mut Ui) -> Response,
    {
        let r = Frame::none()
            .inner_margin(MARGIN_DEFAULT)
            .fill(self.color)
            .rounding(ROUNDING_DEFAULT)
            .multiply_with_opacity(if self.disabled { 0.5 } else { 1.0 })
            .show(ui, add_contents);

        let id = r.response.id;
        ui.interact(
            r.response
                .on_hover_and_drag_cursor(if self.disabled {
                    CursorIcon::NotAllowed
                } else {
                    CursorIcon::PointingHand
                })
                .rect,
            id,
            Sense::click(),
        )
    }
}
