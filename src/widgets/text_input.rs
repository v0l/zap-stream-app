use crate::route::{RouteAction, RouteServices};
use crate::theme::{MARGIN_DEFAULT, NEUTRAL_500, NEUTRAL_800, ROUNDING_DEFAULT};
use crate::widgets::NostrWidget;
use egui::{Frame, Response, TextEdit, Ui};

/// Wrap the [TextEdit] widget to handle native keyboard
pub struct NativeTextInput<'a> {
    pub text: &'a mut String,
    hint_text: Option<&'a str>,
}

impl<'a> NativeTextInput<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            hint_text: None,
        }
    }

    pub fn with_hint_text(mut self, hint_text: &'a str) -> Self {
        self.hint_text = Some(hint_text);
        self
    }
}

impl<'a> NostrWidget for NativeTextInput<'a> {
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_>) -> Response {
        let mut editor = TextEdit::singleline(self.text).frame(false);
        if let Some(hint_text) = self.hint_text {
            editor = editor.hint_text(egui::RichText::new(hint_text).color(NEUTRAL_500));
        }
        let response = Frame::none()
            .inner_margin(MARGIN_DEFAULT)
            .fill(NEUTRAL_800)
            .rounding(ROUNDING_DEFAULT)
            .show(ui, |ui| ui.add(editor))
            .inner;
        if response.lost_focus() {
            services.action(RouteAction::HideKeyboard);
        }
        if response.gained_focus() {
            services.action(RouteAction::ShowKeyboard);
        }
        response
    }
}
