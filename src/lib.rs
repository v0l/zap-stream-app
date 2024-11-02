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
use log::log;
use std::ffi::CStr;
use std::ptr;

#[cfg(target_os = "macos")]
type VaList = egui_video::ffmpeg_sys_the_third::va_list;
#[cfg(target_os = "linux")]
type VaList = *mut egui_video::ffmpeg_sys_the_third::__va_list_tag;
#[cfg(target_os = "android")]
type VaList = [u64; 4];

#[no_mangle]
pub unsafe extern "C" fn av_log_redirect(
    av_class: *mut libc::c_void,
    level: libc::c_int,
    fmt: *const libc::c_char,
    args: VaList,
) {
    use egui_video::ffmpeg_sys_the_third::*;
    let log_level = match level {
        AV_LOG_DEBUG => log::Level::Debug,
        AV_LOG_WARNING => log::Level::Debug, // downgrade to debug (spammy)
        AV_LOG_INFO => log::Level::Info,
        AV_LOG_ERROR => log::Level::Error,
        AV_LOG_PANIC => log::Level::Error,
        AV_LOG_FATAL => log::Level::Error,
        _ => log::Level::Trace,
    };
    let mut buf: [u8; 1024] = [0; 1024];
    let mut prefix: libc::c_int = 1;
    av_log_format_line(
        av_class,
        level,
        fmt,
        args,
        buf.as_mut_ptr() as *mut libc::c_char,
        1024,
        ptr::addr_of_mut!(prefix),
    );
    log!(target: "ffmpeg", log_level, "{}", CStr::from_ptr(buf.as_ptr() as *const libc::c_char).to_str().unwrap().trim());
}

#[cfg(target_os = "android")]
#[no_mangle]
#[tokio::main]
pub async fn android_main(app: AndroidApp) {
    android::start_android(app);
}
