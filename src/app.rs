use crate::profiles::ProfileLoader;
use crate::route::{page, RouteAction, RouteServices, RouteType};
use crate::widgets::{Header, NostrWidget};
use eframe::epaint::{FontFamily, Margin};
use eframe::CreationContext;
use egui::{Color32, FontData, FontDefinitions, Theme, Ui, Visuals};
use enostr::{PoolEvent, RelayEvent, RelayMessage};
use log::{error, info, warn};
use nostrdb::{Filter, Transaction};
use notedeck::AppContext;
use poll_promise::Promise;
use std::collections::HashMap;
use std::sync::mpsc;

pub struct ZapStreamApp {
    current: RouteType,
    routes_rx: mpsc::Receiver<RouteType>,
    routes_tx: mpsc::Sender<RouteType>,

    #[cfg(target_os = "android")]
    app: android_activity::AndroidApp,

    widget: Box<dyn NostrWidget>,
    profiles: ProfileLoader,
    fetch: HashMap<String, Promise<ehttp::Result<ehttp::Response>>>,
}

#[cfg(target_os = "android")]
impl ZapStreamApp {
    pub fn new(cc: &CreationContext, app: android_activity::AndroidApp) -> Self {
        let mut fd = FontDefinitions::default();
        fd.font_data.insert(
            "Outfit".to_string(),
            FontData::from_static(include_bytes!("../assets/Outfit-Light.ttf")),
        );
        fd.families
            .insert(FontFamily::Proportional, vec!["Outfit".to_string()]);
        cc.egui_ctx.set_fonts(fd);

        let (tx, rx) = mpsc::channel();
        Self {
            current: RouteType::HomePage,
            widget: Box::new(page::HomePage::new()),
            profiles: ProfileLoader::new(),
            routes_tx: tx,
            routes_rx: rx,
            app,
        }
    }
}

#[cfg(not(target_os = "android"))]
impl ZapStreamApp {
    pub fn new(cc: &CreationContext) -> Self {
        let mut fd = FontDefinitions::default();
        fd.font_data.insert(
            "Outfit".to_string(),
            FontData::from_static(include_bytes!("../assets/Outfit-Light.ttf")),
        );
        fd.families
            .insert(FontFamily::Proportional, vec!["Outfit".to_string()]);
        cc.egui_ctx.set_fonts(fd);

        let (tx, rx) = mpsc::channel();
        Self {
            current: RouteType::HomePage,
            widget: Box::new(page::HomePage::new()),
            profiles: ProfileLoader::new(),
            routes_tx: tx,
            routes_rx: rx,
            fetch: HashMap::new(),
        }
    }
}

impl notedeck::App for ZapStreamApp {
    fn update(&mut self, ctx: &mut AppContext<'_>, ui: &mut Ui) {
        ctx.accounts.update(ctx.ndb, ctx.pool, ui.ctx());
        while let Some(PoolEvent { event, relay }) = ctx.pool.try_recv() {
            if let RelayEvent::Message(msg) = (&event).into() {
                match msg {
                    RelayMessage::OK(_) => {}
                    RelayMessage::Eose(_) => {}
                    RelayMessage::Event(_sub, ev) => {
                        if let Err(e) = ctx.ndb.process_event(ev) {
                            error!("Error processing event: {:?}", e);
                        }
                    }
                    RelayMessage::Notice(m) => warn!("Notice from {}: {}", relay, m),
                }
            }
        }

        // reset theme
        ui.ctx().set_visuals_of(
            Theme::Dark,
            Visuals {
                panel_fill: Color32::BLACK,
                override_text_color: Some(Color32::WHITE),
                ..Default::default()
            },
        );

        let mut app_frame = egui::containers::Frame::default();
        app_frame.inner_margin = self.frame_margin();

        // handle app state changes
        while let Ok(r) = self.routes_rx.try_recv() {
            if let RouteType::Action(a) = r {
                match a {
                    RouteAction::DemandProfile(p) => {
                        self.profiles.demand(p);
                    }
                    _ => info!("Not implemented"),
                }
            } else {
                self.current = r;
                match &self.current {
                    RouteType::HomePage => {
                        self.widget = Box::new(page::HomePage::new());
                    }
                    RouteType::EventPage { link, .. } => {
                        self.widget = Box::new(page::StreamPage::new_from_link(link.clone()));
                    }
                    RouteType::LoginPage => {
                        self.widget = Box::new(page::LoginPage::new());
                    }
                    RouteType::Action { .. } => panic!("Actions!"),
                    _ => panic!("Not implemented"),
                }
            }
        }
        egui::CentralPanel::default()
            .frame(app_frame)
            .show(ui.ctx(), |ui| {
                let tx = Transaction::new(ctx.ndb).expect("transaction");
                // display app
                ui.vertical(|ui| {
                    let mut svc = RouteServices::new(
                        ui.ctx().clone(),
                        &tx,
                        ctx,
                        self.routes_tx.clone(),
                        &mut self.fetch,
                    );
                    Header::new().render(ui, &mut svc, &tx);
                    if let Err(e) = self.widget.update(&mut svc) {
                        error!("{}", e);
                    }
                    self.widget.render(ui, &mut svc);
                })
                .response
            });

        let profiles = self.profiles.next();
        if !profiles.is_empty() {
            info!("Profiles: {:?}", profiles);
            ctx.pool.subscribe(
                "profiles".to_string(),
                vec![Filter::new().kinds([0]).authors(&profiles).build()],
            );
        }
    }
}

#[cfg(not(target_os = "android"))]
impl ZapStreamApp {
    fn frame_margin(&self) -> Margin {
        Margin::ZERO
    }
}

#[cfg(target_os = "android")]
impl ZapStreamApp {
    fn frame_margin(&self) -> Margin {
        if let Some(wd) = self.app.native_window() {
            let (w, h) = (wd.width(), wd.height());
            let c_rect = self.app.content_rect();
            let dpi = self.app.config().density().unwrap_or(160);
            let dpi_scale = dpi as f32 / 160.0;
            // TODO: this calc is weird but seems to work on my phone
            Margin {
                bottom: (h - c_rect.bottom) as f32,
                left: c_rect.left as f32,
                right: (w - c_rect.right) as f32,
                top: (c_rect.top - (h - c_rect.bottom)) as f32,
            } / dpi_scale
        } else {
            Margin::ZERO
        }
    }
}
