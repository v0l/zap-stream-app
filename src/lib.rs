#[cfg(target_os = "android")]
mod android;
pub mod app;
mod frame_stats;
mod link;
mod note_ref;
mod note_util;
mod note_view;
mod profiles;
mod route;
mod stream_info;
mod sub;
mod theme;
mod widgets;
mod zap;

#[cfg(target_os = "android")]
pub use lumina_video::*;

#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: android_activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("zapstream")
            .with_max_level(log::LevelFilter::Info),
    );
    std::panic::set_hook(Box::new(|info| log::error!("panic: {info}")));
    log::info!("android_main entered");
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    runtime.block_on(async move { android::start_android(app) });
}
