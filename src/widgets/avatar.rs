use crate::route::image_from_cache;
use crate::theme::NEUTRAL_800;
use egui::{vec2, Response, Sense, Ui, Vec2, Widget};
use nostrdb::{Ndb, NdbProfile, Transaction};
use notedeck::AppContext;

pub struct Avatar {
    image: Option<String>,
    size: Option<f32>,
    ring: Option<egui::Color32>,
}

impl Avatar {
    pub fn new_optional(img: Option<&str>) -> Self {
        Self {
            image: img.map(String::from),
            size: None,
            ring: None,
        }
    }

    pub fn pubkey(pk: &[u8; 32], ndb: &Ndb, tx: &Transaction) -> Self {
        let picture = ndb
            .get_profile_by_pubkey(tx, pk)
            .map(|p| p.record().profile().map(|p| p.picture()).unwrap_or(None))
            .unwrap_or(None);
        Self {
            image: picture.map(|s| s.to_string()),
            size: None,
            ring: None,
        }
    }

    pub fn from_profile(p: &Option<NdbProfile<'_>>) -> Self {
        let img = p.map(|f| f.picture()).unwrap_or(None);
        Self {
            image: img.map(String::from),
            size: None,
            ring: None,
        }
    }

    pub fn ring(mut self, color: egui::Color32) -> Self {
        self.ring = Some(color);
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    fn placeholder(ui: &mut Ui, size: f32) -> Response {
        let (response, painter) = ui.allocate_painter(vec2(size, size), Sense::click());
        let pos = response.rect.min + vec2(size / 2., size / 2.);
        painter.circle_filled(pos, size / 2., NEUTRAL_800);
        response
    }

    pub fn render(self, ui: &mut Ui, ctx: &mut AppContext<'_>) -> Response {
        let ring = self.ring;
        let r = self.render_inner(ui, ctx);
        if let Some(c) = ring {
            ui.painter().circle_stroke(
                r.rect.center(),
                r.rect.width() / 2.0 - 2.0,
                egui::Stroke::new(4.0, c),
            );
        }
        r
    }

    fn render_inner(self, ui: &mut Ui, ctx: &mut AppContext<'_>) -> Response {
        let size_v = self.size.unwrap_or(40.);
        let size = Vec2::new(size_v, size_v);
        if !ui.is_rect_visible(ui.cursor()) {
            return Self::placeholder(ui, size_v);
        }
        match self
            .image
            .and_then(|i| {
                image_from_cache(ctx.img_cache, ctx.media_jobs.sender(), ui, &i, Some(size))
            })
        {
            Some(img) => img
                .fit_to_exact_size(size)
                .corner_radius(size_v / 2.0)
                .sense(Sense::click())
                .ui(ui),
            None => Self::placeholder(ui, size_v),
        }
    }
}
