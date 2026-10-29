use crate::widgets::PlaceholderRect;
use egui::{Response, Ui, Widget};
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

    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Widget for &mut StreamPlayer {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = ui.available_size();
        match self.player.as_mut() {
            Some(p) => {
                p.show(ui, size);
                ui.response()
            }
            None => PlaceholderRect.ui(ui),
        }
    }
}
