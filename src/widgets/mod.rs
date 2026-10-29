mod avatar;
mod button;
mod chat;
mod chat_message;
mod chat_zap;
mod header;
mod pill;
mod placeholder_rect;
mod profile;
mod stream_list;
mod stream_player;
mod stream_tile;
mod stream_title;
mod text_input;
mod username;
mod write_chat;
mod zap;

use crate::note_ref::NoteRef;
use crate::route::RouteServices;
use crate::sub::SubRef;
use egui::{Response, Ui};
use nostrdb::{Filter, Ndb, Transaction};
use notedeck::{AppContext, ScopedSubIdentity, SubConfig, SubKey, SubOwnerKey};
use std::collections::HashSet;

/// A stateful widget which requests nostr data
pub trait NostrWidget {
    /// Render with widget on the UI
    fn render(&mut self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) -> Response;

    /// Update widget on draw
    fn update(&mut self, services: &mut RouteServices<'_, '_>) -> anyhow::Result<()>;
}

/// On widget update call this to update NDB data
pub fn sub_or_poll(
    ctx: &mut AppContext<'_>,
    tx: &Transaction,
    store: &mut HashSet<NoteRef>,
    sub: &mut Option<SubRef>,
    filters: Vec<Filter>,
) -> anyhow::Result<()> {
    let ndb: &Ndb = ctx.ndb;
    if let Some(sub) = sub {
        ndb.poll_for_notes(sub.sub, 500).into_iter().for_each(|e| {
            if let Ok(note) = ndb.get_note_by_key(tx, e) {
                store.insert(NoteRef::from_note(&note));
            }
        });
    } else {
        let s = ndb.subscribe(filters.as_slice())?;
        let owner = SubOwnerKey::new(("zap-stream-ndb", s.id()));
        let identity = ScopedSubIdentity::account(owner, SubKey::new(("ndb", s.id())));
        sub.replace(SubRef::new(s, ndb.clone(), owner));
        ndb.query(tx, filters.as_slice(), 500)?
            .into_iter()
            .for_each(|e| {
                store.insert(NoteRef::from_query_result(e));
            });
        let config = SubConfig::builder(filters).accounts_read_important().build();
        ctx.remote
            .scoped_subs(ctx.accounts)
            .set_sub(identity, config);
    }
    Ok(())
}

pub use self::avatar::Avatar;
pub use self::button::Button;
pub use self::chat::Chat;
pub use self::header::Header;
pub use self::pill::Pill;
pub use self::placeholder_rect::PlaceholderRect;
pub use self::profile::Profile;
pub use self::stream_list::StreamList;
pub use self::stream_player::StreamPlayer;
pub use self::stream_title::StreamTitle;
pub use self::text_input::NativeTextInput;
pub use self::username::Username;
pub use self::write_chat::WriteChat;
