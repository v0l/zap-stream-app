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
use egui::{vec2, Frame, Grid, Response, RichText, TextFormat, TextWrapMode, Ui, Widget};
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
        let resp = Button::pill().text(ui, "Zap");
        let open_id = egui::Id::new("zap_modal_open");
        if resp.clicked() {
            ui.ctx().data_mut(|d| d.insert_temp(open_id, true));
        }
        let open = ui.ctx().data(|d| d.get_temp::<bool>(open_id)).unwrap_or(false);
        if !open {
            return resp;
        }

        let modal = egui::Modal::new(egui::Id::new("zapper")).frame(
            Frame::new()
                .fill(crate::theme::SURFACE_LOW)
                .stroke(egui::Stroke::new(1.0, crate::theme::SURFACE))
                .corner_radius(16)
                .inner_margin(egui::Margin::same(24)),
        );
        let modal_resp = modal.show(ui.ctx(), |ui| {
            ui.set_width(360.0);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);

            let pubkey = match &self.target {
                ZapTarget::PublicKey { pubkey } => pubkey,
                ZapTarget::Event { event } => event.host(),
            };

            let zap_state = services.get("zap_state").unwrap_or(ZapState::NotStarted);
            match &zap_state {
                ZapState::NotStarted => match services.fetch_lnurlp(pubkey) {
                    Ok(Poll::Ready(r)) => services.set("zap_state", ZapState::Ready { service: r }),
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

            match &zap_state {
                ZapState::Ready { service } => {
                    self.render_input(ui, services, pubkey, service);
                }
                ZapState::Invoice { invoice } => {
                    ui.vertical_centered(|ui| {
                        qr_code(ui, invoice.pr.as_bytes(), 256.0);

                        let mut job = LayoutJob::default();
                        job.wrap = TextWrapping::from_wrap_mode_and_width(
                            TextWrapMode::Truncate,
                            ui.available_width(),
                        );
                        job.append(&invoice.pr, 0.0, TextFormat::default());
                        ui.label(job);
                    });
                }
                ZapState::Error(e) => {
                    ui.label(e);
                }
                _ => {}
            }
        });

        if modal_resp.should_close() {
            services.set("zap_state", ZapState::NotStarted);
            ui.ctx().data_mut(|d| d.insert_temp(open_id, false));
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
        ui.label(crate::theme::display_bold_text(format!("Zap {}", target_name), 26.0));

        ui.label(crate::theme::medium("Zap amount in sats", 14.0).color(crate::theme::MUTED));

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
                        PRIMARY
                    } else {
                        NEUTRAL_800
                    })
                    .padding(egui::Margin::symmetric(10, 8))
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
        ui.label(crate::theme::medium(format!("Your comment for {}", target_name), 14.0).color(crate::theme::MUTED));
        let old_len = zap_comment.len();
        NativeTextInput::new(&mut zap_comment)
            .with_frame(true)
            .ui(ui);

        if Button::primary().radius(255).padding(egui::Margin::symmetric(24, 10)).text(ui, "Zap").clicked() {
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
            .accounts
            .selected_account_write_relays()
            .into_iter()
            .filter_map(|r| match r {
                enostr::RelayId::Websocket(u) => Url::parse(u.as_str()).ok(),
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

fn qr_code(ui: &mut Ui, data: &[u8], size: f32) {
    let Ok(code) = qrcode::QrCode::new(data) else {
        return;
    };
    let n = code.width();
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, egui::Color32::WHITE);
    let quiet = 2.0;
    let cell = size / (n as f32 + quiet * 2.0);
    for (i, c) in code.to_colors().iter().enumerate() {
        if *c == qrcode::Color::Dark {
            let x = (i % n) as f32;
            let y = (i / n) as f32;
            let min = rect.min + vec2((x + quiet) * cell, (y + quiet) * cell);
            painter.rect_filled(
                egui::Rect::from_min_size(min, vec2(cell, cell)),
                0.0,
                egui::Color32::BLACK,
            );
        }
    }
}
