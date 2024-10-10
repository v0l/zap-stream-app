use crate::route::Router;
use eframe::{App, CreationContext, Frame};
use egui::{Color32, Context, ScrollArea};
use egui_inbox::UiInbox;
use log::warn;
use nostr_sdk::database::{MemoryDatabase, MemoryDatabaseOptions};
use nostr_sdk::{Client, Filter, JsonUtil, Kind, RelayPoolNotification};
use nostrdb::{Config, Ndb};
use tokio::sync::broadcast;

pub struct ZapStreamApp {
    ndb: Ndb,
    client: Client,
    notifications: broadcast::Receiver<RelayPoolNotification>,
    router: Router,
}

impl ZapStreamApp {
    pub fn new(cc: &CreationContext) -> Self {
        let client = Client::builder().database(MemoryDatabase::with_opts(MemoryDatabaseOptions {
            events: true,
            ..Default::default()
        })).build();
        let notifications = client.notifications();

        let ctx_clone = cc.egui_ctx.clone();
        let client_clone = client.clone();
        tokio::spawn(async move {
            client_clone.add_relay("wss://nos.lol").await.expect("Failed to add relay");
            client_clone.connect().await;
            client_clone.subscribe(vec![
                Filter::new()
                    .kind(Kind::LiveEvent)
            ], None).await.expect("Failed to subscribe");
            let mut notifications = client_clone.notifications();
            while let Ok(_) = notifications.recv().await {
                ctx_clone.request_repaint();
            }
        });
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let inbox = UiInbox::new();
        let ndb = Ndb::new(".", &Config::default()).unwrap();
        Self {
            ndb: ndb.clone(),
            client: client.clone(),
            notifications,
            router: Router::new(inbox, cc.egui_ctx.clone(), client.clone(), ndb.clone()),
        }
    }

    fn process_nostr(&mut self) {
        while let Ok(msg) = self.notifications.try_recv() {
            match msg {
                RelayPoolNotification::Event { event, .. } => {
                    if let Err(e) = self.ndb.process_event(event.as_json().as_str()) {
                        warn!("Failed to process event: {:?}", e);
                    }
                }
                _ => {
                    // dont care
                }
            }
        }
    }
}

impl App for ZapStreamApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        self.process_nostr();

        let mut app_frame = egui::containers::Frame::default();
        app_frame.stroke.color = Color32::BLACK;

        //ctx.set_debug_on_hover(true);

        egui::CentralPanel::default()
            .frame(app_frame)
            .show(ctx, |ui| {
                self.router.show(ui)
            });
    }
}
