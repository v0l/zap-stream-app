use crate::note_util::NoteUtil;
use nostrdb::{NdbStrVariant, Note};

pub trait StreamInfo {
    fn title(&self) -> Option<&str>;

    fn summary(&self) -> Option<&str>;

    fn host(&self) -> &[u8; 32];

    fn stream(&self) -> Option<&str>;

    fn starts(&self) -> u64;
}

impl<'a> StreamInfo for Note<'a> {
    fn title(&self) -> Option<&str> {
        if let Some(s) = self.get_tag_value("title") {
            s.variant().str()
        } else {
            None
        }
    }

    fn summary(&self) -> Option<&str> {
        if let Some(s) = self.get_tag_value("summary") {
            s.variant().str()
        } else {
            None
        }
    }

    fn host(&self) -> &[u8; 32] {
        match self.find_tag_value(|t| {
            t[0].variant().str() == Some("p") && t[3].variant().str() == Some("host")
        }) {
            Some(t) => match t.variant() {
                NdbStrVariant::Id(i) => i,
                NdbStrVariant::Str(s) => self.pubkey(),
            },
            None => self.pubkey(),
        }
    }

    fn stream(&self) -> Option<&str> {
        if let Some(s) = self.get_tag_value("streaming") {
            s.variant().str()
        } else {
            None
        }
    }

    fn starts(&self) -> u64 {
        if let Some(s) = self.get_tag_value("starts") {
            s.variant().str()
                .map_or(self.created_at(), |v| v.parse::<u64>().unwrap_or(self.created_at()))
        } else {
            self.created_at()
        }
    }
}
