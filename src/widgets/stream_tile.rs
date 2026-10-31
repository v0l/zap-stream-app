use crate::link::NostrLink;
use crate::route::{image_from_cache, RouteServices, RouteType};
use crate::stream_info::{StreamInfo, StreamStatus};
use crate::theme::{self, MUTED, PRIMARY, SURFACE, TEXT};
use egui::epaint::{Brush, CornerRadius, RectShape};
use egui::{
    pos2, vec2, Color32, CursorIcon, FontId, ImageSource, Label, Painter, Rect, Response, Sense,
    TextWrapMode, Ui, Vec2,
};
use nostrdb::Note;
use std::sync::Arc;

pub struct StreamEvent<'a> {
    event: &'a Note<'a>,
}

pub fn paint_cover(
    ui: &mut Ui,
    painter: &Painter,
    rect: Rect,
    radius: u8,
    cover: Option<egui::Image<'_>>,
) {
    let texture_id = cover.and_then(|c| match c.source(ui.ctx()) {
        ImageSource::Texture(t) => Some(t.id),
        _ => None,
    });
    match texture_id {
        Some(id) => {
            let mut shape = RectShape::filled(rect, CornerRadius::same(radius), Color32::WHITE);
            shape.brush = Some(Arc::new(Brush {
                fill_texture_id: id,
                uv: Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            }));
            painter.add(shape);
        }
        None => {
            painter.rect_filled(rect, CornerRadius::same(radius), SURFACE);
        }
    }
}

pub fn paint_live_pill(painter: &Painter, rect: Rect, status: StreamStatus) {
    let label = match status {
        StreamStatus::Live => "Live",
        StreamStatus::Planned => "Planned",
        StreamStatus::Ended => "Ended",
    };
    let galley = painter.layout_no_wrap(
        label.to_string(),
        FontId::new(13.0, theme::semibold_family()),
        TEXT,
    );
    let dot = 7.0;
    let pad = vec2(8.0, 2.0);
    let gap = if status == StreamStatus::Live { 6.0 } else { 0.0 };
    let dot_w = if status == StreamStatus::Live { dot } else { 0.0 };
    let size = vec2(galley.size().x + dot_w + gap + pad.x * 2.0, galley.size().y + pad.y * 2.0);
    let r = Rect::from_min_size(rect.min + vec2(10.0, 10.0), size);
    painter.rect_filled(
        r,
        CornerRadius::same(6),
        Color32::from_rgba_unmultiplied(22, 20, 27, 204),
    );
    let mut x = r.min.x + pad.x;
    if status == StreamStatus::Live {
        painter.circle_filled(pos2(x + dot / 2.0, r.center().y), dot / 2.0, PRIMARY);
        x += dot + gap;
    }
    painter.galley(pos2(x, r.min.y + pad.y), galley, Color32::PLACEHOLDER);
}

impl<'a> StreamEvent<'a> {
    pub fn new(event: &'a Note<'a>) -> Self {
        Self { event }
    }

    pub fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0., 10.);

            let host = self.event.host();
            let host_profile = services.profile(host);

            let w = ui.available_width();
            let h = (w / 16.0) * 9.0;
            let (response, painter) = ui.allocate_painter(Vec2::new(w, h), Sense::click());

            let cover = if ui.is_rect_visible(response.rect) {
                self.event.image().and_then(|p| {
                    image_from_cache(
                        services.ctx.img_cache,
                        services.ctx.media_jobs.sender(),
                        ui,
                        p,
                        Some(Vec2::new(w, h)),
                    )
                })
            } else {
                None
            };
            paint_cover(ui, &painter, response.rect, 12, cover);
            paint_live_pill(&painter, response.rect, self.event.status());

            let response = response.on_hover_and_drag_cursor(CursorIcon::PointingHand);
            let mut open = response.clicked();

            let title = theme::display_bold_text(self.event.title().unwrap_or("Untitled"), 19.0);
            let t = ui
                .add(Label::new(title).wrap_mode(TextWrapMode::Truncate).sense(Sense::click()))
                .on_hover_and_drag_cursor(CursorIcon::PointingHand);
            open |= t.clicked();

            let name = host_profile
                .and_then(|p| p.name())
                .unwrap_or("Nostrich")
                .to_string();
            ui.add(
                Label::new(theme::medium(name, 14.5).color(MUTED))
                    .wrap_mode(TextWrapMode::Truncate),
            );

            if open {
                services.navigate(RouteType::EventPage {
                    link: NostrLink::from_note(self.event),
                    event: None,
                });
            }
        })
        .response
    }
}
