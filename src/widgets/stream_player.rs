use crate::widgets::PlaceholderRect;
use egui::{Context, Response, Ui, Widget};
use egui_video::{Player, PlayerControls};

pub struct StreamPlayer {
    player: Option<Player>,
}

impl StreamPlayer {
    pub fn new(ctx: &Context, url: &String) -> Self {
        let mut p = Player::new(ctx, url);
        p.set_debug(true);
        p.start();
        Self { player: Some(p) }
    }
}

impl Widget for &mut StreamPlayer {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = ui.available_size();

        if let Some(p) = self.player.as_mut() {
            ui.add_sized(size, p)
        } else {
            PlaceholderRect.ui(ui)
        }
    }
}
