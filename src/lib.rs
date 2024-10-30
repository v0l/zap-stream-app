pub mod app;
mod link;
mod note_store;
mod note_util;
mod route;
mod services;
mod stream_info;
mod theme;
mod widgets;

use crate::app::{AppConfig, ZapStreamApp};
use eframe::Renderer;
use egui::{Margin, ViewportBuilder};
use std::ops::{Div, Mul};
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;
#[cfg(target_os = "android")]
use winit::platform::android::EventLoopBuilderExtAndroid;

#[cfg(target_os = "android")]
#[no_mangle]
#[tokio::main]
pub async fn android_main(app: AndroidApp) {
    std::env::set_var("RUST_BACKTRACE", "full");
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let mut options = eframe::NativeOptions::default();
    options.renderer = Renderer::Glow;

    options.viewport = ViewportBuilder::default().with_fullscreen(true);

    let app_clone_for_event_loop = app.clone();
    options.event_loop_builder = Some(Box::new(move |builder| {
        builder.with_android_app(app_clone_for_event_loop);
    }));

    let data_path = app
        .external_data_path()
        .expect("external data path")
        .to_path_buf();

    let app = app.clone();
    let _res = eframe::run_native(
        "zap.stream",
        options,
        Box::new(move |cc| Ok(Box::new(ZapStreamApp::new(cc, data_path, app)))),
    );
}

#[cfg(target_os = "android")]
impl AppConfig for AndroidApp {
    fn frame_margin(&self) -> Margin {
        if let Some(wd) = self.native_window() {
            let (w, h) = (wd.width(), wd.height());
            let c_rect = self.content_rect();
            let dpi = self.config().density().unwrap_or(160);
            let dpi_scale = dpi as f32 / 160.0;
            // TODO: this calc is weird but seems to work on my phone
            Margin {
                bottom: (h - c_rect.bottom) as f32,
                left: c_rect.left as f32,
                right: (w - c_rect.right) as f32,
                top: (c_rect.top - (h - c_rect.bottom)) as f32,
            }
            .div(dpi_scale)
        } else {
            Margin::ZERO
        }
    }
}
