use anyhow::Error;
use chrono::Utc;
use log::{error, info};
use nostr_sdk::prelude::StreamExt;
use nostr_sdk::Kind::Metadata;
use nostr_sdk::{Client, Filter, SubscribeAutoCloseOptions, SubscriptionId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait QueryClient {
    async fn subscribe(&self, id: &str, filters: &[QueryFilter]) -> Result<(), Error>;
}

pub type QueryFilter = Filter;

pub struct Query {
    pub id: String,
    queue: HashSet<QueryFilter>,
    traces: HashSet<QueryTrace>,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct QueryTrace {
    /// Subscription id on the relay
    pub id: Uuid,
    /// Filters associated with this subscription
    pub filters: Vec<QueryFilter>,
    /// When the query was created
    pub queued: u64,
    /// When the query was sent to the relay
    pub sent: Option<u64>,
    /// When EOSE was received
    pub eose: Option<u64>,
}

impl Query {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            queue: HashSet::new(),
            traces: HashSet::new(),
        }
    }

    /// Add filters to query
    pub fn add(&mut self, filter: Vec<QueryFilter>) {
        for f in filter {
            self.queue.insert(f);
        }
    }

    /// Return next query batch
    pub fn next(&mut self) -> Option<QueryTrace> {
        let mut next: Vec<QueryFilter> = self.queue.drain().collect();
        if next.is_empty() {
            return None;
        }
        let now = Utc::now();
        let id = Uuid::new_v4();

        // remove filters already sent
        next.retain(|f| {
            self.traces.is_empty() || !self.traces.iter().all(|y| y.filters.iter().any(|z| z == f))
        });

        // force profile queries into single filter
        if next.iter().all(|f| {
            if let Some(k) = &f.kinds {
                k.len() == 1 && k.first().unwrap().as_u16() == 0
            } else {
                false
            }
        }) {
            next = vec![Filter::new().kinds([Metadata]).authors(
                next.iter()
                    .flat_map(|f| f.authors.as_ref().unwrap().clone()),
            )]
        }

        if next.is_empty() {
            return None;
        }
        Some(QueryTrace {
            id,
            filters: next,
            queued: now.timestamp() as u64,
            sent: None,
            eose: None,
        })
    }
}

struct QueueDefer {
    id: String,
    filters: Vec<QueryFilter>,
}

pub struct QueryManager<C> {
    client: C,
    queries: Arc<RwLock<HashMap<String, Query>>>,
    queue_into_queries: UnboundedSender<QueueDefer>,
    sender: JoinHandle<()>,
}

impl<C> QueryManager<C>
where
    C: QueryClient + Clone + Send + Sync + 'static,
{
    pub(crate) fn new(client: C) -> Self {
        let queries = Arc::new(RwLock::new(HashMap::new()));
        let (tx, mut rx) = unbounded_channel::<QueueDefer>();
        Self {
            client: client.clone(),
            queries: queries.clone(),
            queue_into_queries: tx,
            sender: tokio::spawn(async move {
                loop {
                    {
                        let mut q = queries.write().await;
                        while let Ok(x) = rx.try_recv() {
                            Self::push_filters(&mut q, &x.id, x.filters);
                        }
                        for (k, v) in q.iter_mut() {
                            if let Some(qt) = v.next() {
                                info!("Sending trace: {:?}", qt);
                                match client
                                    .subscribe(&qt.id.to_string(), qt.filters.as_slice())
                                    .await
                                {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error!("Failed to subscribe to query filters: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }),
        }
    }

    pub async fn query<F>(&mut self, id: &str, filters: F)
    where
        F: Into<Vec<QueryFilter>>,
    {
        let mut qq = self.queries.write().await;
        Self::push_filters(&mut qq, id, filters.into());
    }

    fn push_filters(qq: &mut HashMap<String, Query>, id: &str, filters: Vec<QueryFilter>) {
        if let Some(q) = qq.get_mut(id) {
            q.add(filters);
        } else {
            let mut q = Query::new(id);
            q.add(filters);
            qq.insert(id.to_string(), q);
        }
    }

    pub fn queue_query<F>(&self, id: &str, filters: F)
    where
        F: Into<Vec<QueryFilter>>,
    {
        self.queue_into_queries
            .send(QueueDefer {
                id: id.to_string(),
                filters: filters.into(),
            })
            .unwrap()
    }
}

#[async_trait::async_trait]
impl QueryClient for Client {
    async fn subscribe(&self, id: &str, filters: &[QueryFilter]) -> Result<(), Error> {
        self.subscribe_with_id(
            SubscriptionId::new(id),
            filters.into(),
            Some(SubscribeAutoCloseOptions::default()),
        )
        .await?;
        Ok(())
    }
}
