mod avatar;
mod button;
mod chat;
mod chat_message;
mod header;
mod profile;
mod stream_list;
mod stream_player;
mod stream_tile;
mod stream_title;
mod username;
mod video_placeholder;
mod write_chat;

use crate::route::RouteServices;
use egui::{Response, Ui};

pub trait NostrWidget {
    fn render(&mut self, ui: &mut Ui, services: &RouteServices<'_>) -> Response;
}

pub use self::avatar::Avatar;
pub use self::button::Button;
pub use self::chat::Chat;
pub use self::header::Header;
pub use self::profile::Profile;
pub use self::stream_list::StreamList;
pub use self::stream_player::StreamPlayer;
pub use self::stream_title::StreamTitle;
pub use self::username::Username;
pub use self::video_placeholder::VideoPlaceholder;
pub use self::write_chat::WriteChat;
