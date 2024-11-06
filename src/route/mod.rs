use crate::app::NativeLayerOps;
use crate::link::NostrLink;
use crate::login::Login;
use crate::note_util::OwnedNote;
use crate::route::home::HomePage;
use crate::route::login::LoginPage;
use crate::route::stream::StreamPage;
use crate::services::image_cache::ImageCache;
use crate::services::ndb_wrapper::NDBWrapper;
use crate::widgets::{Header, NostrWidget};
use egui::{Context, Response, Ui};
use egui_inbox::{UiInbox, UiInboxSender};
use log::{info, warn};
use nostr_sdk::{Client, Event, JsonUtil};
use nostrdb::{Ndb, Transaction};
use std::path::PathBuf;

mod home;
mod login;
mod stream;

#[derive(PartialEq)]
pub enum Routes {
    HomePage,
    EventPage {
        link: NostrLink,
        event: Option<OwnedNote>,
    },
    ProfilePage {
        link: NostrLink,
        profile: Option<OwnedNote>,
    },
    LoginPage,

    // special kind for modifying route state
    Action(RouteAction),
}

#[derive(PartialEq)]
pub enum RouteAction {
    ShowKeyboard,
    HideKeyboard,
}

pub struct Router<T: NativeLayerOps> {
    current: Routes,
    current_widget: Option<Box<dyn NostrWidget>>,
    router: UiInbox<Routes>,

    ctx: Context,
    ndb: NDBWrapper,
    login: Login,
    client: Client,
    image_cache: ImageCache,
    native_layer: T,
}

impl<T: NativeLayerOps> Drop for Router<T> {
    fn drop(&mut self) {
        self.login.save(&mut self.native_layer)
    }
}

impl<T: NativeLayerOps> Router<T> {
    pub fn new(
        data_path: PathBuf,
        ctx: Context,
        client: Client,
        ndb: Ndb,
        native_layer: T,
    ) -> Self {
        let mut login = Login::new();
        login.load(&native_layer);

        Self {
            current: Routes::HomePage,
            current_widget: None,
            router: UiInbox::new(),
            ctx: ctx.clone(),
            ndb: NDBWrapper::new(ctx.clone(), ndb.clone(), client.clone()),
            client,
            login,
            image_cache: ImageCache::new(data_path, ctx.clone()),
            native_layer,
        }
    }

    fn load_widget(&mut self, route: Routes, tx: &Transaction) {
        match &route {
            Routes::HomePage => {
                let w = HomePage::new(&self.ndb, tx);
                self.current_widget = Some(Box::new(w));
            }
            Routes::EventPage { link, .. } => {
                let w = StreamPage::new_from_link(&self.ndb, tx, link.clone());
                self.current_widget = Some(Box::new(w));
            }
            Routes::LoginPage => {
                let w = LoginPage::new();
                self.current_widget = Some(Box::new(w));
            }
            _ => warn!("Not implemented"),
        }
        self.current = route;
    }

    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let tx = self.ndb.start_transaction();

        // handle app state changes
        let q = self.router.read(ui);
        for r in q {
            if let Routes::Action(a) = r {
                match a {
                    RouteAction::ShowKeyboard => self.native_layer.show_keyboard(),
                    RouteAction::HideKeyboard => self.native_layer.hide_keyboard(),
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

        let mut svc = RouteServices {
            context: self.ctx.clone(),
            router: self.router.sender(),
            client: self.client.clone(),
            ndb: &self.ndb,
            tx: &tx,
            login: &mut self.login,
            img_cache: &self.image_cache,
        };

        // display app
        ui.vertical(|ui| {
            Header::new().render(ui, &mut svc);
            if let Some(w) = self.current_widget.as_mut() {
                w.render(ui, &mut svc)
            } else {
                ui.label("No widget")
            }
        })
        .response
    }
}

pub struct RouteServices<'a> {
    pub context: Context,              //cloned
    pub router: UiInboxSender<Routes>, //cloned
    pub client: Client,

    pub ndb: &'a NDBWrapper,       //ref
    pub tx: &'a Transaction,       //ref
    pub login: &'a mut Login,      //ref
    pub img_cache: &'a ImageCache, //ref
}

impl<'a> RouteServices<'a> {
    pub fn navigate(&self, route: Routes) {
        if let Err(e) = self.router.send(route) {
            warn!("Failed to navigate");
        }
    }

    pub fn action(&self, route: RouteAction) {
        if let Err(e) = self.router.send(Routes::Action(route)) {
            warn!("Failed to navigate");
        }
    }

    pub fn broadcast_event(&self, event: Event) {
        let client = self.client.clone();

        let ev_json = event.as_json();
        if let Err(e) = self.ndb.submit_event(&ev_json) {
            warn!("Failed to submit event {}", e);
        }
        tokio::spawn(async move {
            match client.send_event(event).await {
                Ok(e) => {
                    info!("Broadcast event: {:?}", e)
                }
                Err(e) => warn!("Failed to broadcast event: {:?}", e),
            }
        });
    }
}
