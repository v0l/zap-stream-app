use eframe::Renderer;
use crate::app::ZapStreamApp;
use egui::Vec2;
use nostrdb::Note;

mod app;
pub mod widgets;
mod services;
mod route;
mod note_util;
mod link;
mod stream_info;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut options = eframe::NativeOptions::default();
    options.renderer = Renderer::Glow;
    options.viewport = options.viewport
        .with_inner_size(Vec2::new(360., 720.));

    let _res = eframe::run_native(
        "zap.stream",
        options,
        Box::new(move |cc| Ok(Box::new(ZapStreamApp::new(cc)))),
    );
}