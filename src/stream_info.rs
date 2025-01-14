use crate::note_util::NoteUtil;
use nostrdb::{NdbStrVariant, Note};
use std::fmt::{Display, Formatter};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum StreamStatus {
    Live,
    Ended,
    Planned,
}

impl Display for StreamStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamStatus::Live => write!(f, "Live"),
            StreamStatus::Ended => write!(f, "Ended"),
            StreamStatus::Planned => write!(f, "Planned"),
        }
    }
}

pub trait StreamInfo {
    fn title(&self) -> Option<&str>;

    fn summary(&self) -> Option<&str>;

    fn host(&self) -> &[u8; 32];

    fn streaming(&self) -> Option<&str>;

    fn recording(&self) -> Option<&str>;

    /// Is the stream playable by this app
    fn can_play(&self) -> bool;

    fn starts(&self) -> u64;

    fn image(&self) -> Option<&str>;

    fn status(&self) -> StreamStatus;

    fn viewers(&self) -> Option<u32>;
}

impl StreamInfo for Note<'_> {
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

    fn streaming(&self) -> Option<&str> {
        if let Some(s) = self.get_tag_value("streaming") {
            s.variant().str()
        } else {
            None
        }
    }

    fn recording(&self) -> Option<&str> {
        if let Some(s) = self.get_tag_value("recording") {
            s.variant().str()
        } else {
            None
        }
    }

    /// Is the stream playable by this app
    fn can_play(&self) -> bool {
        if self.kind() == 30_313 {
            return true; // n94-stream can always be played
        }
        if let Some(stream) = self.streaming() {
            stream.contains(".m3u8")
        } else {
            false
        }
    }

    fn starts(&self) -> u64 {
        if let Some(s) = self.get_tag_value("starts") {
            s.variant().str().map_or(self.created_at(), |v| {
                v.parse::<u64>().unwrap_or(self.created_at())
            })
        } else {
            self.created_at()
        }
    }

    fn image(&self) -> Option<&str> {
        if let Some(s) = self.get_tag_value("image") {
            s.variant().str()
        } else {
            None
        }
    }

    fn status(&self) -> StreamStatus {
        if let Some(s) = self.get_tag_value("status") {
            match s.variant().str() {
                Some("live") => StreamStatus::Live,
                Some("planned") => StreamStatus::Planned,
                _ => StreamStatus::Ended,
            }
        } else {
            StreamStatus::Ended
        }
    }

    fn viewers(&self) -> Option<u32> {
        if let Some(s) = self.get_tag_value("current_participants") {
            s.variant().str().map(|v| v.parse::<u32>().unwrap_or(0))
        } else {
            None
        }
    }
}
