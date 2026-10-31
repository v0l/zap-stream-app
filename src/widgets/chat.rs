use crate::link::NostrLink;
use crate::note_ref::NoteRef;
use crate::route::RouteServices;
use crate::sub::SubRef;
use crate::widgets::chat_message::ChatMessage;
use crate::widgets::chat_zap::ChatZap;
use crate::widgets::{sub_or_poll, NostrWidget};
use crate::zap::Zap;
use egui::{Frame, Margin, Response, ScrollArea, Ui};
use itertools::Itertools;
use nostrdb::{Filter, NoteKey};
use std::collections::HashSet;

pub struct Chat {
    link: NostrLink,
    stream: NoteKey,
    events: HashSet<NoteRef>,
    sub: Option<SubRef>,
    sorted: Vec<NoteRef>,
    sorted_len: usize,
    heights: std::collections::HashMap<NoteKey, f32>,
    measured_width: f32,
}

impl Chat {
    pub fn new<'a>(link: NostrLink, stream: NoteKey) -> Self {
        Self {
            link,
            stream,
            events: HashSet::new(),
            sub: None,
            sorted: Vec::new(),
            sorted_len: usize::MAX,
            heights: Default::default(),
            measured_width: 0.0,
        }
    }

    pub fn get_filter(&self) -> Filter {
        Filter::new()
            .kinds([1_311, 9_735])
            .tags([self.link.to_tag_value().as_str()], 'a')
            .build()
    }
}

impl NostrWidget for Chat {
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        let stream = services
            .ctx
            .ndb
            .get_note_by_key(services.tx, self.stream)
            .unwrap();

        if self.sorted_len != self.events.len() {
            self.sorted = self
                .events
                .iter()
                .copied()
                .sorted_by(|a, b| a.created_at.cmp(&b.created_at))
                .collect();
            self.sorted_len = self.events.len();
        }

        let width = ui.available_width();
        if (self.measured_width - width).abs() > 1.0 {
            self.heights.clear();
            self.measured_width = width;
        }

        let estimate = 32.0;
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show_viewport(ui, |ui, viewport| {
                ui.spacing_mut().item_spacing.y = 0.0;
                let margin = 200.0;
                let mut y = 0.0;
                for idx in 0..self.sorted.len() {
                    let r = self.sorted[idx];
                    let known = self.heights.get(&r.key).copied();
                    let h = known.unwrap_or(estimate);
                    let offscreen =
                        known.is_some() && (y + h < viewport.min.y - margin || y > viewport.max.y + margin);
                    if offscreen {
                        ui.add_space(h);
                        y += h;
                        continue;
                    }
                    let Ok(ev) = services.ctx.ndb.get_note_by_key(services.tx, r.key) else {
                        continue;
                    };
                    let rect = ui
                        .scope(|ui| match ev.kind() {
                            1311 => {
                                let profile = services.profile(ev.pubkey());
                                ChatMessage::new(&stream, &ev, &profile).render(ui, services);
                            }
                            9735 => {
                                if let Ok(zap) = Zap::from_receipt(ev) {
                                    let profile = services.profile(&zap.sender);
                                    ChatZap::new(&zap, &profile).render(ui, services.ctx);
                                }
                            }
                            _ => {}
                        })
                        .response
                        .rect;
                    ui.add_space(8.0);
                    let measured = rect.height() + 8.0;
                    self.heights.insert(r.key, measured);
                    y += measured;
                }
            });
        ui.response()
    }

    fn update(&mut self, services: &mut RouteServices<'_, '_>) -> anyhow::Result<()> {
        let filters = vec![self.get_filter()];
        sub_or_poll(
            services.ctx,
            services.tx,
            &mut self.events,
            &mut self.sub,
            filters,
        )
    }
}
