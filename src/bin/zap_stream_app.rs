use egui::{Margin, Vec2, ViewportBuilder};
use nostr_sdk::serde_json;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use zap_stream_app::app::{NativeLayerOps, ZapStreamApp};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut options = eframe::NativeOptions::default();
    options.viewport = ViewportBuilder::default().with_inner_size(Vec2::new(1300., 900.));

    let data_path = PathBuf::from("./.data");
    let config = DesktopApp::new(data_path.clone());
    let _res = eframe::run_native(
        "zap.stream",
        options,
        Box::new(move |cc| Ok(Box::new(ZapStreamApp::new(cc, data_path, config)))),
    );
}

#[derive(Clone)]
pub struct DesktopApp {
    data_path: PathBuf,
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl DesktopApp {
    pub fn new(data_path: PathBuf) -> Self {
        let mut r = Self {
            data_path,
            data: Arc::new(RwLock::new(HashMap::new())),
        };
        r.load();
        r
    }

    fn storage_file_path(&self) -> PathBuf {
        self.data_path.join("kv.json")
    }

    fn load(&mut self) {
        let path = self.storage_file_path();
        if path.exists() {
            let mut file = std::fs::File::open(path).unwrap();
            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();
            if let Ok(d) = serde_json::from_slice(data.as_slice()) {
                self.data = Arc::new(RwLock::new(d));
            }
        }
    }

    fn save(&self) {
        let path = self.storage_file_path();
        let mut file = std::fs::File::create(path).unwrap();
        let json = serde_json::to_string_pretty(self.data.read().unwrap().deref()).unwrap();
        file.write_all(json.as_bytes()).unwrap();
    }
}

impl NativeLayerOps for DesktopApp {
    fn frame_margin(&self) -> Margin {
        Margin::ZERO
    }

    fn show_keyboard(&self) {
        // nothing to do
    }

    fn hide_keyboard(&self) {
        // nothing to do
    }
    fn get(&self, k: &str) -> Option<String> {
        self.data.read().unwrap().get(k).cloned()
    }

    fn set(&mut self, k: &str, v: &str) -> bool {
        self.data
            .write()
            .unwrap()
            .insert(k.to_owned(), v.to_owned())
            .is_none()
    }

    fn remove(&mut self, k: &str) -> bool {
        self.data.write().unwrap().remove(k).is_some()
    }

    fn get_obj<T: DeserializeOwned>(&self, k: &str) -> Option<T> {
        serde_json::from_str(self.get(k)?.as_str()).ok()
    }

    fn set_obj<T: Serialize>(&mut self, k: &str, v: &T) -> bool {
        self.set(k, serde_json::to_string(v).unwrap().as_str())
    }
}

impl Drop for DesktopApp {
    fn drop(&mut self) {
        self.save();
    }
}
