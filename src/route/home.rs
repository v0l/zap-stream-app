use crate::note_ref::NoteRef;
use crate::note_view::NotesView;
use crate::route::RouteServices;
use crate::stream_info::{StreamInfo, StreamStatus};
use crate::widgets;
use crate::widgets::{sub_or_poll, NostrWidget};
use egui::{Id, Response, RichText, ScrollArea, Ui};
use nostrdb::{Filter, Note, Subscription};
use std::collections::HashSet;

pub struct HomePage {
    events: HashSet<NoteRef>,
    sub: Option<Subscription>,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            events: HashSet::new(),
            sub: None,
        }
    }

    fn get_filters() -> Vec<Filter> {
        vec![Filter::new().kinds([30_311]).limit(100).build()]
    }
}

impl NostrWidget for HomePage {
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        ScrollArea::vertical()
            .show(ui, |ui| {
                let events: Vec<Note> = self
                    .events
                    .iter()
                    .map_while(|n| services.ctx.ndb.get_note_by_key(&services.tx, n.key).ok())
                    .collect();

                let events_live = NotesView::from_vec(
                    events
                        .iter()
                        .filter(|r| matches!(r.status(), StreamStatus::Live))
                        .collect(),
                );
                if events_live.len() > 0 {
                    widgets::StreamList::new(
                        Id::new("live-streams"),
                        events_live,
                        Some(RichText::new("Live").size(32.0)),
                    )
                    .render(ui, services);
                }
                let events_planned = NotesView::from_vec(
                    events
                        .iter()
                        .filter(|r| matches!(r.status(), StreamStatus::Planned))
                        .collect(),
                );
                if events_planned.len() > 0 {
                    widgets::StreamList::new(
                        Id::new("planned-streams"),
                        events_planned,
                        Some(RichText::new("Planned").size(32.0)),
                    )
                    .render(ui, services);
                }
                let events_ended = NotesView::from_vec(
                    events
                        .iter()
                        .filter(|r| matches!(r.status(), StreamStatus::Ended))
                        .collect(),
                );
                if events_ended.len() > 0 {
                    widgets::StreamList::new(
                        Id::new("ended-streams"),
                        events_ended,
                        Some(RichText::new("Ended").size(32.0)),
                    )
                    .render(ui, services);
                }
                ui.response()
            })
            .inner
    }

    fn update(&mut self, services: &mut RouteServices<'_, '_>) -> anyhow::Result<()> {
        sub_or_poll(
            services.ctx.ndb,
            &services.tx,
            &mut services.ctx.pool,
            &mut self.events,
            &mut self.sub,
            Self::get_filters(),
        )
    }
}
