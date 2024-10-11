use nostr_sdk::util::hex;
use nostrdb::{NdbStr, Note, Tag};
use std::fmt::Display;

pub trait NoteUtil {
    fn id_hex(&self) -> String;
    fn get_tag_value(&self, key: &str) -> Option<NdbStr>;
    fn find_tag_value<F>(&self, fx: F) -> Option<NdbStr>
    where
        F: Fn(Vec<NdbStr>) -> bool;
}

impl<'a> NoteUtil for Note<'a> {
    fn id_hex(&self) -> String {
        hex::encode(self.id())
    }

    fn get_tag_value(&self, key: &str) -> Option<NdbStr> {
        self.find_tag_value(|t| t[0].variant().str() == Some(key))
    }

    fn find_tag_value<F>(&self, fx: F) -> Option<NdbStr>
    where
        F: Fn(Vec<NdbStr>) -> bool,
    {
        let tag = self.tags().iter().find(|t| {
            let tag_vec = TagIterBorrow::new(t).collect();
            fx(tag_vec)
        });
        if let Some(t) = tag {
            t.get(1)
        } else {
            None
        }
    }
}


#[derive(Debug, Clone)]
pub struct TagIterBorrow<'a> {
    tag: &'a Tag<'a>,
    index: u16,
}

impl<'a> TagIterBorrow<'a> {
    pub fn new(tag: &'a Tag<'a>) -> Self {
        let index = 0;
        TagIterBorrow { tag, index }
    }

    pub fn done(&self) -> bool {
        self.index >= self.tag.count()
    }
}

impl<'a> Iterator for TagIterBorrow<'a> {
    type Item = NdbStr<'a>;

    fn next(&mut self) -> Option<NdbStr<'a>> {
        let tag = self.tag.get(self.index);
        if tag.is_some() {
            self.index += 1;
            tag
        } else {
            None
        }
    }
}

#[derive(Eq, PartialEq)]
pub struct OwnedNote(pub u64);