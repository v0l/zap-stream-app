use crate::note_util::OwnedNote;
use crate::route::RouteServices;
use crate::services::ndb_wrapper::NDBWrapper;
use crate::widgets;
use crate::widgets::NostrWidget;
use egui::{Response, Ui, Widget};
use log::{error, info};
use nostrdb::{Filter, Ndb, Note, NoteKey, Subscription, Transaction};

pub struct HomePage {
    sub: Subscription,
    events: Vec<OwnedNote>,
    ndb: NDBWrapper,
}

impl HomePage {
    pub fn new(ndb: &NDBWrapper, tx: &Transaction) -> Self {
        let filter = [
            Filter::new()
                .kinds([30_311])
                .limit(10)
                .build()
        ];
        let (sub, events) = ndb.subscribe_with_results(&filter, tx, 100);
        Self {
            sub,
            events: events.iter().map(|e| OwnedNote(e.note_key.as_u64())).collect(),
            ndb: ndb.clone(),
        }
    }
}

impl Drop for HomePage {
    fn drop(&mut self) {
        self.ndb.unsubscribe(self.sub);
    }
}

impl NostrWidget for HomePage {
    fn render(&mut self, ui: &mut Ui, services: &RouteServices<'_>) -> Response {
        let new_notes = services.ndb.poll(self.sub, 100);
        new_notes.iter().for_each(|n| self.events.push(OwnedNote(n.as_u64())));

        let events: Vec<Note<'_>> = self.events.iter()
            .map(|n| services.ndb.get_note_by_key(services.tx, NoteKey::new(n.0)))
            .map_while(|f| f.map_or(None, |f| Some(f)))
            .collect();

        info!("HomePage events: {}", events.len());
        widgets::StreamList::new(&events, &services).ui(ui)
    }
}