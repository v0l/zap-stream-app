use crate::services::profile::ProfileService;
use egui::Context;
use nostr_sdk::Client;

pub mod profile;

pub struct Services {
    pub context: Context,
    pub profile: ProfileService,
}

impl Services {
    pub fn new(client: Client, context: Context) -> Self {
        Self {
            context: context.clone(),
            profile: ProfileService::new(client.clone(), context.clone()),
        }
    }
}