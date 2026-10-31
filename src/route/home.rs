use crate::note_ref::NoteRef;
use crate::note_view::NotesView;
use crate::route::RouteServices;
use crate::stream_info::{StreamInfo, StreamStatus};
use crate::sub::SubRef;
use crate::theme;
use crate::widgets;
use crate::widgets::Hero;
use crate::widgets::{sub_or_poll, NostrWidget};
use egui::{Id, Response, ScrollArea, Ui};
use nostrdb::{Filter, Note};
use std::collections::HashSet;

pub struct HomePage {
    events: HashSet<NoteRef>,
    sub: Option<SubRef>,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            events: HashSet::new(),
            sub: None,
        }
    }

    fn tabs(ui: &mut Ui) {
        const TABS: &[&str] = &[
            "All live",
            "IRL",
            "Gaming",
            "Music",
            "Talk",
            "Art",
            "Gambling",
            "Science & Technology",
        ];
        let width = ui.available_width();
        egui::ScrollArea::horizontal()
            .id_salt("category-tabs")
            .max_width(width)
            .auto_shrink([false, true])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 24.0;
            for (i, t) in TABS.iter().enumerate() {
                let active = i == 0;
                let text = theme::medium(*t, 16.0).color(if active { theme::TEXT } else { theme::MUTED });
                let r = ui.add(egui::Label::new(text).sense(egui::Sense::click()));
                if active {
                    let y = r.rect.bottom() + 6.0;
                    ui.painter().hline(
                        r.rect.x_range(),
                        y,
                        egui::Stroke::new(2.0, theme::PRIMARY),
                    );
                }
            }
        });
            });
        let y = ui.cursor().top() - 4.0;
        ui.painter().hline(
            ui.max_rect().x_range(),
            y,
            egui::Stroke::new(1.0, theme::BORDER),
        );
    }

    fn get_filters() -> Vec<Filter> {
        vec![Filter::new().kinds([30_311, 30_313]).limit(100).build()]
    }
}

impl NostrWidget for HomePage {
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let wide = ui.available_width() >= theme::WIDE_BREAKPOINT;
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(if wide { 0 } else { 16 }, 12))
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 16.0);
                        let events: Vec<Note> = self
                            .events
                            .iter()
                            .filter_map(|n| {
                                services.ctx.ndb.get_note_by_key(services.tx, n.key).ok()
                            })
                            .filter(|e| e.can_play())
                            .collect();

                        let mut live: Vec<&Note> = events
                            .iter()
                            .filter(|r| matches!(r.status(), StreamStatus::Live))
                            .collect();
                        live.sort_by_key(|e| std::cmp::Reverse(e.viewers().unwrap_or(0)));

                        if let Some(top) = live.first() {
                            let w = ui.available_width();
                            let h = if wide { (w * 0.5625).min(365.0) } else { w * 0.5625 };
                            Hero::new(top).render(ui, services, h);
                        }

                        Self::tabs(ui);

                        let events_live = NotesView::from_vec(live.iter().copied().collect());
                        if events_live.len() > 0 {
                            widgets::StreamList::new(
                                Id::new("live-streams"),
                                events_live,
                                None::<egui::RichText>,
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
                                Some(theme::semibold("Planned", 15.0)),
                            )
                            .render(ui, services);
                        }
                        let events_ended = NotesView::from_vec(
                            events
                                .iter()
                                .filter(|r| {
                                    matches!(r.status(), StreamStatus::Ended)
                                        && r.recording().map(|r| !r.is_empty()).unwrap_or(false)
                                })
                                .collect(),
                        );
                        if events_ended.len() > 0 {
                            widgets::StreamList::new(
                                Id::new("ended-streams"),
                                events_ended,
                                Some(theme::semibold("Ended", 15.0)),
                            )
                            .render(ui, services);
                        }
                    });
                ui.response()
            })
            .inner
    }

    fn update(&mut self, services: &mut RouteServices<'_, '_>) -> anyhow::Result<()> {
        sub_or_poll(
            services.ctx,
            services.tx,
            &mut self.events,
            &mut self.sub,
            Self::get_filters(),
        )
    }
}
