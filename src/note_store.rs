use crate::link::NostrLink;
use nostrdb::Note;
use std::collections::HashMap;

pub struct NoteStore<'a> {
    events: HashMap<String, Note<'a>>,
}

impl<'a> NoteStore<'a> {
    pub fn new() -> Self {
        Self {
            events: HashMap::new()
        }
    }

    pub fn from_vec(events: Vec<Note<'a>>) -> Self {
        let mut store = Self::new();
        for note in events {
            store.add(note);
        }
        store
    }

    pub fn add(&mut self, note: Note<'a>) -> Option<Note<'a>> {
        let k = Self::key(&note);
        if let Some(v) = self.events.get(&k) {
            if v.created_at() < note.created_at() {
                return self.events.insert(k, note);
            }
        }
        self.events.insert(k, note)
    }

    pub fn remove(&mut self, note: &Note<'a>) -> Option<Note<'a>> {
        self.events.remove(&Self::key(note))
    }

    pub fn key(note: &Note<'a>) -> String {
        NostrLink::from_note(note)
            .to_tag_value()
    }

    pub fn iter(&self) -> impl Iterator<Item=&Note<'a>> {
        self.events.values()
    }
}

