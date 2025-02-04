use crate::note_util::NoteUtil;
use bech32::{Hrp, NoChecksum};
use nostr::prelude::{hex, Coordinate};
use nostr::{Kind, PublicKey};
use nostrdb::{Filter, NdbStrVariant, Note};
use std::fmt::{Display, Formatter};

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct NostrLink {
    pub hrp: NostrLinkType,
    pub id: IdOrStr,
    pub kind: Option<u32>,
    pub author: Option<[u8; 32]>,
    pub relays: Vec<String>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum IdOrStr {
    Id([u8; 32]),
    Str(String),
}

impl IdOrStr {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            IdOrStr::Id(i) => i,
            IdOrStr::Str(s) => s.as_bytes(),
        }
    }
}

impl Display for IdOrStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IdOrStr::Id(id) => write!(f, "{}", hex::encode(id)),
            IdOrStr::Str(str) => write!(f, "{}", str),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum NostrLinkType {
    Note,
    PublicKey,
    PrivateKey,

    // TLV kinds
    Event,
    Profile,
    Coordinate,
}

impl NostrLink {
    pub fn new(
        hrp: NostrLinkType,
        id: IdOrStr,
        kind: Option<u32>,
        author: Option<[u8; 32]>,
        relays: Vec<String>,
    ) -> Self {
        Self {
            hrp,
            id,
            kind,
            author,
            relays,
        }
    }

    pub fn from_note(note: &Note<'_>) -> Self {
        if note.kind() >= 30_000 && note.kind() < 40_000 {
            Self {
                hrp: NostrLinkType::Coordinate,
                id: IdOrStr::Str(
                    note.get_tag_value("d")
                        .map(|t| match t.variant() {
                            NdbStrVariant::Id(s) => hex::encode(s),
                            NdbStrVariant::Str(s) => s.to_owned(),
                        })
                        .unwrap_or(String::from("")),
                ),
                kind: Some(note.kind()),
                author: Some(*note.pubkey()),
                relays: vec![],
            }
        } else {
            Self {
                hrp: NostrLinkType::Event,
                id: IdOrStr::Id(*note.id()),
                kind: Some(note.kind()),
                author: Some(*note.pubkey()),
                relays: vec![],
            }
        }
    }

    pub fn profile(pubkey: &[u8; 32]) -> Self {
        Self {
            hrp: NostrLinkType::Profile,
            id: IdOrStr::Id(*pubkey),
            kind: None,
            author: None,
            relays: vec![],
        }
    }

    pub fn to_tag(&self) -> Vec<String> {
        if self.hrp == NostrLinkType::Coordinate {
            vec!["a".to_string(), self.to_tag_value()]
        } else {
            vec!["e".to_string(), self.to_tag_value()]
        }
    }

    pub fn to_tag_value(&self) -> String {
        if self.hrp == NostrLinkType::Coordinate {
            format!(
                "{}:{}:{}",
                self.kind.unwrap(),
                hex::encode(self.author.unwrap()),
                self.id
            )
        } else {
            self.id.to_string()
        }
    }
}

impl TryInto<Filter> for &NostrLink {
    type Error = ();
    fn try_into(self) -> Result<Filter, Self::Error> {
        match self.hrp {
            NostrLinkType::Coordinate => Ok(Filter::new()
                .kinds([self.kind.unwrap() as u64])
                .authors([&self.author.unwrap()])
                .tags(
                    [match self.id {
                        IdOrStr::Str(ref s) => s.to_owned(),
                        IdOrStr::Id(ref i) => hex::encode(i),
                    }],
                    'd',
                )
                .build()),
            NostrLinkType::Event | NostrLinkType::Note => Ok(Filter::new().build()),
            _ => Err(()),
        }
    }
}

impl Display for NostrLinkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Note => write!(f, "note"),
            Self::PublicKey => write!(f, "npub"),
            Self::PrivateKey => write!(f, "nsec"),
            Self::Event => write!(f, "nevent"),
            Self::Profile => write!(f, "nprofile"),
            Self::Coordinate => write!(f, "naddr"),
        }
    }
}

impl NostrLinkType {
    pub fn to_hrp(&self) -> Hrp {
        let str = self.to_string();
        Hrp::parse(str.as_str()).unwrap()
    }
}

impl Display for NostrLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.hrp {
            NostrLinkType::Note | NostrLinkType::PrivateKey | NostrLinkType::PublicKey => {
                Ok(bech32::encode_to_fmt::<NoChecksum, Formatter>(
                    f,
                    self.hrp.to_hrp(),
                    match &self.id {
                        IdOrStr::Str(s) => s.as_bytes(),
                        IdOrStr::Id(i) => i,
                    },
                )
                .map_err(|e| std::fmt::Error)?)
            }
            NostrLinkType::Event | NostrLinkType::Profile | NostrLinkType::Coordinate => todo!(),
        }
    }
}

impl TryInto<Coordinate> for NostrLink {
    type Error = ();

    fn try_into(self) -> Result<Coordinate, Self::Error> {
        match self.hrp {
            NostrLinkType::Coordinate => Ok(Coordinate::new(
                Kind::from_u16(self.kind.unwrap() as u16),
                PublicKey::from_slice(&self.author.unwrap()).unwrap(),
            )
            .identifier(self.id.to_string())),
            _ => Err(()),
        }
    }
}
