use nostrdb::{Note, NoteKey, QueryResult};
use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub struct NoteRef {
    pub key: NoteKey,
    pub created_at: u64,
}

impl NoteRef {
    pub fn new(key: NoteKey, created_at: u64) -> Self {
        NoteRef { key, created_at }
    }

    pub fn from_note(note: &Note<'_>) -> Self {
        let created_at = note.created_at();
        let key = note.key().expect("todo: implement NoteBuf");
        NoteRef::new(key, created_at)
    }

    pub fn from_query_result(qr: QueryResult<'_>) -> Self {
        NoteRef {
            key: qr.note_key,
            created_at: qr.note.created_at(),
        }
    }
}

impl Ord for NoteRef {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.created_at.cmp(&other.created_at) {
            Ordering::Equal => self.key.cmp(&other.key),
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
        }
    }
}

impl PartialOrd for NoteRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
