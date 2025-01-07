use anyhow::Result;
use directories::ProjectDirs;
use eframe::Renderer;
use egui::{Vec2, ViewportBuilder};
use log::error;
use zap_stream_app::app::ZapStreamApp;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut options = eframe::NativeOptions::default();
    options.viewport = ViewportBuilder::default().with_inner_size(Vec2::new(1300., 900.));
    options.renderer = Renderer::Glow;

    let data_path = ProjectDirs::from("stream", "zap", "app")
        .unwrap()
        .config_dir()
        .to_path_buf();

    if let Err(e) = eframe::run_native(
        "zap.stream",
        options,
        Box::new(move |cc| {
            let args: Vec<String> = std::env::args().collect();
            let mut notedeck =
                notedeck_chrome::Notedeck::new(&cc.egui_ctx, data_path.clone(), &args);

            let app = ZapStreamApp::new(cc);
            notedeck.add_app(app);

            Ok(Box::new(notedeck))
        }),
    ) {
        error!("{}", e);
    }
    Ok(())
}
