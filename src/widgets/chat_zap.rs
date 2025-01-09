use crate::theme::{MARGIN_DEFAULT, ROUNDING_DEFAULT, ZAP};
use crate::widgets::Avatar;
use crate::zap::Zap;
use eframe::emath::Align;
use eframe::epaint::text::{LayoutJob, TextFormat, TextWrapMode};
use eframe::epaint::Color32;
use egui::{Frame, Label, Response, Stroke, Ui};
use nostrdb::NdbProfile;
use notedeck::ImageCache;

pub struct ChatZap<'a> {
    zap: &'a Zap<'a>,
    profile: &'a Option<NdbProfile<'a>>,
}

impl<'a> ChatZap<'a> {
    pub fn new(zap: &'a Zap, profile: &'a Option<NdbProfile<'a>>) -> Self {
        Self { zap, profile }
    }

    pub fn render(self, ui: &mut Ui, img_cache: &mut ImageCache) -> Response {
        Frame::default()
            .rounding(ROUNDING_DEFAULT)
            .inner_margin(MARGIN_DEFAULT)
            .stroke(Stroke::new(1., ZAP))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let mut job = LayoutJob::default();
                    // TODO: avoid this somehow
                    job.wrap.break_anywhere = true;

                    let name = self
                        .profile
                        .map_or("Nostrich", |f| f.name().map_or("Nostrich", |f| f));

                    let mut format = TextFormat::default();
                    format.line_height = Some(24.0);
                    format.valign = Align::Center;

                    format.color = ZAP;
                    job.append(name, 0.0, format.clone());
                    format.color = Color32::WHITE;
                    job.append("zapped", 5.0, format.clone());
                    format.color = ZAP;
                    job.append(
                        (self.zap.amount / 1000).to_string().as_str(),
                        5.0,
                        format.clone(),
                    );
                    format.color = Color32::WHITE;
                    job.append("sats", 5.0, format.clone());

                    if !self.zap.message.is_empty() {
                        job.append(&format!("\n{}", self.zap.message), 0.0, format.clone());
                    }

                    Avatar::from_profile(&self.profile)
                        .size(24.)
                        .render(ui, img_cache);
                    ui.add(Label::new(job).wrap_mode(TextWrapMode::Wrap));

                    // consume reset of space
                    ui.add_space(ui.available_size_before_wrap().x);
                });
            })
            .response
    }
}
