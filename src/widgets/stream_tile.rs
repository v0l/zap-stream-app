use crate::link::NostrLink;
use crate::route::{image_from_cache, RouteServices, RouteType};
use crate::stream_info::{StreamInfo, StreamStatus};
use crate::theme::{NEUTRAL_800, NEUTRAL_900, PRIMARY, ROUNDING_DEFAULT};
use crate::widgets::avatar::Avatar;
use eframe::epaint::{Rounding, Vec2};
use egui::epaint::RectShape;
use egui::{
    vec2, Color32, CursorIcon, FontId, ImageSource, Label, Pos2, Rect, Response, RichText, Sense,
    TextWrapMode, Ui,
};
use nostrdb::Note;

pub struct StreamEvent<'a> {
    event: &'a Note<'a>,
}

impl<'a> StreamEvent<'a> {
    pub fn new(event: &'a Note<'a>) -> Self {
        Self { event }
    }

    pub fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response {
        ui.vertical(|ui| {
            ui.style_mut().spacing.item_spacing = Vec2::new(12., 16.);

            let host = self.event.host();
            let host_profile = services.profile(host);

            let w = ui.available_width();
            let h = (w / 16.0) * 9.0;

            let (response, painter) = ui.allocate_painter(Vec2::new(w, h), Sense::click());

            let cover = if ui.is_rect_visible(response.rect) {
                self.event
                    .image()
                    .and_then(|p| {
                        image_from_cache(services.ctx.img_cache, ui, p, Some(Vec2::new(w, h)))
                    })
                    .map(|i| i.rounding(ROUNDING_DEFAULT))
            } else {
                None
            };

            if let Some(cover) = cover {
                painter.add(RectShape {
                    rect: response.rect,
                    rounding: Rounding::same(ROUNDING_DEFAULT),
                    fill: Color32::WHITE,
                    stroke: Default::default(),
                    blur_width: 0.0,
                    fill_texture_id: match cover.source(ui.ctx()) {
                        ImageSource::Texture(t) => t.id,
                        _ => Default::default(),
                    },
                    uv: Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                });
            } else {
                painter.rect_filled(response.rect, ROUNDING_DEFAULT, NEUTRAL_800);
            }

            let overlay_label_pad = Vec2::new(5., 5.);
            let live_label_text = self.event.status().to_string().to_uppercase();
            let live_label_color = if self.event.status() == StreamStatus::Live {
                PRIMARY
            } else {
                NEUTRAL_900
            };
            let live_label =
                painter.layout_no_wrap(live_label_text, FontId::default(), Color32::WHITE);

            let overlay_react = response.rect.shrink(8.0);
            let live_label_pos = overlay_react.min
                + vec2(
                    overlay_react.width() - live_label.rect.width() - (overlay_label_pad.x * 2.),
                    0.0,
                );
            let live_label_background = Rect::from_two_pos(
                live_label_pos,
                live_label_pos + live_label.size() + (overlay_label_pad * 2.),
            );
            painter.rect_filled(live_label_background, 8., live_label_color);
            painter.galley(
                live_label_pos + overlay_label_pad,
                live_label,
                Color32::PLACEHOLDER,
            );

            if let Some(viewers) = self.event.viewers() {
                let viewers_label = painter.layout_no_wrap(
                    format!("{} viewers", viewers),
                    FontId::default(),
                    Color32::WHITE,
                );
                let rect_start =
                    overlay_react.max - viewers_label.size() - (overlay_label_pad * 2.0);
                let pos = Rect::from_two_pos(rect_start, overlay_react.max);
                painter.rect_filled(pos, 8., NEUTRAL_900);
                painter.galley(
                    rect_start + overlay_label_pad,
                    viewers_label,
                    Color32::PLACEHOLDER,
                );
            }
            let response = response.on_hover_and_drag_cursor(CursorIcon::PointingHand);
            if response.clicked() {
                services.navigate(RouteType::EventPage {
                    link: NostrLink::from_note(self.event),
                    event: None,
                });
            }
            ui.horizontal(|ui| {
                Avatar::from_profile(&host_profile)
                    .size(40.)
                    .render(ui, services.ctx.img_cache);
                let title = RichText::new(self.event.title().unwrap_or("Untitled"))
                    .size(16.)
                    .color(Color32::WHITE);
                ui.add(Label::new(title).wrap_mode(TextWrapMode::Truncate));
            })
        })
        .response
    }
}
