use crate::link::NostrLink;
use crate::note_util::OwnedNote;
use crate::route::RouteServices;
use crate::services::ndb_wrapper::{NDBWrapper, SubWrapper};
use crate::widgets::chat_message::ChatMessage;
use crate::widgets::NostrWidget;
use egui::{Frame, Margin, Response, ScrollArea, Ui};
use itertools::Itertools;
use nostrdb::{Filter, Note, NoteKey, Transaction};

pub struct Chat {
    link: NostrLink,
    stream: OwnedNote,
    events: Vec<OwnedNote>,
    sub: SubWrapper,
}

impl Chat {
    pub fn new(link: NostrLink, stream: OwnedNote, ndb: &NDBWrapper, tx: &Transaction) -> Self {
        let filter = Filter::new()
            .kinds([1_311])
            .tags([link.to_tag_value()], 'a')
            .build();
        let filter = [filter];

        let (sub, events) = ndb.subscribe_with_results("live-chat", &filter, tx, 500);

        Self {
            link,
            sub,
            stream,
            events: events
                .iter()
                .map(|n| OwnedNote(n.note_key.as_u64()))
                .collect(),
        }
    }
}

impl NostrWidget for Chat {
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_>) -> Response {
        let poll = services.ndb.poll(&self.sub, 500);
        poll.iter()
            .for_each(|n| self.events.push(OwnedNote(n.as_u64())));

        let events: Vec<Note> = self
            .events
            .iter()
            .map_while(|n| {
                services
                    .ndb
                    .get_note_by_key(services.tx, NoteKey::new(n.0))
                    .ok()
            })
            .collect();

        let stream = services
            .ndb
            .get_note_by_key(services.tx, NoteKey::new(self.stream.0))
            .unwrap();

        ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                Frame::none()
                    .outer_margin(Margin::symmetric(12., 8.))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 8.0;
                            for ev in events
                                .into_iter()
                                .sorted_by(|a, b| a.created_at().cmp(&b.created_at()))
                            {
                                let c = ChatMessage::new(&stream, &ev, services);
                                ui.add(c);
                            }
                        })
                    })
                    .response
            })
            .inner
    }
}
