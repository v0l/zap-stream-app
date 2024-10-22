use crate::link::NostrLink;
use crate::note_util::OwnedNote;
use crate::route::RouteServices;
use crate::services::ndb_wrapper::{NDBWrapper, SubWrapper};
use crate::stream_info::StreamInfo;
use crate::widgets::{Chat, NostrWidget, StreamPlayer, StreamTitle, WriteChat};
use egui::{Response, Ui, Vec2, Widget};
use nostrdb::{Filter, NoteKey, Transaction};
use std::borrow::Borrow;

pub struct StreamPage {
    link: NostrLink,
    event: Option<OwnedNote>,
    player: Option<StreamPlayer>,
    chat: Option<Chat>,
    sub: SubWrapper,
    new_msg: WriteChat,
}

impl StreamPage {
    pub fn new_from_link(ndb: &NDBWrapper, tx: &Transaction, link: NostrLink) -> Self {
        let f: Filter = link.borrow().try_into().unwrap();
        let f = [f.limit_mut(1)];
        let (sub, events) = ndb.subscribe_with_results("streams", &f, tx, 1);
        Self {
            link,
            sub,
            event: events.first().map(|n| OwnedNote(n.note_key.as_u64())),
            chat: None,
            player: None,
            new_msg: WriteChat::new(),
        }
    }
}

impl NostrWidget for StreamPage {
    fn render(&mut self, ui: &mut Ui, services: &RouteServices<'_>) -> Response {
        let poll = services.ndb.poll(&self.sub, 1);
        if let Some(k) = poll.first() {
            self.event = Some(OwnedNote(k.as_u64()))
        }

        let event = if let Some(k) = &self.event {
            services
                .ndb
                .get_note_by_key(services.tx, NoteKey::new(k.0))
                .ok()
        } else {
            None
        };
        if let Some(event) = event {
            if let Some(stream) = event.stream() {
                if self.player.is_none() {
                    let p = StreamPlayer::new(ui.ctx(), &stream.to_string());
                    self.player = Some(p);
                }
            }

            if let Some(player) = &mut self.player {
                player.ui(ui);
            }
            StreamTitle::new(&event).render(ui, services);

            if self.chat.is_none() {
                let ok = OwnedNote(event.key().unwrap().as_u64());
                let chat = Chat::new(self.link.clone(), ok, services.ndb, services.tx);
                self.chat = Some(chat);
            }

            let chat_h = 60.0;
            let w = ui.available_width();
            let h = ui.available_height();
            ui.allocate_ui(Vec2::new(w, h - chat_h), |ui| {
                if let Some(c) = self.chat.as_mut() {
                    c.render(ui, services);
                } else {
                    ui.label("Loading..");
                }
                // consume rest of space
                ui.add_space(ui.available_height());
            });
            ui.allocate_ui(Vec2::new(w, chat_h), |ui| self.new_msg.render(ui, services))
                .response
        } else {
            ui.label("Loading..")
        }
    }
}
