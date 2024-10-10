use crate::link::NostrLink;
use crate::note_util::OwnedNote;
use crate::route;
use crate::route::home::HomePage;
use crate::route::stream::StreamPage;
use crate::services::profile::ProfileService;
use crate::widgets::{Header, StreamList};
use egui::{Context, Response, ScrollArea, Ui, Widget};
use egui_inbox::{UiInbox, UiInboxSender};
use egui_video::Player;
use log::{info, warn};
use nostr_sdk::Client;
use nostrdb::{Filter, Ndb, Note, Transaction};

mod stream;
mod home;

pub enum Routes {
    HomePage,
    Event {
        link: NostrLink,
        event: Option<OwnedNote>,
    },
    ProfilePage {
        link: NostrLink,
        profile: Option<OwnedNote>,
    },

    // special kind for modifying route state
    Action(RouteAction),
}

pub enum RouteAction {
    Login([u8; 32]),
    StartPlayer(String),
    PausePlayer,
    SeekPlayer(f32),
    StopPlayer,
}

pub struct Router {
    current: Routes,
    router: UiInbox<Routes>,

    ctx: Context,
    profile_service: ProfileService,
    ndb: Ndb,
    login: Option<[u8; 32]>,
    player: Option<Player>,
}

impl Router {
    pub fn new(rx: UiInbox<Routes>, ctx: Context, client: Client, ndb: Ndb) -> Self {
        Self {
            current: Routes::HomePage,
            router: rx,
            ctx: ctx.clone(),
            profile_service: ProfileService::new(client.clone(), ctx.clone()),
            ndb,
            login: None,
            player: None,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let tx = Transaction::new(&self.ndb).unwrap();
        // handle app state changes
        while let Some(r) = self.router.read(ui).next() {
            if let Routes::Action(a) = r {
                match a {
                    RouteAction::Login(k) => {
                        self.login = Some(k)
                    }
                    RouteAction::StartPlayer(u) => {
                        if self.player.is_none() {
                            if let Ok(p) = Player::new(&self.ctx, &u) {
                                self.player = Some(p)
                            }
                        }
                    }
                    _ => info!("Not implemented")
                }
            } else {
                self.current = r;
            }
        }

        let mut svc = RouteServices {
            context: self.ctx.clone(),
            profile: &self.profile_service,
            router: self.router.sender(),
            ndb: self.ndb.clone(),
            tx: &tx,
            login: &self.login,
            player: &mut self.player,
        };

        // display app
        ScrollArea::vertical().show(ui, |ui| {
            ui.add(Header::new(&svc));
            match &self.current {
                Routes::HomePage => {
                    HomePage::new(&svc).ui(ui)
                }
                Routes::Event { link, event } => {
                    StreamPage::new(&mut svc, link, event)
                        .ui(ui)
                }
                _ => {
                    ui.label("Not found")
                }
            }
        }).inner
    }
}

pub struct RouteServices<'a> {
    pub context: Context, //cloned
    pub router: UiInboxSender<Routes>, //cloned
    pub ndb: Ndb, //cloned

    pub player: &'a mut Option<Player>,
    pub profile: &'a ProfileService, //ref
    pub tx: &'a Transaction, //ref
    pub login: &'a Option<[u8; 32]>, //ref
}

impl<'a> RouteServices<'a> {
    pub fn navigate(&self, route: Routes) {
        if let Err(e) = self.router.send(route) {
            warn!("Failed to navigate");
        }
    }
}