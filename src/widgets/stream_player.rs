use crate::widgets::VideoPlaceholder;
use egui::{Context, Response, Ui, Vec2, Widget};
use egui_video::{AudioDevice, Player};

pub struct StreamPlayer {
    player: Option<Player>,
    audio: AudioDevice,
}

impl StreamPlayer {
    pub fn new(ctx: &Context, url: &String) -> Self {
        let mut audio = AudioDevice::new().unwrap();
        Self {
            player: Player::new(ctx, url).map_or(None, |mut f| {
                f.add_audio(&mut audio).expect("Failed to add audio");
                f.start();
                Some(f)
            }),
            audio,
        }
    }
}

impl Widget for &mut StreamPlayer {
    fn ui(self, ui: &mut Ui) -> Response {
        let w = ui.available_width();
        let h = w / 16. * 9.;
        let size = Vec2::new(w, h);

        if let Some(p) = self.player.as_mut() {
            p.ui(ui, size)
        } else {
            VideoPlaceholder.ui(ui)
        }
    }
}
