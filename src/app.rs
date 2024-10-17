use crate::route::Router;
use eframe::{App, CreationContext, Frame};
use egui::{Color32, Context};
use nostr_sdk::database::MemoryDatabase;
use nostr_sdk::{Client, RelayPoolNotification};
use nostrdb::{Config, Ndb};
use std::path::PathBuf;
use tokio::sync::broadcast;

pub struct ZapStreamApp {
    client: Client,
    notifications: broadcast::Receiver<RelayPoolNotification>,
    router: Router,
}

impl ZapStreamApp {
    pub fn new(cc: &CreationContext, data_path: PathBuf) -> Self {
        let client = Client::builder()
            .database(MemoryDatabase::with_opts(Default::default()))
            .build();
        let notifications = client.notifications();

        let ctx_clone = cc.egui_ctx.clone();
        let client_clone = client.clone();
        tokio::spawn(async move {
            client_clone
                .add_relay("wss://nos.lol")
                .await
                .expect("Failed to add relay");
            client_clone.connect().await;
            let mut notifications = client_clone.notifications();
            while let Ok(_) = notifications.recv().await {
                ctx_clone.request_repaint();
            }
        });
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let ndb_path = data_path.join("ndb");
        std::fs::create_dir_all(&ndb_path).expect("Failed to create ndb directory");

        let ndb = Ndb::new(ndb_path.to_str().unwrap(), &Config::default()).unwrap();

        Self {
            client: client.clone(),
            notifications,
            router: Router::new(data_path, cc.egui_ctx.clone(), client.clone(), ndb.clone()),
        }
    }
}

impl App for ZapStreamApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let mut app_frame = egui::containers::Frame::default();
        app_frame.stroke.color = Color32::BLACK;

        //ctx.set_debug_on_hover(true);

        egui::CentralPanel::default()
            .frame(app_frame)
            .show(ctx, |ui| {
                self.router.show(ui);
            });
    }
}
