use crate::app::{NativeLayerOps, ZapStreamApp};
use crate::av_log_redirect;
use eframe::Renderer;
use egui::{Margin, ViewportBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::ops::Div;
use winit::platform::android::activity::AndroidApp;
use winit::platform::android::EventLoopBuilderExtAndroid;

pub fn start_android(app: AndroidApp) {
    std::env::set_var("RUST_BACKTRACE", "full");
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    unsafe {
        egui_video::ffmpeg_sys_the_third::av_log_set_callback(Some(av_log_redirect));
    }

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
        Box::new(move |cc| Ok(Box::new(ZapStreamApp::new(cc, data_path, app)))),
    ) {
        eprintln!("{}", e);
    }
}

impl NativeLayerOps for AndroidApp {
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

    fn show_keyboard(&self) {
        self.show_soft_input(true);
    }

    fn hide_keyboard(&self) {
        self.hide_soft_input(true);
    }

    fn get(&self, k: &str) -> Option<String> {
        None
    }

    fn set(&mut self, k: &str, v: &str) -> bool {
        false
    }

    fn remove(&mut self, k: &str) -> bool {
        false
    }

    fn get_obj<T: DeserializeOwned>(&self, k: &str) -> Option<T> {
        None
    }

    fn set_obj<T: Serialize>(&mut self, k: &str, v: &T) -> bool {
        false
    }
}
