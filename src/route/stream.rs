use crate::link::NostrLink;
use crate::route::RouteServices;
use crate::stream_info::StreamInfo;
use crate::theme::{MARGIN_DEFAULT, NEUTRAL_800, ROUNDING_DEFAULT};
use crate::widgets::{
    sub_or_poll, Chat, NostrWidget, PlaceholderRect, StreamPlayer, StreamTitle, WriteChat,
};
use egui::{vec2, Align, Frame, Layout, Response, Stroke, Ui, Vec2, Widget};
use nostrdb::{Filter, Note};

use crate::note_ref::NoteRef;
use crate::sub::SubRef;
use std::borrow::Borrow;
use std::collections::HashSet;

pub struct StreamPage {
    link: NostrLink,
    player: Option<StreamPlayer>,
    chat: Option<Chat>,
    new_msg: WriteChat,

    events: HashSet<NoteRef>,
    sub: Option<SubRef>,
}

impl StreamPage {
    pub fn new_from_link(link: NostrLink) -> Self {
        Self {
            new_msg: WriteChat::new(link.clone()),
            link,
            chat: None,
            player: None,
            events: HashSet::new(),
            sub: None,
        }
    }

    fn get_filters(&self) -> Vec<Filter> {
        let f: Filter = self.link.borrow().try_into().unwrap();
        vec![f.limit_mut(1)]
    }

    fn render_mobile(
        &mut self,
        event: &Note<'_>,
        ui: &mut Ui,
        services: &mut RouteServices<'_, '_>,
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
                StreamTitle::new(event).render(ui, services);

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
        services: &mut RouteServices<'_, '_>,
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
                    StreamTitle::new(event).render(ui, services);
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
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        let events: Vec<Note> = self
            .events
            .iter()
            .map_while(|e| services.ctx.ndb.get_note_by_key(services.tx, e.key).ok())
            .collect();

        if let Some(event) = events.first() {
            if let Some(stream) = event.stream() {
                if self.player.is_none() {
                    let p = StreamPlayer::new(ui.ctx(), &stream.to_string());
                    self.player = Some(p);
                }
            }

            if self.chat.is_none() {
                let ok = event.key().unwrap();
                let chat = Chat::new(self.link.clone(), ok);
                self.chat = Some(chat);
            }

            if ui.available_width() < 720.0 {
                self.render_mobile(event, ui, services)
            } else {
                self.render_desktop(event, ui, services)
            }
        } else {
            ui.label("Loading..")
        }
    }

    fn update(&mut self, services: &mut RouteServices<'_, '_>) -> anyhow::Result<()> {
        let filters = self.get_filters();
        sub_or_poll(
            services.ctx.ndb,
            services.tx,
            services.ctx.pool,
            &mut self.events,
            &mut self.sub,
            filters,
        )?;
        if let Some(c) = self.chat.as_mut() {
            c.update(services)?;
        }
        Ok(())
    }
}
