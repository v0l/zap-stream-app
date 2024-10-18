use crate::note_store::NoteStore;
use crate::note_util::OwnedNote;
use crate::route::RouteServices;
use crate::services::ndb_wrapper::{NDBWrapper, SubWrapper};
use crate::widgets;
use crate::widgets::NostrWidget;
use egui::{Response, ScrollArea, Ui, Widget};
use nostrdb::{Filter, Note, NoteKey, Transaction};

pub struct HomePage {
    sub: SubWrapper,
    events: Vec<OwnedNote>,
}

impl HomePage {
    pub fn new(ndb: &NDBWrapper, tx: &Transaction) -> Self {
        let filter = [Filter::new().kinds([30_311]).limit(10).build()];
        let (sub, events) = ndb.subscribe_with_results("home-page", &filter, tx, 100);
        Self {
            sub,
            events: events
                .iter()
                .map(|e| OwnedNote(e.note_key.as_u64()))
                .collect(),
        }
    }
}

impl NostrWidget for HomePage {
    fn render(&mut self, ui: &mut Ui, services: &RouteServices<'_>) -> Response {
        let new_notes = services.ndb.poll(&self.sub, 100);
        new_notes
            .iter()
            .for_each(|n| self.events.push(OwnedNote(n.as_u64())));

        let events: Vec<Note<'_>> = self
            .events
            .iter()
            .map(|n| services.ndb.get_note_by_key(services.tx, NoteKey::new(n.0)))
            .map_while(|f| f.ok())
            .collect();

        let events = NoteStore::from_vec(events);
        ScrollArea::vertical()
            .show(ui, |ui| {
                widgets::StreamList::new(&events, services).ui(ui)
            }).inner
    }
}
