use crate::theme::{self, ON_PRIMARY, PRIMARY, SURFACE, TEXT};
use egui::{Color32, CornerRadius, CursorIcon, Frame, Margin, Response, RichText, Sense, Ui};

#[derive(Clone, Copy)]
enum Kind {
    Primary,
    Secondary,
}

pub struct Button {
    kind: Kind,
    color: Option<Color32>,
    disabled: bool,
    padding: Margin,
    radius: u8,
}

impl Button {
    pub fn new() -> Self {
        Self {
            kind: Kind::Secondary,
            color: None,
            disabled: false,
            padding: Margin::symmetric(12, 8),
            radius: 12,
        }
    }

    pub fn primary() -> Self {
        Self {
            kind: Kind::Primary,
            ..Self::new()
        }
    }

    pub fn pill() -> Self {
        Self {
            radius: 255,
            ..Self::new()
        }
    }

    pub fn padding(mut self, padding: Margin) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_color(mut self, color: impl Into<Color32>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }

    pub fn simple(ui: &mut Ui, content: &str) -> Response {
        Button::new().text(ui, content)
    }

    pub fn text(self, ui: &mut Ui, text: &str) -> Response {
        let (_, text_color) = self.colors();
        self.show(ui, |ui| {
            ui.label(theme::medium(text, theme::FONT_SIZE).color(text_color))
        })
    }

    fn colors(&self) -> (Color32, Color32) {
        match (self.kind, self.color) {
            (_, Some(c)) if c == PRIMARY => (PRIMARY, ON_PRIMARY),
            (_, Some(c)) => (c, TEXT),
            (Kind::Primary, None) => (PRIMARY, ON_PRIMARY),
            (Kind::Secondary, None) => (SURFACE, TEXT),
        }
    }

    pub fn show<F>(self, ui: &mut Ui, add_contents: F) -> Response
    where
        F: FnOnce(&mut Ui) -> Response,
    {
        let (fill, _) = self.colors();
        let r = Frame::new()
            .inner_margin(self.padding)
            .fill(fill)
            .corner_radius(CornerRadius::same(self.radius))
            .multiply_with_opacity(if self.disabled { 0.5 } else { 1.0 })
            .show(ui, add_contents);

        let id = r.response.id;
        let rect = r
            .response
            .on_hover_and_drag_cursor(if self.disabled {
                CursorIcon::NotAllowed
            } else {
                CursorIcon::PointingHand
            })
            .rect;
        ui.interact(rect, id, Sense::click())
    }
}

pub fn label_on(text: impl Into<String>, color: Color32) -> RichText {
    theme::medium(text, theme::FONT_SIZE).color(color)
}
