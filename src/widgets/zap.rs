use crate::link::NostrLink;
use crate::route::RouteServices;
use crate::stream_info::StreamInfo;
use crate::theme::{
    FONT_SIZE_LG, MARGIN_DEFAULT, NEUTRAL_700, NEUTRAL_800, NEUTRAL_900, PRIMARY, ROUNDING_DEFAULT,
};
use crate::widgets::{Button, NativeTextInput};
use crate::zap::format_sats;
use anyhow::{anyhow, bail};
use egui::text::{LayoutJob, TextWrapping};
use egui::{vec2, Frame, Grid, Response, RichText, Stroke, TextFormat, TextWrapMode, Ui, Widget};
use egui_modal::Modal;
use egui_qr::QrCodeWidget;
use enostr::PoolRelay;
use itertools::Itertools;
use lnurl::pay::{LnURLPayInvoice, PayResponse};
use nostr::prelude::{hex, ZapRequestData};
use nostr::{serde_json, EventBuilder, JsonUtil, Kind, PublicKey, Tag, Url};
use nostrdb::Note;
use std::fmt::{Display, Formatter};
use std::task::Poll;

pub enum ZapTarget<'a> {
    PublicKey { pubkey: [u8; 32] },
    Event { event: &'a Note<'a> },
}

impl Display for ZapTarget<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ZapTarget::PublicKey { pubkey } => write!(f, "{}", hex::encode(pubkey)),
            ZapTarget::Event { event } => write!(f, "{}", hex::encode(event.id())),
        }
    }
}

#[derive(Clone)]
pub enum ZapState {
    NotStarted,
    Ready { service: PayResponse },
    FetchingInvoice { callback: String },
    Invoice { invoice: LnURLPayInvoice },
    Error(String),
}

pub struct ZapButton<'a> {
    target: ZapTarget<'a>,
}

impl<'a> ZapButton<'a> {
    pub fn pubkey(pubkey: [u8; 32]) -> Self {
        Self {
            target: ZapTarget::PublicKey { pubkey },
        }
    }

    pub fn event(event: &'a Note<'a>) -> Self {
        Self {
            target: ZapTarget::Event { event },
        }
    }

    pub fn render(self, ui: &mut Ui, services: &mut RouteServices) -> Response {
        // TODO: fix id
        let modal = Modal::new(ui.ctx(), format!("zapper-{}", 0)).with_close_on_outside_click(true);

        let resp = Button::new().show(ui, |ui| ui.label("ZAP"));
        if resp.clicked() {
            modal.open();
        }
        ui.visuals_mut().window_rounding = ROUNDING_DEFAULT.into();
        ui.visuals_mut().window_stroke = Stroke::NONE;
        ui.visuals_mut().window_fill = NEUTRAL_900;

        modal.show(|ui| {
            Frame::none().inner_margin(MARGIN_DEFAULT).show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 8.0);

                let pubkey = match &self.target {
                    ZapTarget::PublicKey { pubkey } => pubkey,
                    ZapTarget::Event { event } => event.host(),
                };

                // zapping state machine
                let zap_state = services.get("zap_state").unwrap_or(ZapState::NotStarted);
                match &zap_state {
                    ZapState::NotStarted => match services.fetch_lnurlp(pubkey) {
                        Ok(Poll::Ready(r)) => {
                            services.set("zap_state", ZapState::Ready { service: r })
                        }
                        Err(e) => services.set("zap_state", ZapState::Error(e.to_string())),
                        _ => {}
                    },
                    ZapState::FetchingInvoice { callback } => {
                        match self.zap_get_invoice(callback, services) {
                            Ok(Poll::Ready(s)) => {
                                services.set("zap_state", ZapState::Invoice { invoice: s })
                            }
                            Err(e) => services.set("zap_state", ZapState::Error(e.to_string())),
                            _ => {}
                        }
                    }
                    _ => {}
                }

                // when ready state, show zap button
                match &zap_state {
                    ZapState::Ready { service } => {
                        self.render_input(ui, services, pubkey, service);
                    }
                    ZapState::Invoice { invoice } => {
                        if let Ok(q) = QrCodeWidget::from_data(invoice.pr.as_bytes()) {
                            ui.vertical_centered(|ui| {
                                ui.add_sized(vec2(256., 256.), q);

                                let mut job = LayoutJob::default();
                                job.wrap = TextWrapping::from_wrap_mode_and_width(
                                    TextWrapMode::Truncate,
                                    ui.available_width(),
                                );
                                job.append(&invoice.pr, 0.0, TextFormat::default());
                                ui.label(job);
                            });
                        }
                    }
                    ZapState::Error(e) => {
                        ui.label(e);
                    }
                    _ => {}
                }
            })
        });

        if modal.was_outside_clicked() {
            services.set("zap_state", ZapState::NotStarted)
        }
        resp
    }

    fn zap_get_invoice(
        &self,
        callback: &str,
        services: &mut RouteServices,
    ) -> anyhow::Result<Poll<LnURLPayInvoice>> {
        match services.fetch(callback) {
            Poll::Ready(Ok(r)) => {
                if r.ok {
                    let inv: LnURLPayInvoice = serde_json::from_slice(&r.bytes)?;
                    Ok(Poll::Ready(inv))
                } else {
                    bail!("Invalid response code {}", r.status);
                }
            }
            Poll::Ready(Err(e)) => Err(anyhow!("{}", e)),
            Poll::Pending => Ok(Poll::Pending),
        }
    }

    fn render_input(
        &self,
        ui: &mut Ui,
        services: &mut RouteServices,
        pubkey: &[u8; 32],
        service: &PayResponse,
    ) {
        let target_name = match self.target {
            ZapTarget::PublicKey { pubkey } => services.profile(&pubkey).and_then(|p| p.name()),
            ZapTarget::Event { event } => {
                let host = event.host();
                services.profile(host).and_then(|p| p.name())
            }
        };
        let fallback_name = self.target.to_string();
        let target_name = target_name.unwrap_or(&fallback_name);
        ui.label(RichText::new(format!("Zap {}", target_name)).size(FONT_SIZE_LG));

        ui.label("Zap amount in sats");

        // amount buttons
        const SATS_AMOUNTS: &[u64] = &[
            21, 69, 121, 420, 1_000, 2_100, 4_200, 10_000, 21_000, 42_000, 69_000, 100_000,
            210_000, 500_000, 1_000_000,
        ];
        const COLS: u32 = 5;
        let selected_amount = services.get("zap_amount").unwrap_or(0);
        Grid::new("zap_amounts_grid").show(ui, |ui| {
            let mut ctr = 0;
            for x in SATS_AMOUNTS {
                if Button::new()
                    .with_color(if selected_amount == *x {
                        NEUTRAL_700
                    } else {
                        NEUTRAL_800
                    })
                    .text(ui, &format_sats(*x as f32))
                    .clicked()
                {
                    services.set("zap_amount", *x);
                }
                ctr += 1;
                if ctr % COLS == 0 {
                    ui.end_row();
                }
            }
        });

        // comment section
        let mut zap_comment = services.get("zap_comment").unwrap_or(String::new());
        ui.label(format!("Your comment for {}", target_name));
        let old_len = zap_comment.len();
        NativeTextInput::new(&mut zap_comment)
            .with_frame(true)
            .ui(ui);

        if Button::new().with_color(PRIMARY).text(ui, "Zap!").clicked() {
            // on-click setup callback URL and transition state
            match self.zap_callback(
                services,
                pubkey,
                &zap_comment,
                selected_amount * 1_000,
                &service,
            ) {
                Ok(callback) => services.set(
                    "zap_state",
                    ZapState::FetchingInvoice {
                        callback: callback.to_string(),
                    },
                ),
                Err(e) => services.set("zap_state", ZapState::Error(e.to_string())),
            }
        }

        if zap_comment.len() != old_len {
            services.set("zap_comment", zap_comment);
        }
    }

    fn zap_callback(
        &self,
        services: &mut RouteServices,
        pubkey: &[u8; 32],
        zap_comment: &str,
        amount: u64,
        lnurlp: &PayResponse,
    ) -> anyhow::Result<Url> {
        let relays: Vec<Url> = services
            .ctx
            .pool
            .relays
            .iter()
            .filter_map(|r| match r {
                PoolRelay::Websocket(w) => Url::parse(&w.relay.url).ok(),
                _ => None,
            })
            .collect();
        if relays.is_empty() {
            bail!("No relays found");
        }
        let mut req = ZapRequestData::new(PublicKey::from_slice(pubkey)?, relays)
            .message(zap_comment)
            .amount(amount);
        match &self.target {
            ZapTarget::Event { event } => {
                req.event_coordinate = Some(
                    NostrLink::from_note(event)
                        .try_into()
                        .map_err(|e| anyhow!("{:?}", e))?,
                )
            }
            _ => {}
        };

        let req_tags: Vec<Tag> = req.into();
        let keys = if let Some(k) = services.current_account_keys() {
            k
        } else {
            bail!("Not logged in")
        };

        let req_ev = EventBuilder::new(Kind::ZapRequest, zap_comment)
            .tags(req_tags)
            .sign_with_keys(&keys)?;

        let mut url = Url::parse(&lnurlp.callback)?;
        url.query_pairs_mut()
            .append_pair("amount", amount.to_string().as_str());
        if lnurlp.nostr_pubkey.is_some() {
            url.query_pairs_mut()
                .append_pair("nostr", req_ev.as_json().as_str());
        }
        Ok(url)
    }
}
