use crate::note_util::NoteUtil;
use nostrdb::Note;

pub trait StreamInfo {
    fn title(&self) -> Option<String>;

    fn summary(&self) -> Option<String>;

    fn host(&self) -> [u8; 32];

    fn stream(&self) -> Option<String>;
}

impl<'a> StreamInfo for Note<'a> {
    fn title(&self) -> Option<String> {
        if let Some(s) = self.get_tag_value("title") {
            s.variant().str().map(ToString::to_string)
        } else {
            None
        }
    }

    fn summary(&self) -> Option<String> {
        todo!()
    }

    fn host(&self) -> [u8; 32] {
        todo!()
    }

    fn stream(&self) -> Option<String> {
        if let Some(s) = self.get_tag_value("streaming") {
            s.variant().str().map(ToString::to_string)
        } else {
            None
        }
    }
}