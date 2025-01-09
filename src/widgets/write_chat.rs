use crate::link::NostrLink;
use crate::route::RouteServices;
use crate::theme::{MARGIN_DEFAULT, NEUTRAL_900, ROUNDING_DEFAULT};
use crate::widgets::NativeTextInput;
use eframe::emath::Align;
use egui::{Frame, Layout, Response, Sense, Ui, Widget};
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
        Frame::none()
            .inner_margin(MARGIN_DEFAULT)
            .outer_margin(MARGIN_DEFAULT)
            .fill(NEUTRAL_900)
            .rounding(ROUNDING_DEFAULT)
            .show(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if services
                        .image_bytes("send-03.svg", logo_bytes)
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
