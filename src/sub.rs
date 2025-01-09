use log::info;
use nostrdb::{Ndb, Subscription};

pub struct SubRef {
    pub sub: Subscription,
    ndb: Ndb,
}

impl SubRef {
    pub fn new(sub: Subscription, ndb: Ndb) -> Self {
        info!("Creating sub: {}", sub.id());
        SubRef { sub, ndb }
    }
}

impl Drop for SubRef {
    fn drop(&mut self) {
        self.ndb.unsubscribe(self.sub).expect("unsubscribe failed");
        info!("Closing sub: {}", self.sub.id());
    }
}
