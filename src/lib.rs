#[cfg(target_os = "android")]
mod android;
pub mod app;
mod link;
mod note_ref;
mod note_util;
mod note_view;
mod profiles;
mod route;
mod services;
mod stream_info;
mod sub;
mod theme;
mod widgets;
mod zap;

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

#[cfg(target_os = "android")]
#[no_mangle]
#[tokio::main]
pub async fn android_main(app: AndroidApp) {
    android::start_android(app);
}
