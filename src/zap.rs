use crate::note_util::NoteUtil;
use anyhow::{anyhow, bail, Result};
use nostr::{Event, JsonUtil, Kind, TagStandard};
use nostrdb::Note;

pub struct Zap<'a> {
    pub sender: [u8; 32],
    pub receiver: [u8; 32],
    pub zapper_service: &'a [u8; 32],
    pub amount: u64,
    pub message: String,
}

impl<'a> Zap<'a> {
    pub fn from_receipt(event: Note<'a>) -> Result<Zap> {
        if event.kind() != 9735 {
            bail!("not a zap receipt");
        }

        let req_json = event
            .get_tag_value("description")
            .ok_or(anyhow!("missing description"))?;
        let req = Event::from_json(
            req_json
                .variant()
                .str()
                .ok_or(anyhow!("empty description"))?,
        )?;

        if req.kind != Kind::ZapRequest {
            bail!("not a zap request");
        }

        let dest = req
            .tags
            .iter()
            .find_map(|t| match t.as_standardized() {
                Some(TagStandard::PublicKey { public_key, .. }) => Some(public_key.to_bytes()),
                _ => None,
            })
            .ok_or(anyhow!("missing p tag in zap request"))?;

        let amount = req.tags.iter().find_map(|t| match t.as_standardized() {
            Some(TagStandard::Amount { millisats, .. }) => Some(*millisats),
            _ => None,
        });

        Ok(Zap {
            sender: req.pubkey.to_bytes(),
            receiver: dest,
            zapper_service: event.pubkey(),
            amount: amount.unwrap_or(0u64),
            message: req.content,
        })
    }
}
