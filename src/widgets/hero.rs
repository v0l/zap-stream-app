use crate::link::NostrLink;
use crate::route::{image_from_cache, RouteServices, RouteType};
use crate::stream_info::StreamInfo;
use crate::theme::{self, ON_PRIMARY, PRIMARY, TEXT, TEXT_SOFT};
use crate::widgets::stream_tile::paint_cover;
use egui::epaint::{Color32, CornerRadius, Mesh, Shape};
use egui::{pos2, vec2, CursorIcon, Rect, Sense, Ui};
use nostrdb::Note;

pub struct Hero<'a> {
    event: &'a Note<'a>,
}

impl<'a> Hero<'a> {
    pub fn new(event: &'a Note<'a>) -> Self {
        Self { event }
    }

    pub fn render(
        &mut self,
        ui: &mut Ui,
        services: &mut RouteServices<'_, '_>,
        height: f32,
        preview: Option<&mut crate::widgets::StreamPlayer>,
    ) {
        let w = ui.available_width();
        let (response, painter) = ui.allocate_painter(vec2(w, height), Sense::click());
        let rect = response.rect;
        let radius = 16u8;

        let cover = self.event.image().and_then(|p| {
            image_from_cache(
                services.ctx.img_cache,
                services.ctx.media_jobs.sender(),
                ui,
                p,
                Some(vec2(w, height)),
            )
        });
        let has_cover = cover.is_some();
        paint_cover(ui, &painter, rect, radius, cover);
        if !has_cover {
            if let Some(p) = preview {
                p.show_in(ui, rect);
                crate::widgets::stream_player::mask_corners(ui, rect, radius as f32);
            }
        }

        let fade = Rect::from_min_max(pos2(rect.min.x, rect.max.y - height * 0.65), rect.max);
        let mut mesh = Mesh::default();
        let top = Color32::from_rgba_unmultiplied(22, 20, 27, 0);
        let bottom = Color32::from_rgba_unmultiplied(22, 20, 27, 242);
        let i = mesh.vertices.len() as u32;
        mesh.colored_vertex(fade.left_top(), top);
        mesh.colored_vertex(fade.right_top(), top);
        mesh.colored_vertex(fade.right_bottom(), bottom);
        mesh.colored_vertex(fade.left_bottom(), bottom);
        mesh.add_triangle(i, i + 1, i + 2);
        mesh.add_triangle(i, i + 2, i + 3);
        painter.add(Shape::mesh(mesh));
        painter.rect_stroke(
            rect,
            CornerRadius::same(radius),
            egui::Stroke::new(0.0, Color32::TRANSPARENT),
            egui::StrokeKind::Inside,
        );

        let pad = 28.0;
        let mut y = rect.max.y - pad;

        let button_galley = painter.layout_no_wrap(
            "Watch stream".to_string(),
            egui::FontId::new(16.0, theme::medium_family()),
            ON_PRIMARY,
        );
        let btn = Rect::from_min_size(
            pos2(
                rect.max.x - pad - (button_galley.size().x + 32.0),
                rect.max.y - pad - (button_galley.size().y + 20.0),
            ),
            vec2(button_galley.size().x + 32.0, button_galley.size().y + 20.0),
        );
        painter.rect_filled(btn, CornerRadius::same(12), PRIMARY);
        painter.galley(
            btn.min + vec2(16.0, 10.0),
            button_galley,
            Color32::PLACEHOLDER,
        );

        let viewers = self.event.viewers().unwrap_or(0);
        let host = services
            .profile(self.event.host())
            .and_then(|p| p.name())
            .unwrap_or("Nostrich")
            .to_string();
        let meta = painter.layout_no_wrap(
            format!("{}   {} watching", host, viewers),
            egui::FontId::new(16.0, egui::FontFamily::Proportional),
            TEXT_SOFT,
        );
        y -= meta.size().y;
        painter.galley(pos2(rect.min.x + pad, y), meta, Color32::PLACEHOLDER);

        let narrow = w < 560.0;
        let title_size = if narrow { 26.0 } else { 46.08 };
        let title_width = if narrow { w - pad * 2.0 } else { (w * 0.55).max(200.0) };
        let title = painter.layout(
            self.event.title().unwrap_or("Untitled").to_string(),
            egui::FontId::new(title_size, theme::display_family()),
            TEXT,
            title_width,
        );
        y -= title.size().y + 6.0;
        painter.galley(pos2(rect.min.x + pad, y), title, Color32::PLACEHOLDER);

        let live = painter.layout_no_wrap(
            "Live now".to_string(),
            egui::FontId::new(14.0, theme::medium_family()),
            TEXT,
        );
        y -= live.size().y + 8.0;
        painter.circle_filled(pos2(rect.min.x + pad + 5.0, y + live.size().y / 2.0), 5.0, PRIMARY);
        painter.galley(pos2(rect.min.x + pad + 18.0, y), live, Color32::PLACEHOLDER);

        let response = response.on_hover_and_drag_cursor(CursorIcon::PointingHand);
        if response.clicked() {
            services.navigate(RouteType::EventPage {
                link: NostrLink::from_note(self.event),
                event: None,
            });
        }
    }
}
