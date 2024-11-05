use crate::link::NostrLink;
use crate::note_util::OwnedNote;
use crate::route::RouteServices;
use crate::services::ndb_wrapper::{NDBWrapper, SubWrapper};
use crate::stream_info::StreamInfo;
use crate::theme::{MARGIN_DEFAULT, NEUTRAL_800, ROUNDING_DEFAULT};
use crate::widgets::{Chat, NostrWidget, PlaceholderRect, StreamPlayer, StreamTitle, WriteChat};
use egui::{vec2, Align, Frame, Layout, Response, Stroke, Ui, Vec2, Widget};
use nostrdb::{Filter, Note, NoteKey, Transaction};
use std::borrow::Borrow;

pub struct StreamPage {
    link: NostrLink,
    event: Option<OwnedNote>,
    player: Option<StreamPlayer>,
    chat: Option<Chat>,
    sub: SubWrapper,
    new_msg: WriteChat,
}

impl StreamPage {
    pub fn new_from_link(ndb: &NDBWrapper, tx: &Transaction, link: NostrLink) -> Self {
        let f: Filter = link.borrow().try_into().unwrap();
        let f = [f.limit_mut(1)];
        let (sub, events) = ndb.subscribe_with_results("streams", &f, tx, 1);
        Self {
            link: link.clone(),
            sub,
            event: events.first().map(|n| OwnedNote(n.note_key.as_u64())),
            chat: None,
            player: None,
            new_msg: WriteChat::new(link),
        }
    }
    fn render_mobile(
        &mut self,
        event: &Note<'_>,
        ui: &mut Ui,
        services: &mut RouteServices<'_>,
    ) -> Response {
        let chat_h = 60.0;
        let w = ui.available_width();
        let h = ui
            .available_height()
            .max(ui.available_rect_before_wrap().height());
        ui.allocate_ui_with_layout(
            Vec2::new(w, h),
            Layout::top_down_justified(Align::Min),
            |ui| {
                let video_h =
                    ((ui.available_width() / 16.0) * 9.0).min(ui.available_height() * 0.33);
                ui.allocate_ui(vec2(ui.available_width(), video_h), |ui| {
                    if let Some(player) = &mut self.player {
                        player.ui(ui)
                    } else {
                        ui.add(PlaceholderRect)
                    }
                });
                StreamTitle::new(&event).render(ui, services);

                if let Some(c) = self.chat.as_mut() {
                    ui.allocate_ui(
                        vec2(ui.available_width(), ui.available_height() - chat_h),
                        |ui| c.render(ui, services),
                    );
                } else {
                    ui.label("Loading..");
                }
                // consume rest of space
                if ui.available_height().is_finite() {
                    ui.add_space(ui.available_height() - chat_h);
                }
                self.new_msg.render(ui, services);
            },
        );
        ui.response()
    }

    fn render_desktop(
        &mut self,
        event: &Note<'_>,
        ui: &mut Ui,
        services: &mut RouteServices<'_>,
    ) -> Response {
        let max_h = ui.available_height();
        let chat_w = 450.0;
        let video_width = ui.available_width() - chat_w;
        let video_height = max_h.min((video_width / 16.0) * 9.0);

        ui.with_layout(
            Layout::left_to_right(Align::TOP).with_main_justify(true),
            |ui| {
                ui.vertical(|ui| {
                    ui.allocate_ui(vec2(video_width, video_height), |ui| {
                        if let Some(player) = &mut self.player {
                            player.ui(ui)
                        } else {
                            ui.add(PlaceholderRect)
                        }
                    });
                    ui.add_space(10.);
                    StreamTitle::new(&event).render(ui, services);
                });
                ui.allocate_ui_with_layout(
                    vec2(chat_w, max_h),
                    Layout::top_down_justified(Align::Min),
                    |ui| {
                        Frame::none()
                            .stroke(Stroke::new(1.0, NEUTRAL_800))
                            .outer_margin(MARGIN_DEFAULT)
                            .rounding(ROUNDING_DEFAULT)
                            .show(ui, |ui| {
                                let chat_h = 60.0;
                                if let Some(c) = self.chat.as_mut() {
                                    ui.allocate_ui(
                                        vec2(ui.available_width(), ui.available_height() - chat_h),
                                        |ui| {
                                            c.render(ui, services);
                                        },
                                    );
                                } else {
                                    ui.label("Loading..");
                                }
                                if ui.available_height().is_finite() {
                                    ui.add_space(ui.available_height() - chat_h);
                                }
                                self.new_msg.render(ui, services);
                            });
                    },
                );
            },
        );

        ui.response()
    }
}

impl NostrWidget for StreamPage {
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_>) -> Response {
        let poll = services.ndb.poll(&self.sub, 1);
        if let Some(k) = poll.first() {
            self.event = Some(OwnedNote(k.as_u64()))
        }

        let event = if let Some(k) = &self.event {
            services
                .ndb
                .get_note_by_key(services.tx, NoteKey::new(k.0))
                .ok()
        } else {
            None
        };
        if let Some(event) = event {
            if let Some(stream) = event.stream() {
                if self.player.is_none() {
                    let p = StreamPlayer::new(ui.ctx(), &stream.to_string());
                    self.player = Some(p);
                }
            }

            if self.chat.is_none() {
                let ok = OwnedNote(event.key().unwrap().as_u64());
                let chat = Chat::new(self.link.clone(), ok, services.ndb, services.tx);
                self.chat = Some(chat);
            }

            if ui.available_width() < 720.0 {
                self.render_mobile(&event, ui, services)
            } else {
                self.render_desktop(&event, ui, services)
            }
        } else {
            ui.label("Loading..")
        }
    }
}
