use crate::link::NostrLink;
use crate::route::RouteServices;
use crate::theme::{NEUTRAL_900, ROUNDING_DEFAULT};
use crate::widgets::NativeTextInput;
use egui::emath::Align;
use egui::{Frame, Image, Layout, Response, Sense, Ui, Widget};
use log::info;

pub struct WriteChat {
    link: NostrLink,
    msg: String,
}

impl WriteChat {
    pub fn new(link: NostrLink) -> Self {
        Self {
            link,
            msg: String::new(),
        }
    }

    pub fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        let logo_bytes = include_bytes!("../resources/send-03.svg");
        Frame::new()
            .inner_margin(egui::Margin::symmetric(12, 8))
            .outer_margin(egui::Margin::same(12))
            .fill(NEUTRAL_900)
            .stroke(egui::Stroke::new(1.0, crate::theme::BORDER))
            .corner_radius(ROUNDING_DEFAULT)
            .show(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if Image::from_bytes("send-03.svg", logo_bytes)
                        .sense(Sense::click())
                        .ui(ui)
                        .clicked()
                        || self.msg.ends_with('\n')
                    {
                        if let Some(ev) = services.write_live_chat_msg(&self.link, self.msg.trim())
                        {
                            info!("Sending: {:?}", ev);
                            services.broadcast_event(ev);
                        }
                        self.msg.clear();
                    }

                    ui.add(NativeTextInput::new(&mut self.msg).with_hint_text("Message.."));
                });
            })
            .response
    }
}
