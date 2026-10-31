pub const DEFAULT_RELAYS: &[&str] = &[
    "wss://relay.zap.stream",
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.primal.net",
];

pub fn args(mut base: Vec<String>) -> Vec<String> {
    if !base.iter().any(|a| a == "-r" || a == "--relay") {
        for r in DEFAULT_RELAYS {
            base.push("-r".to_string());
            base.push((*r).to_string());
        }
    }
    base
}
