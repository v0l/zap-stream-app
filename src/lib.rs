#[cfg(target_os = "android")]
mod android;
pub mod app;
mod link;
mod login;
mod note_store;
mod note_util;
mod route;
mod services;
mod stream_info;
mod theme;
mod widgets;

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

#[cfg(target_os = "android")]
#[no_mangle]
#[tokio::main]
pub async fn android_main(app: AndroidApp) {
    android::start_android(app);
}
