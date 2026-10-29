use log::info;
use nostrdb::{Ndb, Subscription};
use notedeck::SubOwnerKey;
use std::sync::Mutex;

static RELEASED: Mutex<Vec<SubOwnerKey>> = Mutex::new(Vec::new());

pub fn take_released() -> Vec<SubOwnerKey> {
    std::mem::take(&mut *RELEASED.lock().unwrap())
}

pub struct SubRef {
    pub sub: Subscription,
    owner: SubOwnerKey,
    ndb: Ndb,
}

impl SubRef {
    pub fn new(
        sub: Subscription,
        ndb: Ndb,
        owner: SubOwnerKey,
    ) -> Self {
        info!("Creating sub: {}", sub.id());
        SubRef {
            sub,
            owner,
            ndb,
        }
    }
}

impl Drop for SubRef {
    fn drop(&mut self) {
        let _ = self.ndb.unsubscribe(self.sub);
        RELEASED.lock().unwrap().push(self.owner);
        info!("Closing sub: {}", self.sub.id());
    }
}
