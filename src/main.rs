use crate::app::ZapStreamApp;
use eframe::Renderer;
use egui::Vec2;

mod app;
mod link;
mod note_util;
mod route;
mod services;
mod stream_info;
pub mod widgets;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // TODO: redirect FFMPEG logs to log file (noisy)

    let mut options = eframe::NativeOptions::default();
    options.renderer = Renderer::Glow;
    options.viewport = options.viewport.with_inner_size(Vec2::new(360., 720.));

    let _res = eframe::run_native(
        "zap.stream",
        options,
        Box::new(move |cc| Ok(Box::new(ZapStreamApp::new(cc)))),
    );
}
