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
}

impl Chat {
    pub fn new<'a>(link: NostrLink, stream: NoteKey) -> Self {
        Self {
            link,
            stream,
            events: HashSet::new(),
            sub: None,
        }
    }

    pub fn get_filter(&self) -> Filter {
        Filter::new()
            .kinds([1_311, 9_735])
            .tags([self.link.to_tag_value()], 'a')
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

        ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                Frame::none()
                    .outer_margin(Margin::symmetric(12., 8.))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 8.0;
                            for ev in self
                                .events
                                .iter()
                                .sorted_by(|a, b| a.created_at.cmp(&b.created_at))
                            {
                                if let Ok(ev) =
                                    services.ctx.ndb.get_note_by_key(services.tx, ev.key)
                                {
                                    match ev.kind() {
                                        1311 => {
                                            let profile = services.profile(ev.pubkey());
                                            ChatMessage::new(&stream, &ev, &profile)
                                                .render(ui, services);
                                        }
                                        9735 => {
                                            if let Ok(zap) = Zap::from_receipt(ev) {
                                                let profile = services.profile(&zap.sender);
                                                ChatZap::new(&zap, &profile)
                                                    .render(ui, services.ctx.img_cache);
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        })
                    })
                    .response
            })
            .inner
    }

    fn update(&mut self, services: &mut RouteServices<'_, '_>) -> anyhow::Result<()> {
        let filters = vec![self.get_filter()];
        sub_or_poll(
            services.ctx.ndb,
            services.tx,
            services.ctx.pool,
            &mut self.events,
            &mut self.sub,
            filters,
        )
    }
}
