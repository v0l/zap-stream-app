use egui::load::BytesLoader;
use log::{info, warn};
use nostr_sdk::prelude::hex;
use nostr_sdk::{Client, Metadata, PublicKey};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;

pub struct ProfileService {
    client: Client,
    pub profiles: Arc<Mutex<HashMap<[u8; 32], Option<Metadata>>>>,
    ctx: egui::Context,
    request_profile: UnboundedSender<[u8; 32]>,
}

struct Loader {
    client: Client,
    ctx: egui::Context,
    profiles: Arc<Mutex<HashMap<[u8; 32], Option<Metadata>>>>,
    queue: UnboundedReceiver<[u8; 32]>,
}

impl Loader {
    pub async fn run(&mut self) {
        while let Some(key) = self.queue.recv().await {
            let mut profiles = self.profiles.lock().await;
            if !profiles.contains_key(&key) {
                info!("Requesting profile {}", hex::encode(key));
                match self.client.fetch_metadata(PublicKey::from_slice(&key).unwrap(),
                                                 Some(Duration::from_secs(3))).await {
                    Ok(meta) => {
                        profiles.insert(key, Some(meta));
                        self.ctx.request_repaint();
                    }
                    Err(e) => {
                        warn!("Error getting metadata: {}", e);
                    }
                }
            }
        }
    }
}

impl ProfileService {
    pub fn new(client: Client, ctx: egui::Context) -> ProfileService
    {
        let profiles = Arc::new(Mutex::new(HashMap::new()));
        let (tx, rx) = unbounded_channel();
        let mut loader = Loader {
            client: client.clone(),
            ctx: ctx.clone(),
            profiles: profiles.clone(),
            queue: rx,
        };

        tokio::spawn(async move {
            loader.run().await;
        });

        Self {
            client,
            ctx,
            profiles,
            request_profile: tx,
        }
    }

    pub fn get_profile(&self, public_key: &[u8; 32]) -> Option<Metadata> {
        if let Ok(profiles) = self.profiles.try_lock() {
            return if let Some(p) = profiles.get(public_key) {
                if let Some(p) = p {
                    Some(p.clone())
                } else {
                    None
                }
            } else {
                self.request_profile.send(*public_key).expect("Failed request");
                None
            };
        }
        None
    }
}