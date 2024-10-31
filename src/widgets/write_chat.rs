use crate::link::NostrLink;
use crate::route::{RouteAction, RouteServices};
use crate::theme::{MARGIN_DEFAULT, NEUTRAL_900, ROUNDING_DEFAULT};
use crate::widgets::{NativeTextInput, NostrWidget};
use eframe::emath::Align;
use egui::{Frame, Image, Layout, Margin, Response, Rounding, Sense, Stroke, TextEdit, Ui, Widget};
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
}

impl NostrWidget for WriteChat {
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_>) -> Response {
        let logo_bytes = include_bytes!("../resources/send-03.svg");
        Frame::none()
            .inner_margin(MARGIN_DEFAULT)
            .stroke(Stroke::new(1.0, NEUTRAL_900))
            .show(ui, |ui| {
                Frame::none()
                    .fill(NEUTRAL_900)
                    .rounding(ROUNDING_DEFAULT)
                    .inner_margin(MARGIN_DEFAULT)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if services
                                .img_cache
                                .load_bytes("send-03.svg", logo_bytes)
                                .sense(Sense::click())
                                .ui(ui)
                                .clicked()
                            {
                                if let Ok(ev) =
                                    services.login.write_live_chat_msg(&self.link, &self.msg)
                                {
                                    info!("Sending: {:?}", ev);
                                    services.broadcast_event(ev);
                                }
                                self.msg.clear();
                            }

                            ui.allocate_ui(ui.available_size(), |ui| {
                                let mut editor = NativeTextInput::new(&mut self.msg);
                                editor.render(ui, services);
                            });
                        });
                    })
            })
            .response
    }
}
