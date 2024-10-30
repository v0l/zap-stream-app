use eframe::Renderer;
use egui::{Margin, Vec2};
use std::path::PathBuf;
use zap_stream_app::app::{AppConfig, ZapStreamApp};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // TODO: redirect FFMPEG logs to log file (noisy)

    let mut options = eframe::NativeOptions::default();
    options.renderer = Renderer::Glow;
    options.viewport = options.viewport.with_inner_size(Vec2::new(360., 720.));

    let config = DesktopApp;
    let data_path = PathBuf::from("./.data");
    let _res = eframe::run_native(
        "zap.stream",
        options,
        Box::new(move |cc| Ok(Box::new(ZapStreamApp::new(cc, data_path, config)))),
    );
}

struct DesktopApp;

impl AppConfig for DesktopApp {
    fn frame_margin(&self) -> Margin {
        Margin::ZERO
    }
}
