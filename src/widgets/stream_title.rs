use crate::note_util::NoteUtil;
use crate::route::RouteServices;
use crate::stream_info::StreamInfo;
use crate::theme::{self, MUTED, TEXT, TEXT_SOFT};
use crate::widgets::zap::ZapButton;
use crate::widgets::{Avatar, Button};
use egui::{Frame, Label, Margin, Response, RichText, TextWrapMode, Ui};
use nostrdb::Note;

pub struct StreamTitle<'a> {
    event: &'a Note<'a>,
}

impl<'a> StreamTitle<'a> {
    pub fn new(event: &'a Note<'a>) -> StreamTitle<'a> {
        StreamTitle { event }
    }

    pub fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        Frame::new()
            .inner_margin(Margin::symmetric(0, 8))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(8., 10.0);

                let size = if ui.available_width() < 560.0 { 24.0 } else { 30.72 };
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.add(
                        Label::new(theme::display(
                            self.event.title().unwrap_or("Untitled"),
                            size,
                        ))
                        .wrap_mode(TextWrapMode::Wrap),
                    );
                });

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    let profile = services.profile(self.event.host());
                    Avatar::from_profile(&profile)
                        .size(32.)
                        .render(ui, services.ctx);
                    let name = profile
                        .and_then(|p| p.name())
                        .unwrap_or("Nostrich")
                        .to_string();
                    ui.label(theme::semibold(name, 16.0).color(TEXT));
                    ui.add_space(8.0);
                    ZapButton::event(self.event).render(ui, services);
                });

                if let Some(summary) = self
                    .event
                    .get_tag_value("summary")
                    .and_then(|r| r.variant().str())
                {
                    if !summary.is_empty() {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                            ui.add(
                                Label::new(RichText::new(summary).color(TEXT_SOFT))
                                    .wrap_mode(TextWrapMode::Wrap),
                            );
                        });
                    }
                }

                let tags = self.tags();
                if !tags.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 14.0;
                        for t in tags {
                            ui.label(theme::medium(t, 15.0).color(MUTED));
                        }
                    });
                }
            })
            .response
    }

    fn tags(&self) -> Vec<String> {
        let mut out = Vec::new();
        for tag in self.event.tags() {
            if tag.get_unchecked(0).variant().str() == Some("t") {
                if let Some(v) = tag.get_unchecked(1).variant().str() {
                    if !v.is_empty() {
                        out.push(v.to_string());
                    }
                }
            }
        }
        out
    }
}
