use anyhow::Result;
use directories::ProjectDirs;
use egui::{Vec2, ViewportBuilder};
use log::{error, info};
use notedeck::Notedeck;
use zap_stream_app::app::ZapStreamApp;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size(Vec2::new(1300., 900.)),
        ..Default::default()
    };

    let data_path = ProjectDirs::from("stream", "zap", "zap_stream_app")
        .unwrap()
        .data_dir()
        .to_path_buf();

    info!("Data path: {}", data_path.display());
    if let Err(e) = eframe::run_native(
        "zap.stream",
        options,
        Box::new(move |cc| {
            let args: Vec<String> = std::env::args().collect();
            let mut notedeck = Notedeck::init(&cc.egui_ctx, data_path.clone(), &args);
            notedeck.setup(&cc.egui_ctx);
            notedeck.set_app(ZapStreamApp::new(cc.wgpu_render_state.clone()));
            Ok(Box::new(notedeck))
        }),
    ) {
        error!("{}", e);
    }
    Ok(())
}
