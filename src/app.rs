use crate::route::{page, RouteServices, RouteType};
use crate::widgets::{Header, NostrWidget};
use eframe::epaint::{FontFamily, Margin};
use eframe::CreationContext;
use egui::{Color32, FontData, FontDefinitions, Ui};
use enostr::ewebsock::{WsEvent, WsMessage};
use enostr::{PoolEvent, RelayEvent, RelayMessage};
use log::{error, info, warn};
use nostrdb::Transaction;
use notedeck::AppContext;
use std::ops::Div;
use std::sync::mpsc;

pub struct ZapStreamApp {
    current: RouteType,
    routes_rx: mpsc::Receiver<RouteType>,
    routes_tx: mpsc::Sender<RouteType>,

    widget: Box<dyn NostrWidget>,
}

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
            routes_tx: tx,
            routes_rx: rx,
        }
    }
}

impl notedeck::App for ZapStreamApp {
    fn update(&mut self, ctx: &mut AppContext<'_>, ui: &mut Ui) {
        ctx.accounts.update(ctx.ndb, ctx.pool, ui.ctx());
        while let Some(PoolEvent { event, relay }) = ctx.pool.try_recv() {
            match (&event).into() {
                RelayEvent::Message(msg) => match msg {
                    RelayMessage::OK(_) => {}
                    RelayMessage::Eose(_) => {}
                    RelayMessage::Event(_sub, ev) => {
                        if let Err(e) = ctx.ndb.process_event(ev) {
                            error!("Error processing event: {:?}", e);
                        }
                    }
                    RelayMessage::Notice(m) => warn!("Notice from {}: {}", relay, m),
                },
                _ => {}
            }
        }

        let mut app_frame = egui::containers::Frame::default();
        let margin = self.frame_margin();

        app_frame.inner_margin = margin;
        app_frame.stroke.color = Color32::BLACK;

        // handle app state changes
        while let Ok(r) = self.routes_rx.try_recv() {
            if let RouteType::Action(a) = r {
                match a {
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
                ui.visuals_mut().override_text_color = Some(Color32::WHITE);

                // display app
                ui.vertical(|ui| {
                    let mut svc = RouteServices {
                        router: self.routes_tx.clone(),
                        tx: Transaction::new(ctx.ndb).expect("transaction"),
                        egui: ui.ctx().clone(),
                        ctx,
                    };
                    Header::new().render(ui, &mut svc);
                    if let Err(e) = self.widget.update(&mut svc) {
                        error!("{}", e);
                    }
                    self.widget.render(ui, &mut svc);
                })
                .response
            });
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
        if let Some(wd) = self.native_window() {
            let (w, h) = (wd.width(), wd.height());
            let c_rect = self.content_rect();
            let dpi = self.config().density().unwrap_or(160);
            let dpi_scale = dpi as f32 / 160.0;
            // TODO: this calc is weird but seems to work on my phone
            Margin {
                bottom: (h - c_rect.bottom) as f32,
                left: c_rect.left as f32,
                right: (w - c_rect.right) as f32,
                top: (c_rect.top - (h - c_rect.bottom)) as f32,
            }
            .div(dpi_scale)
        } else {
            Margin::ZERO
        }
    }
}
