use crate::app::ZapStreamApp;
use android_activity::AndroidApp;
use notedeck::Notedeck;

pub fn start_android(app: AndroidApp) {
    std::env::set_var("RUST_BACKTRACE", "full");
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let data_path = app
        .internal_data_path()
        .expect("internal data path")
        .to_path_buf();

    let options = eframe::NativeOptions {
        android_app: Some(app.clone()),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "zap.stream",
        options,
        Box::new(move |cc| {
            let args = vec!["argv0-placeholder".to_string()];
            let mut notedeck = Notedeck::init(&cc.egui_ctx, data_path.clone(), &args);
            notedeck.set_android_context(app.clone());
            notedeck.setup(&cc.egui_ctx);
            notedeck.set_app(ZapStreamApp::new(app.clone(), cc.wgpu_render_state.clone()));
            Ok(Box::new(notedeck))
        }),
    ) {
        eprintln!("{}", e);
    }
}
