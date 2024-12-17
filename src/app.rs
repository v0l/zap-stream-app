use crate::route::Router;
use eframe::epaint::FontFamily;
use eframe::CreationContext;
use egui::{Color32, FontData, FontDefinitions, Margin};
use nostr_sdk::prelude::MemoryDatabase;
use nostr_sdk::Client;
use nostrdb::{Config, Ndb};
use notedeck::AppContext;
use std::path::PathBuf;

pub struct ZapStreamApp<T: NativeLayerOps> {
    client: Client,
    router: Router<T>,
    native_layer: T,
}

pub trait NativeLayerOps {
    /// Get any display layout margins
    fn frame_margin(&self) -> Margin;
    /// Show the keyboard on the screen
    fn show_keyboard(&self);
    /// Hide on screen keyboard
    fn hide_keyboard(&self);
    fn get(&self, k: &str) -> Option<String>;
    fn set(&mut self, k: &str, v: &str) -> bool;
    fn remove(&mut self, k: &str) -> bool;
    fn get_obj<T: serde::de::DeserializeOwned>(&self, k: &str) -> Option<T>;
    fn set_obj<T: serde::Serialize>(&mut self, k: &str, v: &T) -> bool;
}

impl<T> ZapStreamApp<T>
where
    T: NativeLayerOps + Clone,
{
    pub fn new(cc: &CreationContext, data_path: PathBuf, config: T) -> Self {
        let client = Client::builder()
            .database(MemoryDatabase::with_opts(Default::default()))
            .build();

        let client_clone = client.clone();
        tokio::spawn(async move {
            client_clone
                .add_relay("wss://nos.lol")
                .await
                .expect("Failed to add relay");
            client_clone
                .add_relay("wss://relay.damus.io")
                .await
                .expect("Failed to add relay");
            client_clone
                .add_relay("wss://relay.snort.social")
                .await
                .expect("Failed to add relay");
            client_clone.connect().await;
        });

        let ndb_path = data_path.join("ndb");
        std::fs::create_dir_all(&ndb_path).expect("Failed to create ndb directory");

        let mut ndb_config = Config::default();
        ndb_config.set_ingester_threads(4);

        let ndb = Ndb::new(ndb_path.to_str().unwrap(), &ndb_config).unwrap();

        let mut fd = FontDefinitions::default();
        fd.font_data.insert(
            "Outfit".to_string(),
            FontData::from_static(include_bytes!("../assets/Outfit-Light.ttf")),
        );
        fd.families
            .insert(FontFamily::Proportional, vec!["Outfit".to_string()]);
        cc.egui_ctx.set_fonts(fd);

        let cfg = config.clone();
        Self {
            client: client.clone(),
            router: Router::new(
                data_path,
                cc.egui_ctx.clone(),
                client.clone(),
                ndb.clone(),
                cfg,
            ),
            native_layer: config,
        }
    }
}

#[cfg(not(feature = "notedeck"))]
impl<T> App for ZapStreamApp<T>
where
    T: NativeLayerOps,
{
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let mut app_frame = egui::containers::Frame::default();
        let margin = self.native_layer.frame_margin();

        app_frame.inner_margin = margin;
        app_frame.stroke.color = Color32::BLACK;

        //ctx.set_debug_on_hover(true);

        egui::CentralPanel::default()
            .frame(app_frame)
            .show(ctx, |ui| {
                ui.visuals_mut().override_text_color = Some(Color32::WHITE);
                self.router.show(ui);
            });
    }
}

#[cfg(feature = "notedeck")]
impl<T> notedeck::App for ZapStreamApp<T>
where
    T: NativeLayerOps,
{
    fn update(&mut self, ctx: &mut AppContext<'_>) {
        let mut app_frame = egui::containers::Frame::default();
        let margin = self.native_layer.frame_margin();

        app_frame.inner_margin = margin;
        app_frame.stroke.color = Color32::BLACK;

        //ctx.set_debug_on_hover(true);

        egui::CentralPanel::default()
            .frame(app_frame)
            .show(ctx.egui, |ui| {
                ui.visuals_mut().override_text_color = Some(Color32::WHITE);
                self.router.show(ui);
            });
    }
}
