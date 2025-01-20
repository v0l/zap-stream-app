use crate::link::NostrLink;
use crate::services::ffmpeg_loader::FfmpegLoader;
use crate::widgets::PlaceholderRect;
use anyhow::{anyhow, bail};
use egui::load::SizedTexture;
use egui::{vec2, Context, Id, Image, ImageSource, TextureHandle, Ui, Vec2};
use egui_video::ffmpeg_rs_raw::Transcoder;
use ehttp::Response;
use enostr::EventClientMessage;
use lnurl::lightning_address::LightningAddress;
use lnurl::pay::PayResponse;
use lnurl::LnUrlResponse;
use log::{info, warn};
use nostr::{serde_json, Event, EventBuilder, JsonUtil, Keys, Kind, SecretKey, Tag};
use nostrdb::{NdbProfile, NoteKey, Transaction};
use notedeck::{AppContext, ImageCache};
use poll_promise::Promise;
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc;
use std::task::Poll;

mod home;
mod login;
mod profile;
mod stream;

pub mod page {
    pub use super::home::HomePage;
    pub use super::login::LoginPage;
    pub use super::profile::ProfilePage;
    pub use super::stream::StreamPage;
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
    pub egui: Context,
    pub tx: &'a Transaction,
    pub ctx: &'a mut AppContext<'ctx>,

    router: mpsc::Sender<RouteType>,
    fetch: &'a mut HashMap<String, Promise<ehttp::Result<Response>>>,
}

impl<'a, 'ctx> RouteServices<'a, 'ctx> {
    pub fn new(
        egui: Context,
        tx: &'a Transaction,
        ctx: &'a mut AppContext<'ctx>,
        router: mpsc::Sender<RouteType>,
        fetch: &'a mut HashMap<String, Promise<ehttp::Result<Response>>>,
    ) -> Self {
        Self {
            egui,
            tx,
            ctx,
            router,
            fetch,
        }
    }

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
            self.action(RouteAction::DemandProfile(*pk));
        }
        p
    }

    /// Create a poll_promise fetch
    pub fn fetch(&mut self, url: &str) -> Poll<&ehttp::Result<Response>> {
        if !self.fetch.contains_key(url) {
            let (sender, promise) = Promise::new();
            let request = ehttp::Request::get(url);
            let ctx = self.egui.clone();
            ehttp::fetch(request, move |response| {
                sender.send(response);
                ctx.request_repaint();
            });
            info!("Fetching {}", url);
            self.fetch.insert(url.to_string(), promise);
        }
        self.fetch.get(url).expect("fetch").poll()
    }

    pub fn fetch_lnurlp(&mut self, pubkey: &[u8; 32]) -> anyhow::Result<Poll<PayResponse>> {
        let target = self
            .profile(pubkey)
            .and_then(|p| p.lud16())
            .ok_or(anyhow!("No lightning address found"))?;

        let addr = LightningAddress::new(target)?;
        match self.fetch(&addr.lnurlp_url()) {
            Poll::Ready(Ok(r)) => {
                if r.ok {
                    let rsp: PayResponse = serde_json::from_slice(&r.bytes)?;
                    Ok(Poll::Ready(rsp))
                } else {
                    bail!("Invalid response code {}", r.status);
                }
            }
            Poll::Ready(Err(e)) => Err(anyhow!("{}", e)),
            Poll::Pending => Ok(Poll::Pending),
        }
    }

    pub fn write_live_chat_msg(&self, link: &NostrLink, msg: &str) -> Option<Event> {
        if msg.is_empty() {
            return None;
        }
        if let Some(key) = self.current_account_keys() {
            EventBuilder::new(Kind::LiveEventMessage, msg)
                .tag(Tag::parse(link.to_tag()).unwrap())
                .sign_with_keys(&key)
                .ok()
        } else {
            None
        }
    }

    pub fn current_account_keys(&self) -> Option<Keys> {
        self.ctx
            .accounts
            .get_selected_account()
            .and_then(|acc| acc.secret_key.as_ref().map(|k| Keys::new(k.clone())))
    }

    /// Simple wrapper around egui temp data
    pub fn get<T: Clone + 'static>(&self, k: &str) -> Option<T> {
        let id = Id::new(k);
        self.egui.data(|d| d.get_temp(id))
    }

    /// Simple wrapper around egui temp data
    pub fn set<T: Clone + Send + Sync + 'static>(&mut self, k: &str, v: T) {
        self.egui.data_mut(|d| d.insert_temp(Id::new(k), v));
    }
}

const LOGO_BYTES: &[u8] = include_bytes!("../resources/logo.svg");

pub fn image_from_cache<'a>(
    img_cache: &mut ImageCache,
    ui: &Ui,
    url: &str,
    size: Option<Vec2>,
) -> Option<Image<'a>> {
    if url.len() == 0 {
        return None;
    }
    let cache_key = if let Some(s) = size {
        format!("{}:{}", url, s)
    } else {
        url.to_string()
    };
    if let Some(promise) = img_cache.map().get(&cache_key) {
        match promise.poll() {
            Poll::Ready(Ok(t)) => Some(Image::new(SizedTexture::from_handle(t))),
            _ => None,
        }
    } else {
        let fetch = fetch_img(img_cache, ui.ctx(), url, size);
        img_cache.map_mut().insert(cache_key.clone(), fetch);
        None
    }
}

fn fetch_img(
    img_cache: &ImageCache,
    ctx: &Context,
    url: &str,
    size: Option<Vec2>,
) -> Promise<notedeck::Result<TextureHandle>> {
    let name = ImageCache::key(url);
    let dst_path = img_cache.cache_dir.join(&name);
    if dst_path.exists() {
        let ctx = ctx.clone();
        Promise::spawn_thread("load_from_disk", move || {
            info!("Loading image from disk: {}", dst_path.display());
            match FfmpegLoader::new().load_image(dst_path, size) {
                Ok(img) => {
                    ctx.request_repaint();
                    ctx.forget_image(&name);
                    Ok(ctx.load_texture(&name, img, Default::default()))
                }
                Err(e) => Err(notedeck::Error::Generic(e.to_string())),
            }
        })
    } else {
        let url = url.to_string();
        let ctx = ctx.clone();
        Promise::spawn_thread("load_from_net", move || {
            let img = match fetch_img_from_net(&url).block_and_take() {
                Ok(img) => img,
                Err(e) => return Err(notedeck::Error::Generic(e.to_string())),
            };
            std::fs::create_dir_all(&dst_path.parent().unwrap()).unwrap();
            std::fs::write(&dst_path, &img.bytes).unwrap();

            info!("Loading image from net: {}", &url);
            match FfmpegLoader::new().load_image(dst_path, size) {
                Ok(img) => {
                    ctx.request_repaint();
                    ctx.forget_image(&name);
                    Ok(ctx.load_texture(&name, img, Default::default()))
                }
                Err(e) => Err(notedeck::Error::Generic(e.to_string())),
            }
        })
    }
}

fn fetch_img_from_net(url: &str) -> Promise<ehttp::Result<Response>> {
    let (sender, promise) = Promise::new();
    let request = ehttp::Request::get(url);
    info!("Downloaded image: {}", url);
    ehttp::fetch(request, move |response| {
        sender.send(response);
    });
    promise
}
