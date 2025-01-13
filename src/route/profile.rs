use crate::note_ref::NoteRef;
use crate::note_view::NotesView;
use crate::route::{image_from_cache, RouteServices};
use crate::sub::SubRef;
use crate::theme::{MARGIN_DEFAULT, ROUNDING_DEFAULT};
use crate::widgets::{sub_or_poll, NostrWidget, PlaceholderRect, Profile, StreamList};
use egui::{vec2, Frame, Id, Response, ScrollArea, Ui, Widget};
use nostrdb::{Filter, Note};
use std::collections::HashSet;

pub struct ProfilePage {
    pubkey: [u8; 32],
    events: HashSet<NoteRef>,
    sub: Option<SubRef>,
}

impl ProfilePage {
    pub fn new(pubkey: [u8; 32]) -> Self {
        Self {
            pubkey,
            events: HashSet::new(),
            sub: None,
        }
    }
}

impl NostrWidget for ProfilePage {
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        let profile = services.profile(&self.pubkey);

        ScrollArea::vertical().show(ui, |ui| {
            Frame::default()
                .inner_margin(MARGIN_DEFAULT)
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;

                    if let Some(banner) = profile.map(|p| p.banner()).flatten() {
                        image_from_cache(&mut services.ctx.img_cache, ui, banner)
                            .fit_to_exact_size(vec2(ui.available_width(), 360.0))
                            .rounding(ROUNDING_DEFAULT)
                            .ui(ui);
                    } else {
                        ui.add(PlaceholderRect);
                    }
                    Profile::from_profile(&self.pubkey, &profile)
                        .size(88.0)
                        .render(ui, services);
                });

            let events: Vec<Note> = self
                .events
                .iter()
                .filter_map(|e| services.ctx.ndb.get_note_by_key(services.tx, e.key).ok())
                .collect();

            StreamList::new(
                Id::from("profile-streams"),
                NotesView::from_vec(events.iter().collect()),
                Some("Past Streams"),
            )
            .render(ui, services);
        });
        ui.response()
    }

    fn update(&mut self, services: &mut RouteServices<'_, '_>) -> anyhow::Result<()> {
        sub_or_poll(
            services.ctx.ndb,
            services.tx,
            services.ctx.pool,
            &mut self.events,
            &mut self.sub,
            vec![
                Filter::new()
                    .kinds([30_311])
                    .authors(&[self.pubkey])
                    .build(),
                Filter::new()
                    .kinds([30_311])
                    .pubkeys(&[self.pubkey])
                    .build(),
            ],
        )
    }
}
