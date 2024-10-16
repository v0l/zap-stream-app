use crate::link::NostrLink;
use crate::note_util::OwnedNote;
use crate::route;
use crate::route::home::HomePage;
use crate::route::stream::StreamPage;
use crate::services::ndb_wrapper::NDBWrapper;
use crate::widgets::{Header, NostrWidget, StreamList};
use egui::{Context, Response, ScrollArea, Ui, Widget};
use egui_inbox::{UiInbox, UiInboxSender};
use egui_video::{Player, PlayerState};
use log::{info, warn};
use nostr_sdk::nips::nip01;
use nostr_sdk::{Client, Kind, PublicKey};
use nostrdb::{Filter, Ndb, Note, Transaction};
use std::borrow::Borrow;

mod home;
mod stream;

#[derive(PartialEq)]
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

#[derive(PartialEq)]
pub enum RouteAction {
    Login([u8; 32]),
}

pub struct Router {
    current: Routes,
    current_widget: Option<Box<dyn NostrWidget>>,
    router: UiInbox<Routes>,

    ctx: Context,
    ndb: NDBWrapper,
    login: Option<[u8; 32]>,
    client: Client,
}

impl Router {
    pub fn new(ctx: Context, client: Client, ndb: Ndb) -> Self {
        Self {
            current: Routes::HomePage,
            current_widget: None,
            router: UiInbox::new(),
            ctx: ctx.clone(),
            ndb: NDBWrapper::new(ctx.clone(), ndb.clone(), client.clone()),
            client,
            login: None,
        }
    }

    fn load_widget(&mut self, route: Routes, tx: &Transaction) {
        match &route {
            Routes::HomePage => {
                let w = HomePage::new(&self.ndb, tx);
                self.current_widget = Some(Box::new(w));
            }
            Routes::Event { link, .. } => {
                let w = StreamPage::new_from_link(&self.ndb, tx, link.clone());
                self.current_widget = Some(Box::new(w));
            }
            _ => warn!("Not implemented"),
        }
        self.current = route;
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let tx = self.ndb.start_transaction();

        // handle app state changes
        while let Some(r) = self.router.read(ui).next() {
            if let Routes::Action(a) = &r {
                match a {
                    RouteAction::Login(k) => self.login = Some(k.clone()),
                    _ => info!("Not implemented"),
                }
            } else {
                self.load_widget(r, &tx);
            }
        }

        // load homepage on start
        if self.current_widget.is_none() {
            self.load_widget(Routes::HomePage, &tx);
        }

        let svc = RouteServices {
            context: self.ctx.clone(),
            router: self.router.sender(),
            ndb: &self.ndb,
            tx: &tx,
            login: &self.login,
        };

        // display app
        ScrollArea::vertical()
            .show(ui, |ui| {
                Header::new().render(ui, &svc);
                if let Some(w) = self.current_widget.as_mut() {
                    w.render(ui, &svc)
                } else {
                    ui.label("No widget")
                }
            })
            .inner
    }
}

pub struct RouteServices<'a> {
    pub context: Context,              //cloned
    pub router: UiInboxSender<Routes>, //cloned

    pub ndb: &'a NDBWrapper,         //ref
    pub tx: &'a Transaction,         //ref
    pub login: &'a Option<[u8; 32]>, //ref
}

impl<'a> RouteServices<'a> {
    pub fn navigate(&self, route: Routes) {
        if let Err(e) = self.router.send(route) {
            warn!("Failed to navigate");
        }
    }
}
