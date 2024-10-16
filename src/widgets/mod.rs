mod avatar;
mod chat;
mod chat_message;
mod header;
mod profile;
mod stream;
mod stream_list;
mod stream_player;
mod video_placeholder;

use crate::route::RouteServices;
use egui::{Response, Ui};

pub trait NostrWidget {
    fn render(&mut self, ui: &mut Ui, services: &RouteServices<'_>) -> Response;
}

pub use self::avatar::Avatar;
pub use self::chat::Chat;
pub use self::header::Header;
pub use self::profile::Profile;
pub use self::stream_list::StreamList;
pub use self::stream_player::StreamPlayer;
pub use self::video_placeholder::VideoPlaceholder;
