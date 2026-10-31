use crate::note_view::NotesView;
use crate::route::RouteServices;
use crate::stream_info::StreamInfo;
use crate::widgets::stream_tile::StreamEvent;
use egui::{vec2, Frame, Response, Ui, WidgetText};

pub struct StreamList<'a> {
    id: egui::Id,
    streams: NotesView<'a>,
    heading: Option<WidgetText>,
}

impl<'a> StreamList<'a> {
    pub fn new(
        id: egui::Id,
        streams: NotesView<'a>,
        heading: Option<impl Into<WidgetText>>,
    ) -> Self {
        Self {
            id,
            streams,
            heading: heading.map(Into::into),
        }
    }

    pub fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        let cols = match ui.available_width() as u16 {
            0..560 => 1,
            560..820 => 2,
            820..1200 => 3,
            1200..1500 => 4,
            1500..2000 => 5,
            _ => 6,
        };

        let grid_padding = 20.;
        Frame::new()
            .show(ui, |ui| {
                let grid_spacing_consumed = (cols - 1) as f32 * grid_padding;
                let g_w = (ui.available_width() - grid_spacing_consumed) / cols as f32;
                if let Some(heading) = self.heading.take() {
                    ui.label(heading);
                }
                let mut events: Vec<_> = self.streams.iter().collect();
                events.sort_by(|a, b| {
                    a.status()
                        .cmp(&b.status())
                        .then(a.starts().cmp(&b.starts()).reverse())
                });
                for row in events.chunks(cols as usize) {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = grid_padding;
                        for event in row {
                            ui.allocate_ui_with_layout(
                                vec2(g_w, 0.0),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(g_w);
                                    StreamEvent::new(event).render(ui, services);
                                },
                            );
                        }
                    });
                    ui.add_space(grid_padding - 8.0);
                }
            })
            .response
    }
}
