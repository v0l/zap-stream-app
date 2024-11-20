use crate::app::NativeLayerOps;
use crate::link::NostrLink;
use anyhow::Error;
use nostr_sdk::secp256k1::{Keypair, XOnlyPublicKey};
use nostr_sdk::{Event, EventBuilder, Keys, Kind, SecretKey, Tag, UnsignedEvent};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::ops::Deref;

#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LoginKind {
    PublicKey {
        #[serde_as(as = "serde_with::hex::Hex")]
        key: [u8; 32],
    },
    PrivateKey {
        #[serde_as(as = "serde_with::hex::Hex")]
        key: [u8; 32],
    },
    LoggedOut,
}

pub struct Login {
    kind: LoginKind,
}

impl Login {
    pub fn new() -> Self {
        Self {
            kind: LoginKind::LoggedOut,
        }
    }

    pub fn load<T: NativeLayerOps>(&mut self, storage: &T) {
        if let Some(k) = storage.get_obj("login") {
            self.kind = k;
        }
    }

    pub fn save<T: NativeLayerOps>(&mut self, storage: &mut T) {
        storage.set_obj("login", &self.kind);
    }

    pub fn login(&mut self, kind: LoginKind) {
        self.kind = kind;
    }

    pub fn is_logged_in(&self) -> bool {
        !matches!(self.kind, LoginKind::LoggedOut)
    }

    pub fn public_key(&self) -> Option<[u8; 32]> {
        match self.kind {
            LoginKind::PublicKey { key } => Some(key),
            LoginKind::PrivateKey { key } => {
                // TODO: wow this is annoying
                let sk = Keypair::from_seckey_slice(nostr_sdk::SECP256K1.deref(), key.as_slice())
                    .unwrap();
                Some(XOnlyPublicKey::from_keypair(&sk).0.serialize())
            }
            _ => None,
        }
    }

    fn secret_key(&self) -> Result<Keys, Error> {
        if let LoginKind::PrivateKey { key } = self.kind {
            Ok(Keys::new(SecretKey::from_slice(key.as_slice())?))
        } else {
            anyhow::bail!("No private key");
        }
    }

    pub fn sign_event(&self, ev: UnsignedEvent) -> Result<Event, Error> {
        let secret = self.secret_key()?;
        ev.sign_with_keys(&secret).map_err(Error::new)
    }

    pub fn write_live_chat_msg(&self, link: &NostrLink, msg: &str) -> Result<Event, Error> {
        if msg.len() == 0 {
            return Err(anyhow::anyhow!("Empty message"));
        }
        let secret = self.secret_key()?;
        EventBuilder::new(Kind::LiveEventMessage, msg, [Tag::parse(&link.to_tag())?])
            .to_event(&secret)
            .map_err(Error::new)
    }
}
