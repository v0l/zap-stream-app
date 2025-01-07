use crate::link::NostrLink;
use crate::services::ffmpeg_loader::FfmpegLoader;
use egui::load::SizedTexture;
use egui::{Context, Image, TextureHandle};
use egui_inbox::RequestRepaintTrait;
use enostr::EventClientMessage;
use itertools::Itertools;
use log::{info, warn};
use nostr::{Event, EventBuilder, JsonUtil, Kind, Tag};
use nostrdb::{NdbProfile, NoteKey, Transaction};
use notedeck::{AppContext, ImageCache};
use poll_promise::Promise;
use std::path::Path;
use std::sync::mpsc;
use std::task::Poll;

mod home;
mod login;
mod stream;

pub mod page {
    use crate::route::{home, login, stream};
    pub use home::HomePage;
    pub use login::LoginPage;
    pub use stream::StreamPage;
}

#[derive(PartialEq)]
pub enum RouteType {
    HomePage,
    EventPage {
        link: NostrLink,
        event: Option<NoteKey>,
    },
    ProfilePage {
        link: NostrLink,
        profile: Option<NoteKey>,
    },
    LoginPage,

    // special kind for modifying route state
    Action(RouteAction),
}

#[derive(PartialEq)]
pub enum RouteAction {
    DemandProfile([u8; 32]),
}

pub struct RouteServices<'a, 'ctx> {
    pub router: mpsc::Sender<RouteType>,
    pub egui: Context,
    pub tx: &'a Transaction,
    pub ctx: &'a mut AppContext<'ctx>,
}

impl<'a, 'ctx> RouteServices<'a, 'ctx> {
    pub fn navigate(&self, route: RouteType) {
        self.router.send(route).expect("route send failed");
        self.egui.request_repaint();
    }

    pub fn action(&self, route: RouteAction) {
        self.router
            .send(RouteType::Action(route))
            .expect("route send failed");
        self.egui.request_repaint();
    }

    pub fn broadcast_event(&mut self, event: Event) {
        let ev_json = event.as_json();
        if let Err(e) = self.ctx.ndb.process_event(&ev_json) {
            warn!("Failed to submit event {}", e);
        }
        self.ctx
            .pool
            .send(&enostr::ClientMessage::Event(EventClientMessage {
                note_json: ev_json,
            }));
    }

    /// Load/Fetch profiles
    pub fn profile(&self, pk: &[u8; 32]) -> Option<NdbProfile<'a>> {
        let p = self
            .ctx
            .ndb
            .get_profile_by_pubkey(self.tx, pk)
            .map(|p| p.record().profile())
            .ok()
            .flatten();
        if p.is_none() {
            self.action(RouteAction::DemandProfile(pk.clone()));
        }
        p
    }

    /// Load image from URL
    pub fn image<'img, 'b>(&'b mut self, url: &'b str) -> Image<'img> {
        image_from_cache(self.ctx.img_cache, &self.egui, url)
    }

    /// Load image from bytes
    pub fn image_bytes(&self, name: &'static str, data: &'static [u8]) -> Image<'_> {
        // TODO: loader
        Image::from_bytes(name, data)
    }

    pub fn write_live_chat_msg(&self, link: &NostrLink, msg: &str) -> Option<Event> {
        if msg.len() == 0 {
            return None;
        }
        if let Some(acc) = self.ctx.accounts.get_selected_account() {
            if let Some(key) = &acc.secret_key {
                let nostr_key =
                    nostr::Keys::new(nostr::SecretKey::from_slice(key.as_secret_bytes()).unwrap());
                return Some(
                    EventBuilder::new(Kind::LiveEventMessage, msg)
                        .tag(Tag::parse(&link.to_tag()).unwrap())
                        .sign_with_keys(&nostr_key)
                        .ok()?,
                );
            }
        }
        None
    }
}
const BLACK_PIXEL: [u8; 4] = [0, 0, 0, 0];
pub fn image_from_cache<'a>(img_cache: &mut ImageCache, ctx: &Context, url: &str) -> Image<'a> {
    if let Some(promise) = img_cache.map().get(url) {
        match promise.poll() {
            Poll::Ready(Ok(t)) => Image::new(SizedTexture::from_handle(t)),
            _ => Image::from_bytes(url.to_string(), &BLACK_PIXEL),
        }
    } else {
        let fetch = fetch_img(img_cache, ctx, url);
        img_cache.map_mut().insert(url.to_string(), fetch);
        Image::from_bytes(url.to_string(), &BLACK_PIXEL)
    }
}

fn fetch_img(
    img_cache: &ImageCache,
    ctx: &Context,
    url: &str,
) -> Promise<notedeck::Result<TextureHandle>> {
    let k = ImageCache::key(url);
    let dst_path = img_cache.cache_dir.join(k);
    if dst_path.exists() {
        let ctx = ctx.clone();
        let url = url.to_owned();
        let dst_path = dst_path.clone();
        Promise::spawn_blocking(move || {
            info!("Loading image from disk: {}", dst_path.display());
            match FfmpegLoader::new().load_image(dst_path) {
                Ok(img) => Ok(ctx.load_texture(&url, img, Default::default())),
                Err(e) => Err(notedeck::Error::Generic(e.to_string())),
            }
        })
    } else {
        fetch_img_from_net(&dst_path, ctx, url)
    }
}

fn fetch_img_from_net(
    cache_path: &Path,
    ctx: &Context,
    url: &str,
) -> Promise<notedeck::Result<TextureHandle>> {
    let (sender, promise) = Promise::new();
    let request = ehttp::Request::get(url);
    let ctx = ctx.clone();
    let cloned_url = url.to_owned();
    let cache_path = cache_path.to_owned();
    ehttp::fetch(request, move |response| {
        let handle = response
            .and_then(|img| {
                std::fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
                std::fs::write(&cache_path, &img.bytes).unwrap();
                info!("Loading image from net: {}", cloned_url);
                let img_loaded = FfmpegLoader::new()
                    .load_image(cache_path)
                    .map_err(|e| e.to_string())?;

                Ok(ctx.load_texture(&cloned_url, img_loaded, Default::default()))
            })
            .map_err(notedeck::Error::Generic);

        sender.send(handle);
        ctx.request_repaint();
    });

    promise
}
