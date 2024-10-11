use egui::CursorIcon::Default;
use log::{info, warn};
use nostr_sdk::secp256k1::Context;
use nostr_sdk::{Client, JsonUtil, Kind, RelayPoolNotification};
use nostrdb::{Error, Filter, Ndb, Note, NoteKey, ProfileRecord, QueryResult, Subscription, Transaction};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub struct NDBWrapper {
    ctx: egui::Context,
    ndb: Ndb,
    client: Client,
}

impl NDBWrapper {
    pub fn new(ctx: egui::Context, ndb: Ndb, client: Client) -> Self {
        let client_clone = client.clone();
        let ndb_clone = ndb.clone();
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut notifications = client_clone.notifications();
            while let Ok(e) = notifications.recv().await {
                match e {
                    RelayPoolNotification::Event { event, .. } => {
                        if let Err(e) = ndb_clone.process_event(event.as_json().as_str()) {
                            warn!("Failed to process event: {:?}", e);
                        } else {
                            ctx_clone.request_repaint();
                        }
                    }
                    _ => {
                        // dont care
                    }
                }
            }
        });
        Self { ctx, ndb, client }
    }

    pub fn start_transaction(&self) -> Transaction {
        Transaction::new(&self.ndb).unwrap()
    }

    pub fn subscribe(&self, filters: &[Filter]) -> Subscription {
        let sub = self.ndb.subscribe(filters).unwrap();
        let c_clone = self.client.clone();
        let filters = filters.iter().map(|f| nostr_sdk::Filter::from_json(f.json().unwrap()).unwrap()).collect();
        let id_clone = sub.id();
        tokio::spawn(async move {
            let nostr_sub = c_clone.subscribe(filters, None).await.unwrap();
            info!("Sub mapping {}->{}", id_clone, nostr_sub.id())
        });
        sub
    }

    pub fn unsubscribe(&self, sub: Subscription) {
        self.ndb.unsubscribe(sub).unwrap()
    }

    pub fn subscribe_with_results<'a>(&self, filters: &[Filter], tx: &'a Transaction, max_results: i32) -> (Subscription, Vec<QueryResult<'a>>) {
        let sub = self.subscribe(filters);
        let q = self.query(tx, filters, max_results);
        (sub, q)
    }


    pub fn query<'a>(&self, tx: &'a Transaction, filters: &[Filter], max_results: i32) -> Vec<QueryResult<'a>> {
        self.ndb.query(tx, filters, max_results).unwrap()
    }

    pub fn poll(&self, sub: Subscription, max_results: u32) -> Vec<NoteKey> {
        self.ndb.poll_for_notes(sub, max_results)
    }

    pub fn get_note_by_key<'a>(&self, tx: &'a Transaction, key: NoteKey) -> Result<Note<'a>, Error> {
        self.ndb.get_note_by_key(tx, key)
    }

    pub fn get_profile_by_pubkey<'a>(&self, tx: &'a Transaction, pubkey: &[u8; 32]) -> Result<ProfileRecord<'a>, Error> {
        self.ndb.get_profile_by_pubkey(tx, pubkey)
    }
}