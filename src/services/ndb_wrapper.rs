use crate::services::query::QueryManager;
use log::warn;
use nostr_sdk::{nostr, Client, JsonUtil, Kind, PublicKey, RelayPoolNotification};
use nostrdb::{
    Error, Filter, Ndb, NdbProfile, Note, NoteKey, ProfileRecord, QueryResult, Subscription,
    Transaction,
};
use std::collections::HashSet;
use std::sync::Mutex;

pub struct NDBWrapper {
    ctx: egui::Context,
    ndb: Ndb,
    client: Client,
    query_manager: QueryManager<Client>,
    profiles: Mutex<HashSet<[u8; 32]>>,
}

/// Automatic cleanup for subscriptions
pub struct SubWrapper {
    ndb: Ndb,
    subscription: Subscription,
}

impl SubWrapper {
    pub fn new(ndb: Ndb, subscription: Subscription) -> Self {
        Self { ndb, subscription }
    }
}

impl From<&SubWrapper> for u64 {
    fn from(val: &SubWrapper) -> Self {
        val.subscription.id()
    }
}

impl Drop for SubWrapper {
    fn drop(&mut self) {
        self.ndb.unsubscribe(self.subscription).unwrap()
    }
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
        let qm = QueryManager::new(client.clone());

        Self {
            ctx,
            ndb,
            client,
            query_manager: qm,
            profiles: Mutex::new(HashSet::new()),
        }
    }

    pub fn start_transaction(&self) -> Transaction {
        Transaction::new(&self.ndb).unwrap()
    }

    pub fn subscribe(&self, id: &str, filters: &[Filter]) -> SubWrapper {
        let sub = self.ndb.subscribe(filters).unwrap();
        // very lazy conversion
        let filters: Vec<nostr_sdk::Filter> = filters
            .iter()
            .map(|f| nostr_sdk::Filter::from_json(f.json().unwrap()).unwrap())
            .collect();
        self.query_manager.queue_query(id, filters);
        SubWrapper::new(self.ndb.clone(), sub)
    }

    pub fn unsubscribe(&self, sub: &SubWrapper) {
        self.ndb.unsubscribe(sub.subscription).unwrap()
    }

    pub fn subscribe_with_results<'a>(
        &self,
        id: &str,
        filters: &[Filter],
        tx: &'a Transaction,
        max_results: i32,
    ) -> (SubWrapper, Vec<QueryResult<'a>>) {
        let sub = self.subscribe(id, filters);
        let q = self.query(tx, filters, max_results);
        (sub, q)
    }

    pub fn query<'a>(
        &self,
        tx: &'a Transaction,
        filters: &[Filter],
        max_results: i32,
    ) -> Vec<QueryResult<'a>> {
        self.ndb.query(tx, filters, max_results).unwrap()
    }

    pub fn poll(&self, sub: &SubWrapper, max_results: u32) -> Vec<NoteKey> {
        self.ndb.poll_for_notes(sub.subscription, max_results)
    }

    pub fn get_note_by_key<'a>(
        &self,
        tx: &'a Transaction,
        key: NoteKey,
    ) -> Result<Note<'a>, Error> {
        self.ndb.get_note_by_key(tx, key)
    }

    pub fn get_profile_by_pubkey<'a>(
        &self,
        tx: &'a Transaction,
        pubkey: &[u8; 32],
    ) -> Result<ProfileRecord<'a>, Error> {
        self.ndb.get_profile_by_pubkey(tx, pubkey)
    }

    pub fn fetch_profile<'a>(
        &self,
        tx: &'a Transaction,
        pubkey: &[u8; 32],
    ) -> (Option<NdbProfile<'a>>, Option<SubWrapper>) {
        let p = self
            .get_profile_by_pubkey(tx, pubkey)
            .map_or(None, |p| p.record().profile());

        // TODO: fix this shit
        if p.is_none() && self.profiles.lock().unwrap().insert(*pubkey) {
            self.query_manager.queue_query(
                "profile",
                &[nostr::Filter::new()
                    .kinds([Kind::Metadata])
                    .authors([PublicKey::from_slice(pubkey).unwrap()])],
            )
        }
        let sub = None;
        (p, sub)
    }

    pub fn submit_event(&self, ev: &str) -> Result<(), Error> {
        self.ndb.process_event(ev)
    }
}
