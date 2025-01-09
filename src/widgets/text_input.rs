use crate::theme::{MARGIN_DEFAULT, NEUTRAL_500, NEUTRAL_900, ROUNDING_DEFAULT};
use egui::{Frame, Response, TextEdit, Ui, Widget};

/// Wrap the [TextEdit] widget to handle native keyboard
pub struct NativeTextInput<'a> {
    pub text: &'a mut String,
    hint_text: Option<&'a str>,
    frame: bool,
}

impl<'a> NativeTextInput<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            hint_text: None,
            frame: false,
        }
    }

    pub fn with_hint_text(mut self, hint_text: &'a str) -> Self {
        self.hint_text = Some(hint_text);
        self
    }

    pub fn with_frame(mut self, frame: bool) -> Self {
        self.frame = frame;
        self
    }
}

impl Widget for NativeTextInput<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut editor = TextEdit::multiline(self.text)
            .frame(false)
            .desired_rows(1)
            .desired_width(f32::INFINITY);
        if let Some(hint_text) = self.hint_text {
            editor = editor.hint_text(egui::RichText::new(hint_text).color(NEUTRAL_500));
        }
        
        if self.frame {
            Frame::none()
                .inner_margin(MARGIN_DEFAULT)
                .fill(NEUTRAL_900)
                .rounding(ROUNDING_DEFAULT)
                .show(ui, |ui| ui.add(editor))
                .inner
        } else {
            ui.add(editor)
        }
    }
}
