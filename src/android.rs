use crate::app::ZapStreamApp;
use android_activity::AndroidApp;
use notedeck::Notedeck;

pub fn start_android(app: AndroidApp) {
    std::env::set_var("RUST_BACKTRACE", "full");
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("zapstream")
            .with_max_level(log::LevelFilter::Info),
    );
    log::info!("start_android");

    std::panic::set_hook(Box::new(|info| log::error!("panic: {info}")));

    let data_path = app
        .internal_data_path()
        .expect("internal data path")
        .to_path_buf();

    log::info!("building vulkan setup");
    let options = eframe::NativeOptions {
        android_app: Some(app.clone()),
        system_font_fallback: false,
        wgpu_options: egui_wgpu::WgpuConfiguration {
            wgpu_setup: egui_wgpu::WgpuSetup::Existing(
                lumina_video::android_wgpu::android_wgpu_setup().expect("android vulkan setup"),
            ),
            ..Default::default()
        },
        ..Default::default()
    };

    let result = eframe::run_native(
        "zap.stream",
        options,
        Box::new(move |cc| {
            log::info!("app creator called");
            let args = crate::relays::args(vec!["argv0-placeholder".to_string()]);
            let mut notedeck = Notedeck::init(&cc.egui_ctx, data_path.clone(), &args);
            notedeck.set_android_context(app.clone());
            notedeck.setup(&cc.egui_ctx);
            notedeck.set_app(ZapStreamApp::new(app.clone(), cc.wgpu_render_state.clone()));
            Ok(Box::new(notedeck))
        }),
    );
    log::info!("run_native returned: {:?}", result.as_ref().err());
}
