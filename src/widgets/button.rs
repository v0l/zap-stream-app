use crate::theme::NEUTRAL_800;
use egui::{Color32, CursorIcon, Frame, Margin, Response, Sense, Ui};

pub struct Button {
    color: Color32,
}

impl Button {
    pub fn new() -> Self {
        Self {
            color: NEUTRAL_800
        }
    }

    pub fn show<F>(self, ui: &mut Ui, add_contents: F) -> Response
    where
        F: FnOnce(&mut Ui) -> Response,
    {
        let r = Frame::none()
            .inner_margin(Margin::symmetric(12., 8.))
            .fill(self.color)
            .rounding(12.)
            .show(ui, add_contents);

        let id = r.response.id;
        ui.interact(
            r.response.on_hover_and_drag_cursor(CursorIcon::PointingHand).rect,
            id,
            Sense::click(),
        )
    }
}