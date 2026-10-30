use crate::profiles::ProfileLoader;
use crate::route::{page, RouteAction, RouteServices, RouteType};
use crate::widgets::{Header, NostrWidget};
use egui::epaint::{FontFamily, Margin};
use egui::{Color32, FontData, FontDefinitions, Theme, Ui, Visuals};
use log::{error, info};
use nostrdb::{Filter, Transaction};
use notedeck::{
    AppContext, AppResponse, ScopedSubIdentity, SubConfig, SubKey, SubOwnerKey,
};
use poll_promise::Promise;
use std::collections::HashMap;
use std::sync::{mpsc, Arc};

pub struct ZapStreamApp {
    current: RouteType,
    routes_rx: mpsc::Receiver<RouteType>,
    routes_tx: mpsc::Sender<RouteType>,

    #[cfg(target_os = "android")]
    app: android_activity::AndroidApp,

    widget: Box<dyn NostrWidget>,
    profiles: ProfileLoader,
    fetch: HashMap<String, Promise<ehttp::Result<ehttp::Response>>>,
    fonts_set: bool,
    frame_stats: Option<crate::frame_stats::FrameStats>,
    startup_link: Option<crate::link::NostrLink>,
    render_state: Option<egui_wgpu::RenderState>,
}

impl ZapStreamApp {
    #[cfg(target_os = "android")]
    pub fn new(app: android_activity::AndroidApp, render_state: Option<egui_wgpu::RenderState>) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            current: RouteType::HomePage,
            widget: Box::new(page::HomePage::new()),
            profiles: ProfileLoader::new(),
            routes_tx: tx,
            routes_rx: rx,
            fetch: HashMap::new(),
            fonts_set: false,
            frame_stats: crate::frame_stats::FrameStats::enabled(),
            startup_link: startup_link_from_env(),
            render_state,
            app,
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn new(render_state: Option<egui_wgpu::RenderState>) -> Self {
        Self::base(render_state)
    }

    #[cfg(not(target_os = "android"))]
    fn base(render_state: Option<egui_wgpu::RenderState>) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            current: RouteType::HomePage,
            widget: Box::new(page::HomePage::new()),
            profiles: ProfileLoader::new(),
            routes_tx: tx,
            routes_rx: rx,
            fetch: HashMap::new(),
            fonts_set: false,
            frame_stats: crate::frame_stats::FrameStats::enabled(),
            startup_link: startup_link_from_env(),
            render_state,
        }
    }

    fn setup_fonts(&mut self, ctx: &egui::Context) {
        if self.fonts_set {
            return;
        }
        self.fonts_set = true;
        let mut fd = FontDefinitions::default();
        fd.font_data.insert(
            "Outfit".to_string(),
            Arc::new(FontData::from_static(include_bytes!(
                "../assets/Outfit-Light.ttf"
            ))),
        );
        fd.families
            .insert(FontFamily::Proportional, vec!["Outfit".to_string()]);
        ctx.set_fonts(fd);
    }
}

fn startup_link_from_env() -> Option<crate::link::NostrLink> {
    let v = std::env::var("ZAP_STREAM_OPEN").ok()?;
    let (author, d) = v.split_once(':')?;
    let author: [u8; 32] = nostr::prelude::hex::decode(author).ok()?.try_into().ok()?;
    Some(crate::link::NostrLink::new(
        crate::link::NostrLinkType::Coordinate,
        crate::link::IdOrStr::Str(d.to_string()),
        Some(30_311),
        Some(author),
        vec![],
    ))
}

fn profiles_sub_identity() -> ScopedSubIdentity {
    ScopedSubIdentity::account(
        SubOwnerKey::new("zap-stream-profiles"),
        SubKey::new("profiles"),
    )
}

impl notedeck::App for ZapStreamApp {
    fn render(&mut self, ctx: &mut AppContext<'_>, ui: &mut Ui) -> AppResponse {
        self.setup_fonts(ui.ctx());
        if let Some(s) = self.frame_stats.as_mut() {
            s.tick();
        }

        ui.ctx().set_visuals_of(
            Theme::Dark,
            Visuals {
                panel_fill: Color32::BLACK,
                override_text_color: Some(Color32::WHITE),
                ..Visuals::dark()
            },
        );

        if let Some(link) = self.startup_link.take() {
            self.widget = Box::new(page::StreamPage::new_from_link(
                link.clone(),
                self.render_state.clone(),
            ));
            self.current = RouteType::EventPage { link, event: None };
        }

        let released = crate::sub::take_released();
        if !released.is_empty() {
            let mut api = ctx.remote.scoped_subs(ctx.accounts);
            for owner in released {
                api.drop_owner(owner);
            }
        }

        while let Ok(r) = self.routes_rx.try_recv() {
            if let RouteType::Action(a) = r {
                match a {
                    RouteAction::DemandProfile(p) => {
                        self.profiles.demand(p);
                    }
                }
            } else {
                self.current = r;
                match &self.current {
                    RouteType::HomePage => {
                        self.widget = Box::new(page::HomePage::new());
                    }
                    RouteType::EventPage { link, .. } => {
                        self.widget = Box::new(page::StreamPage::new_from_link(
                            link.clone(),
                            self.render_state.clone(),
                        ));
                    }
                    RouteType::LoginPage => {
                        self.widget = Box::new(page::LoginPage::new());
                    }
                    RouteType::ProfilePage { link } => {
                        self.widget = Box::new(page::ProfilePage::new(
                            link.id.as_bytes().try_into().unwrap(),
                        ));
                    }
                    RouteType::Action(_) => unreachable!(),
                }
            }
        }

        let app_frame = egui::containers::Frame::default().outer_margin(self.frame_margin());
        let egui_ctx = ui.ctx().clone();
        egui::CentralPanel::default()
            .frame(app_frame)
            .show(ui, |ui| {
                let tx = Transaction::new(ctx.ndb).expect("transaction");
                ui.vertical(|ui| {
                    let mut svc = RouteServices::new(
                        egui_ctx,
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
                });
            });

        let profiles = self.profiles.next();
        if !profiles.is_empty() {
            info!("Profiles: {:?}", profiles);
            let config = SubConfig::builder(vec![Filter::new()
                .kinds([0])
                .authors(&profiles)
                .build()])
            .accounts_read_important()
            .build();
            ctx.remote
                .scoped_subs(ctx.accounts)
                .set_sub(profiles_sub_identity(), config);
        }

        AppResponse::none()
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
            let to_i8 = |v: f32| v.round().clamp(0.0, 127.0) as i8;
            Margin {
                bottom: to_i8((h - c_rect.bottom) as f32 / dpi_scale),
                left: to_i8(c_rect.left as f32 / dpi_scale),
                right: to_i8((w - c_rect.right) as f32 / dpi_scale),
                top: to_i8((c_rect.top - (h - c_rect.bottom)) as f32 / dpi_scale),
            }
        } else {
            Margin::ZERO
        }
    }
}
