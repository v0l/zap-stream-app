mod header;
mod stream;
mod stream_list;
mod avatar;
mod stream_player;
mod video_placeholder;
mod chat;
mod chat_message;
mod profile;

use egui::{Response, Ui};
use crate::route::RouteServices;

pub trait NostrWidget {
    fn render(&mut self, ui: &mut Ui, services: &RouteServices<'_>) -> Response;
}

pub use self::avatar::Avatar;
pub use self::header::Header;
pub use self::stream_list::StreamList;
pub use self::video_placeholder::VideoPlaceholder;
pub use self::stream_player::StreamPlayer;
pub use self::profile::Profile;
pub use self::chat::Chat;
