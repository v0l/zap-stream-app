use crate::route::Router;
use eframe::{App, CreationContext, Frame};
use egui::{Color32, Context};
use nostr_sdk::database::MemoryDatabase;
use nostr_sdk::Client;
use nostrdb::{Config, Ndb};
use std::path::PathBuf;

pub struct ZapStreamApp {
    client: Client,
    router: Router,
}

impl ZapStreamApp {
    pub fn new(cc: &CreationContext, data_path: PathBuf) -> Self {
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
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let ndb_path = data_path.join("ndb");
        std::fs::create_dir_all(&ndb_path).expect("Failed to create ndb directory");

        let mut ndb_config = Config::default();
        ndb_config.set_ingester_threads(4);

        let ndb = Ndb::new(ndb_path.to_str().unwrap(), &ndb_config).unwrap();

        Self {
            client: client.clone(),
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
