use crate::link::NostrLink;
use crate::note_util::{NoteUtil, OwnedNote};
use crate::route::RouteServices;
use crate::services::ndb_wrapper::{NDBWrapper, SubWrapper};
use crate::stream_info::StreamInfo;
use crate::widgets::{Chat, NostrWidget, StreamPlayer};
use egui::{Color32, Label, Response, RichText, TextWrapMode, Ui, Widget};
use nostrdb::{Filter, NoteKey, Subscription, Transaction};
use std::borrow::Borrow;

pub struct StreamPage {
    link: NostrLink,
    event: Option<OwnedNote>,
    player: Option<StreamPlayer>,
    chat: Option<Chat>,
    sub: SubWrapper,
}

impl StreamPage {
    pub fn new_from_link(ndb: &NDBWrapper, tx: &Transaction, link: NostrLink) -> Self {
        let f: Filter = link.borrow().try_into().unwrap();
        let f = [f.limit_mut(1)];
        let (sub, events) = ndb.subscribe_with_results("streams", &f, tx, 1);
        Self {
            link,
            sub,
            event: events
                .first()
                .map_or(None, |n| Some(OwnedNote(n.note_key.as_u64()))),
            chat: None,
            player: None,
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
                .map_or(None, |f| Some(f))
        } else {
            None
        };
        if let Some(event) = event {
            if let Some(stream) = event.stream() {
                if self.player.is_none() {
                    let p = StreamPlayer::new(ui.ctx(), &stream);
                    self.player = Some(p);
                }
            }

            if let Some(player) = &mut self.player {
                player.ui(ui);
            }

            let title = RichText::new(match event.get_tag_value("title") {
                Some(s) => s.variant().str().unwrap_or("Unknown"),
                None => "Unknown",
            })
            .size(16.)
            .color(Color32::WHITE);
            ui.add(Label::new(title).wrap_mode(TextWrapMode::Truncate));

            if self.chat.is_none() {
                let chat = Chat::new(self.link.clone(), &services.ndb, services.tx);
                self.chat = Some(chat);
            }

            if let Some(c) = self.chat.as_mut() {
                c.render(ui, services)
            } else {
                ui.label("Loading..")
            }
        } else {
            ui.label("Loading..")
        }
    }
}
