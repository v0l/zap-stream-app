use crate::note_store::NoteStore;
use crate::route::RouteServices;
use crate::stream_info::StreamInfo;
use crate::widgets::stream_tile::StreamEvent;
use egui::{vec2, Frame, Grid, Margin, Response, Ui, Widget, WidgetText};
use itertools::Itertools;

pub struct StreamList<'a> {
    id: egui::Id,
    streams: &'a NoteStore<'a>,
    services: &'a RouteServices<'a>,
    heading: Option<WidgetText>,
}

impl<'a> StreamList<'a> {
    pub fn new(
        id: egui::Id,
        streams: &'a NoteStore<'a>,
        services: &'a RouteServices<'a>,
        heading: Option<impl Into<WidgetText>>,
    ) -> Self {
        Self {
            id,
            streams,
            services,
            heading: heading.map(Into::into),
        }
    }
}

impl Widget for StreamList<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let cols = match ui.available_width() as u16 {
            720..1080 => 2,
            1080..1300 => 3,
            1300..1500 => 4,
            1500..2000 => 5,
            2000.. => 6,
            _ => 1,
        };

        let grid_padding = 20.;
        let frame_margin = 16.0;
        Frame::none()
            .inner_margin(Margin::symmetric(frame_margin, 0.))
            .show(ui, |ui| {
                let grid_spacing_consumed = (cols - 1) as f32 * grid_padding;
                let g_w = (ui.available_width() - grid_spacing_consumed) / cols as f32;
                if let Some(heading) = self.heading {
                    ui.label(heading);
                }
                Grid::new(self.id)
                    .spacing(vec2(grid_padding, grid_padding))
                    .show(ui, |ui| {
                        let mut ctr = 0;
                        for event in self.streams.iter().sorted_by(|a, b| {
                            a.status()
                                .cmp(&b.status())
                                .then(a.starts().cmp(&b.starts()).reverse())
                        }) {
                            ui.add_sized(
                                vec2(g_w, (g_w / 16.0) * 9.0),
                                StreamEvent::new(event, self.services),
                            );
                            ctr += 1;
                            if ctr % cols == 0 {
                                ui.end_row();
                            }
                        }
                    })
            })
            .response
    }
}
