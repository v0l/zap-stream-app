use crate::link::NostrLink;
use crate::route::RouteServices;
use crate::theme::{MARGIN_DEFAULT, NEUTRAL_800, ROUNDING_DEFAULT};
use crate::widgets::{
    sub_or_poll, Chat, NostrWidget, PlaceholderRect, StreamPlayer, StreamTitle, WriteChat,
};
use egui::{vec2, Align, Frame, Layout, Response, ScrollArea, Stroke, Ui, Vec2, Widget};
use nostrdb::{Filter, Note};

use crate::note_ref::NoteRef;
use crate::stream_info::StreamInfo;
use crate::sub::SubRef;
use std::borrow::Borrow;
use std::collections::HashSet;

pub struct StreamPage {
    link: NostrLink,
    render_state: Option<egui_wgpu::RenderState>,
    player: Option<StreamPlayer>,
    chat: Option<Chat>,
    new_msg: WriteChat,

    events: HashSet<NoteRef>,
    sub: Option<SubRef>,
}

impl StreamPage {
    pub fn new_from_link(link: NostrLink, render_state: Option<egui_wgpu::RenderState>) -> Self {
        Self {
            render_state,
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
        let chat_h = 64.0;
        let w = ui.available_width();
        let h = ui.available_height();
        let margin = 16.0;
        ui.allocate_ui_with_layout(
            Vec2::new(w, h),
            Layout::top_down(Align::Min),
            |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                let video_h = (w / 16.0) * 9.0;
                let (rect, _) = ui.allocate_exact_size(vec2(w, video_h), egui::Sense::hover());
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
                if let Some(player) = &mut self.player {
                    child.add_sized(rect.size(), player);
                } else {
                    child.add_sized(rect.size(), PlaceholderRect);
                }

                let info_h = (h - video_h) * 0.38;
                ScrollArea::vertical()
                    .id_salt("stream-info")
                    .max_height(info_h)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        Frame::new()
                            .inner_margin(egui::Margin::symmetric(margin as i8, 8))
                            .show(ui, |ui| {
                                ui.set_width(w - margin * 2.0);
                                StreamTitle::new(event).render(ui, services);
                            });
                    });

                ui.painter().hline(
                    ui.max_rect().x_range(),
                    ui.cursor().top(),
                    Stroke::new(1.0, crate::theme::BORDER),
                );
                let remaining = ui.available_height();
                ui.allocate_ui_with_layout(
                    vec2(w, remaining),
                    Layout::top_down(Align::Min),
                    |ui| {
                        if let Some(c) = self.chat.as_mut() {
                            ui.allocate_ui(
                                vec2(w, (ui.available_height() - chat_h).max(0.0)),
                                |ui| c.render(ui, services),
                            );
                        } else {
                            ui.label("Loading..");
                        }
                        self.new_msg.render(ui, services);
                    },
                );
            },
        );
        ui.response()
    }

    fn chat_panel(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) {
        let chat_h = 64.0;
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(0., 0.);
            Frame::new()
                .inner_margin(egui::Margin {
                    left: 16,
                    right: 16,
                    top: 16,
                    bottom: 4,
                })
                .show(ui, |ui| {
                    ui.label(crate::theme::semibold("Stream chat", 15.0));
                });
            ui.painter().hline(
                ui.max_rect().x_range(),
                ui.cursor().top(),
                Stroke::new(1.0, crate::theme::BORDER),
            );
            if let Some(c) = self.chat.as_mut() {
                ui.allocate_ui(
                    vec2(ui.available_width(), (ui.available_height() - chat_h).max(0.0)),
                    |ui| {
                        c.render(ui, services);
                    },
                );
            } else {
                ui.label("Loading..");
            }
            self.new_msg.render(ui, services);
        });
    }

    fn render_desktop(
        &mut self,
        event: &Note<'_>,
        ui: &mut Ui,
        services: &mut RouteServices<'_, '_>,
    ) -> Response {
        let max_h = ui.available_height();
        let chat_w = 398.0;
        let gap = 24.0;
        let video_width = ui.available_width() - chat_w - gap;
        let video_height = max_h.min((video_width / 16.0) * 9.0);

        ui.horizontal_top(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            ui.allocate_ui_with_layout(
                vec2(video_width, max_h),
                Layout::top_down_justified(Align::Min),
                |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        let (rect, _) = ui.allocate_exact_size(
                            vec2(video_width, video_height),
                            egui::Sense::hover(),
                        );
                        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
                        if let Some(player) = &mut self.player {
                            child.add_sized(rect.size(), player);
                        } else {
                            child.add_sized(rect.size(), PlaceholderRect);
                        }
                        ui.add_space(14.);
                        StreamTitle::new(event).render(ui, services);
                    });
                },
            );
            ui.allocate_ui_with_layout(
                vec2(chat_w, max_h),
                Layout::top_down_justified(Align::Min),
                |ui| {
                    self.chat_panel(ui, services);
                },
            );
        });

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
            if self.player.is_none() {
                match event.kind() {
                    30_311 => {
                        if let Some(u) = event.streaming().or(event.recording()) {
                            let p = StreamPlayer::new(self.render_state.as_ref(), &u.to_string());
                            self.player = Some(p);
                        }
                    }
                    _ => {}
                };
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
            services.ctx,
            services.tx,
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
