use crate::services::Services;
use crate::widget::NostrWidget;
use crate::widgets::header::Header;
use crate::widgets::stream_list::StreamList;
use eframe::{App, CreationContext, Frame};
use egui::{Color32, Context, ScrollArea};
use nostr_sdk::database::{MemoryDatabase, MemoryDatabaseOptions};
use nostr_sdk::{Client, Event, Filter, Kind, RelayPoolNotification};
use nostrdb::{Config, Ndb, Transaction};
use tokio::sync::broadcast;

pub struct ZapStreamApp {
    ndb: Ndb,
    client: Client,
    notifications: broadcast::Receiver<RelayPoolNotification>,
    events: Vec<Event>,
    services: Services,
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

        Self {
            ndb: Ndb::new(".", &Config::default()).unwrap(),
            client: client.clone(),
            notifications,
            services: Services::new(client, cc.egui_ctx.clone()),
            events: vec![],
        }
    }

    fn process_nostr(&mut self) {
        while let Ok(msg) = self.notifications.try_recv() {
            match msg {
                RelayPoolNotification::Event { event, .. } => {
                    self.events.push(*event);
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
        let txn = Transaction::new(&self.ndb).expect("txn");
        self.process_nostr();

        let mut app_frame = egui::containers::Frame::default();
        app_frame.stroke.color = Color32::BLACK;

        ctx.set_debug_on_hover(true);
        egui::CentralPanel::default()
            .frame(app_frame)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.add(Header::new(&self.services));
                    ui.add(StreamList::new(&self.events, &self.services));
                });
            });
    }
}