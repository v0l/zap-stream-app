use crate::app::ZapStreamApp;
use eframe::Renderer;
use egui::ViewportBuilder;
use winit::platform::android::activity::AndroidApp;
use winit::platform::android::EventLoopBuilderExtAndroid;

pub fn start_android(app: AndroidApp) {
    std::env::set_var("RUST_BACKTRACE", "full");
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let mut options = eframe::NativeOptions::default();
    options.renderer = Renderer::Glow;

    options.viewport = ViewportBuilder::default()
        .with_active(true)
        .with_always_on_top()
        .with_fullscreen(true);

    let app_clone_for_event_loop = app.clone();
    options.event_loop_builder = Some(Box::new(move |builder| {
        builder.with_android_app(app_clone_for_event_loop);
    }));

    let data_path = app
        .external_data_path()
        .expect("external data path")
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
        eprintln!("{}", e);
    }
}
