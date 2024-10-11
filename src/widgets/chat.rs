use crate::link::NostrLink;
use crate::note_util::OwnedNote;
use crate::route::RouteServices;
use crate::services::ndb_wrapper::NDBWrapper;
use crate::widgets::chat_message::ChatMessage;
use crate::widgets::NostrWidget;
use egui::{Response, ScrollArea, Ui, Widget};
use nostrdb::{Filter, Note, NoteKey, Subscription, Transaction};
use std::borrow::Borrow;

pub struct Chat {
    link: NostrLink,
    events: Vec<OwnedNote>,
    sub: Subscription,
}

impl Chat {
    pub fn new(link: NostrLink, ndb: &NDBWrapper, tx: &Transaction) -> Self {
        let filter = Filter::new()
            .kinds([1_311])
            .tags([link.to_tag_value()], 'a')
            .build();
        let filter = [filter];

        let (sub, events) = ndb.subscribe_with_results(&filter, tx, 500);

        Self {
            link,
            sub,
            events: events.iter().map(|n| OwnedNote(n.note_key.as_u64())).collect(),
        }
    }
}

impl NostrWidget for Chat {
    fn render(&mut self, ui: &mut Ui, services: &RouteServices<'_>) -> Response {
        let poll = services.ndb.poll(self.sub, 500);
        poll.iter().for_each(|n| self.events.push(OwnedNote(n.as_u64())));

        let events: Vec<Note> = self.events.iter().map_while(|n|
            services.ndb
                .get_note_by_key(services.tx, NoteKey::new(n.0))
                .map_or(None, |n| Some(n))
        ).collect();

        ScrollArea::vertical().show(ui, |ui| {
            ui.vertical(|ui| {
                for ev in events {
                    ChatMessage::new(&ev, services).ui(ui);
                }
            }).response
        }).inner
    }
}