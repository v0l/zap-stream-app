use crate::widgets::PlaceholderRect;
use crate::theme::BG;
use egui::epaint::{Mesh, Shape};
use egui::{pos2, Pos2, Rect, Response, Ui, Widget};
use lumina_video::VideoPlayer;

pub struct StreamPlayer {
    player: Option<VideoPlayer>,
    url: String,
}

impl StreamPlayer {
    pub fn new(render_state: Option<&egui_wgpu::RenderState>, url: &str) -> Self {
        let player = render_state.map(|rs| VideoPlayer::with_wgpu(url, rs).with_autoplay(true));
        Self {
            player,
            url: url.to_string(),
        }
    }

    pub fn preview(render_state: Option<&egui_wgpu::RenderState>, url: &str) -> Self {
        let player = render_state.map(|rs| {
            VideoPlayer::with_wgpu(url, rs)
                .with_autoplay(true)
                .with_muted(true)
                .with_controls(false)
        });
        Self {
            player,
            url: url.to_string(),
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

impl StreamPlayer {
    pub fn show_in(&mut self, ui: &mut Ui, rect: Rect) -> bool {
        match self.player.as_mut() {
            Some(p) => {
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
                p.show(&mut child, rect.size());
                true
            }
            None => false,
        }
    }
}

impl Widget for &mut StreamPlayer {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = ui.available_size();
        match self.player.as_mut() {
            Some(p) => {
                let response = p.show(ui, size);
                mask_corners(ui, response.response.rect, 16.0);
                ui.response()
            }
            None => PlaceholderRect.ui(ui),
        }
    }
}

pub fn mask_corners(ui: &Ui, rect: Rect, r: f32) {
    let painter = ui.painter();
    let steps = 8;
    let corners: [(Pos2, f32, f32); 4] = [
        (rect.left_top(), 1.0, 1.0),
        (rect.right_top(), -1.0, 1.0),
        (rect.right_bottom(), -1.0, -1.0),
        (rect.left_bottom(), 1.0, -1.0),
    ];
    for (corner, sx, sy) in corners {
        let center = pos2(corner.x + sx * r, corner.y + sy * r);
        let mut mesh = Mesh::default();
        mesh.colored_vertex(corner, BG);
        for i in 0..=steps {
            let a = std::f32::consts::FRAC_PI_2 * i as f32 / steps as f32;
            let p = pos2(center.x - sx * r * a.sin(), center.y - sy * r * a.cos());
            mesh.colored_vertex(p, BG);
        }
        for i in 0..steps {
            mesh.add_triangle(0, 1 + i as u32, 2 + i as u32);
        }
        painter.add(Shape::mesh(mesh));
    }
}
