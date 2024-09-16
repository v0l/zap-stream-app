use nostr_sdk::Filter;

pub trait NostrWidget {
    fn subscribe(&self) -> Vec<Filter>;
}