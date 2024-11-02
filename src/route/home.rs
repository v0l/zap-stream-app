use crate::note_store::NoteStore;
use crate::note_util::OwnedNote;
use crate::route::RouteServices;
use crate::services::ndb_wrapper::{NDBWrapper, SubWrapper};
use crate::stream_info::{StreamInfo, StreamStatus};
use crate::widgets;
use crate::widgets::NostrWidget;
use egui::{Id, Response, RichText, ScrollArea, Ui, Widget};
use nostrdb::{Filter, Note, NoteKey, Transaction};

pub struct HomePage {
    sub: SubWrapper,
    events: Vec<OwnedNote>,
}

impl HomePage {
    pub fn new(ndb: &NDBWrapper, tx: &Transaction) -> Self {
        let filter = [Filter::new().kinds([30_311]).limit(100).build()];
        let (sub, events) = ndb.subscribe_with_results("home-page", &filter, tx, 1000);
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
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_>) -> Response {
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

        ScrollArea::vertical()
            .show(ui, |ui| {
                let events_live = NoteStore::from_vec(
                    events
                        .iter()
                        .filter(|r| matches!(r.status(), StreamStatus::Live))
                        .collect(),
                );
                if events_live.len() > 0 {
                    widgets::StreamList::new(
                        Id::new("live-streams"),
                        &events_live,
                        services,
                        Some(RichText::new("Live").size(32.0)),
                    )
                    .ui(ui);
                }
                let events_planned = NoteStore::from_vec(
                    events
                        .iter()
                        .filter(|r| matches!(r.status(), StreamStatus::Planned))
                        .collect(),
                );
                if events_planned.len() > 0 {
                    widgets::StreamList::new(
                        Id::new("planned-streams"),
                        &events_planned,
                        services,
                        Some(RichText::new("Planned").size(32.0)),
                    )
                    .ui(ui);
                }
                let events_ended = NoteStore::from_vec(
                    events
                        .iter()
                        .filter(|r| matches!(r.status(), StreamStatus::Ended))
                        .collect(),
                );
                if events_ended.len() > 0 {
                    widgets::StreamList::new(
                        Id::new("ended-streams"),
                        &events_ended,
                        services,
                        Some(RichText::new("Ended").size(32.0)),
                    )
                    .ui(ui);
                }
                ui.response()
            })
            .inner
    }
}
