use crate::note_ref::NoteRef;
use crate::note_view::NotesView;
use crate::route::RouteServices;
use crate::sub::SubRef;
use crate::theme::{self, MUTED, PRIMARY, TEXT_SOFT};
use crate::widgets::{sub_or_poll, Avatar, NostrWidget, StreamList};
use egui::{vec2, Frame, Id, Label, Margin, Response, RichText, ScrollArea, TextWrapMode, Ui};
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

    fn stat(ui: &mut Ui, value: usize, label: &str) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(theme::display(value.to_string(), 30.0).color(PRIMARY));
            ui.label(theme::medium(label, 14.0).color(MUTED));
        });
    }
}

impl NostrWidget for ProfilePage {
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        let profile = services.profile(&self.pubkey);
        let wide = ui.available_width() >= theme::WIDE_BREAKPOINT;

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                Frame::new()
                    .inner_margin(Margin::symmetric(if wide { 0 } else { 16 }, 16))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = vec2(16.0, 14.0);

                        ui.horizontal_top(|ui| {
                            Avatar::from_profile(&profile)
                                .size(132.0)
                                .ring(PRIMARY)
                                .render(ui, services.ctx);
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 8.0;
                                let name = profile
                                    .and_then(|p| p.display_name().or(p.name()))
                                    .unwrap_or("Nostrich")
                                    .to_string();
                                ui.add(
                                    Label::new(theme::display(
                                        name,
                                        if wide { 48.0 } else { 32.0 },
                                    ))
                                    .wrap_mode(TextWrapMode::Wrap),
                                );
                                if let Some(site) = profile.and_then(|p| p.website()) {
                                    if !site.is_empty() {
                                        ui.label(theme::medium(site, 14.5).color(MUTED));
                                    }
                                }
                            });
                        });

                        if let Some(about) = profile.and_then(|p| p.about()) {
                            if !about.is_empty() {
                                ui.add(
                                    Label::new(RichText::new(about).size(16.0).color(TEXT_SOFT))
                                        .wrap_mode(TextWrapMode::Wrap),
                                );
                            }
                        }

                        let count = self.events.len();
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 48.0;
                            Self::stat(ui, 0, "Sats received");
                            Self::stat(ui, 0, "Supporters");
                            Self::stat(ui, count, "Past streams");
                        });

                        let events: Vec<Note> = self
                            .events
                            .iter()
                            .filter_map(|e| {
                                services.ctx.ndb.get_note_by_key(services.tx, e.key).ok()
                            })
                            .collect();

                        StreamList::new(
                            Id::new("profile-streams"),
                            NotesView::from_vec(events.iter().collect()),
                            None::<RichText>,
                        )
                        .render(ui, services);
                    });
            });
        ui.response()
    }

    fn update(&mut self, services: &mut RouteServices<'_, '_>) -> anyhow::Result<()> {
        sub_or_poll(
            services.ctx,
            services.tx,
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
