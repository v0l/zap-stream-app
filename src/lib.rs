
pub mod app;
mod link;
mod note_util;
mod route;
mod services;
mod stream_info;
mod widgets;
mod theme;
mod note_store;


#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;
#[cfg(target_os = "android")]
use winit::platform::android::EventLoopBuilderExtAndroid;

#[cfg(target_os = "android")]
#[no_mangle]
#[tokio::main]
pub async fn android_main(app: AndroidApp) {
    std::env::set_var("RUST_BACKTRACE", "full");
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));

    let mut options = eframe::NativeOptions::default();
    options.renderer = Renderer::Glow;

    let app_clone_for_event_loop = app.clone();
    options.event_loop_builder = Some(Box::new(move |builder| {
        builder.with_android_app(app_clone_for_event_loop);
    }));

    let data_path = app
        .external_data_path()
        .expect("external data path")
        .to_path_buf();

    let _res = eframe::run_native(
        "zap.stream",
        options,
        Box::new(move |cc| Ok(Box::new(ZapStreamApp::new(cc, data_path)))),
    );
}