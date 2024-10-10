use crate::route::RouteServices;
use egui::{Response, Ui, Widget};
use nostrdb::{Filter, Note};
use crate::widgets;

pub struct HomePage<'a> {
    services: &'a RouteServices<'a>,
}

impl<'a> HomePage<'a> {
    pub fn new(services: &'a RouteServices) -> Self {
        Self { services }
    }
}

impl<'a> Widget for HomePage<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let events = self.services.ndb.query(&self.services.tx, &[
            Filter::new()
                .kinds([30_311])
                .limit(10)
                .build()
        ], 10).unwrap();
        let events: Vec<Note<'_>> = events.iter().map(|v| v.note.clone()).collect();
        widgets::StreamList::new(&events, &self.services).ui(ui)
    }
}