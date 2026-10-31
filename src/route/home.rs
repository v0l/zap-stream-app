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
    zaps: HashSet<NoteRef>,
    zap_sub: Option<SubRef>,
    preview: Option<(String, widgets::StreamPlayer)>,
    board: (u64, Vec<([u8; 32], u64)>),
    board_len: usize,
    render_state: Option<egui_wgpu::RenderState>,
}

impl HomePage {
    pub fn new(render_state: Option<egui_wgpu::RenderState>) -> Self {
        Self {
            preview: None,
            board: (0, Vec::new()),
            board_len: usize::MAX,
            render_state,
            events: HashSet::new(),
            sub: None,
            zaps: HashSet::new(),
            zap_sub: None,
        }
    }

    fn zap_filters() -> Vec<Filter> {
        let since = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().saturating_sub(86_400))
            .unwrap_or(0);
        vec![Filter::new().kinds([9_735]).since(since).limit(500).build()]
    }

    fn leaderboard(&mut self, services: &mut RouteServices<'_, '_>) -> (u64, Vec<([u8; 32], u64)>) {
        if self.board_len == self.zaps.len() {
            return self.board.clone();
        }
        self.board_len = self.zaps.len();
        self.board = self.compute_leaderboard(services);
        self.board.clone()
    }

    fn compute_leaderboard(&self, services: &mut RouteServices<'_, '_>) -> (u64, Vec<([u8; 32], u64)>) {
        let mut total = 0u64;
        let mut by_sender: std::collections::HashMap<[u8; 32], u64> = Default::default();
        for r in &self.zaps {
            if let Ok(note) = services.ctx.ndb.get_note_by_key(services.tx, r.key) {
                if let Ok(z) = crate::zap::Zap::from_receipt(note) {
                    let sats = z.amount / 1000;
                    total += sats;
                    *by_sender.entry(z.sender).or_default() += sats;
                }
            }
        }
        let mut list: Vec<_> = by_sender.into_iter().collect();
        list.sort_by_key(|(_, v)| std::cmp::Reverse(*v));
        list.truncate(5);
        (total, list)
    }

    fn zapped_panel(
        &self,
        ui: &mut Ui,
        services: &mut RouteServices<'_, '_>,
        total: u64,
        top: &[([u8; 32], u64)],
    ) {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.label(theme::semibold("Zapped in the last 24 hours", 15.0));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.label(theme::display(crate::zap::format_sats(total as f32), 46.0).color(theme::PRIMARY));
                ui.label(theme::medium("sats", 16.0).color(theme::MUTED));
            });
            for (pk, sats) in top {
                let profile = services.profile(pk);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    crate::widgets::Avatar::from_profile(&profile)
                        .size(24.0)
                        .render(ui, services.ctx);
                    let name = profile.and_then(|p| p.name()).unwrap_or("Nostrich").to_string();
                    ui.label(theme::semibold(name, 15.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            theme::semibold(crate::zap::format_sats(*sats as f32), 15.0)
                                .color(theme::PRIMARY),
                        );
                    });
                });
            }
        });
    }

    fn tabs(ui: &mut Ui) -> Option<&'static str> {
        const TABS_TAGS: &[Option<&str>] = &[
            None,
            Some("irl"),
            Some("gaming"),
            Some("music"),
            Some("talk"),
            Some("art"),
            Some("gambling"),
            Some("science"),
        ];
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
        let sel_id = Id::new("home-category");
        let selected: usize = ui.ctx().data(|d| d.get_temp(sel_id)).unwrap_or(0);
        let mut clicked = None;
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
                let active = i == selected;
                let text = theme::medium(*t, 16.0).color(if active { theme::TEXT } else { theme::MUTED });
                let r = ui
                    .add(egui::Label::new(text).sense(egui::Sense::click()))
                    .on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
                if r.clicked() {
                    clicked = Some(i);
                }
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
        let selected = match clicked {
            Some(i) => {
                ui.ctx().data_mut(|d| d.insert_temp(sel_id, i));
                i
            }
            None => selected,
        };
        TABS_TAGS.get(selected).copied().flatten()
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

                        let wide_panel = wide && ui.available_width() > 900.0;
                        let (total, top_zappers) = if wide_panel {
                            let _t = crate::frame_stats::Section::start("leaderboard");
                            self.leaderboard(services)
                        } else {
                            (0, Vec::new())
                        };
                        let _q = crate::frame_stats::Section::start("home_body");
                        let preview_url = live.first().and_then(|t| {
                            if t.image().map(|i| !i.is_empty()).unwrap_or(false) {
                                None
                            } else {
                                t.streaming().map(|s| s.to_string())
                            }
                        });
                        match (&preview_url, &self.preview) {
                            (Some(u), Some((cur, _))) if u == cur => {}
                            (Some(u), _) => {
                                self.preview = Some((
                                    u.clone(),
                                    widgets::StreamPlayer::preview(self.render_state.as_ref(), u),
                                ));
                            }
                            (None, _) => self.preview = None,
                        }
                        if let Some(top) = live.first() {
                            let w = ui.available_width();
                            if wide && w > 900.0 {
                                let panel_w = 336.0;
                                let gap = 24.0;
                                ui.horizontal_top(|ui| {
                                    ui.spacing_mut().item_spacing.x = gap;
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(w - panel_w - gap, 365.0),
                                        egui::Layout::top_down(egui::Align::Min),
                                        |ui| {
                                            let p = self.preview.as_mut().map(|(_, p)| p);
                                            Hero::new(top).render(ui, services, 365.0, p)
                                        },
                                    );
                                    ui.allocate_ui_with_layout(
                                        egui::vec2(panel_w, 365.0),
                                        egui::Layout::top_down(egui::Align::Min),
                                        |ui| self.zapped_panel(ui, services, total, &top_zappers),
                                    );
                                });
                            } else {
                                let h = w * 0.5625;
                                let p = self.preview.as_mut().map(|(_, p)| p);
                                Hero::new(top).render(ui, services, h, p);
                            }
                        }

                        let filter = Self::tabs(ui);
                        let query: String = ui
                            .ctx()
                            .data(|d| d.get_temp(Id::new("search-query")))
                            .unwrap_or_default();
                        let query = query.to_lowercase();
                        live.retain(|e| {
                            filter.map_or(true, |f| e.hashtags().iter().any(|t| t.contains(f)))
                                && (query.is_empty()
                                    || e.title().unwrap_or("").to_lowercase().contains(&query))
                        });

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
            &mut self.zaps,
            &mut self.zap_sub,
            Self::zap_filters(),
        )?;
        sub_or_poll(
            services.ctx,
            services.tx,
            &mut self.events,
            &mut self.sub,
            Self::get_filters(),
        )
    }
}
