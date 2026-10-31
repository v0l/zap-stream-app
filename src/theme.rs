use egui::epaint::{FontFamily, FontId};
use egui::{Color32, Margin, Stroke, TextStyle};
use std::sync::Arc;

pub const FONT_SIZE: f32 = 14.5;
pub const FONT_SIZE_SM: f32 = 13.0;
pub const FONT_SIZE_LG: f32 = 19.0;
pub const ROUNDING_DEFAULT: f32 = 12.0;
pub const ROUNDING_LG: f32 = 16.0;
pub const MARGIN_DEFAULT: Margin = Margin::symmetric(12, 6);

pub const BG: Color32 = Color32::from_rgb(13, 12, 15);
pub const SURFACE: Color32 = Color32::from_rgb(32, 29, 38);
pub const SURFACE_LOW: Color32 = Color32::from_rgb(22, 20, 27);
pub const BORDER: Color32 = Color32::from_rgb(44, 41, 51);
pub const TEXT: Color32 = Color32::from_rgb(242, 238, 232);
pub const TEXT_SOFT: Color32 = Color32::from_rgb(201, 196, 204);
pub const MUTED: Color32 = Color32::from_rgb(154, 149, 163);
pub const PRIMARY: Color32 = Color32::from_rgb(255, 176, 0);
pub const ON_PRIMARY: Color32 = Color32::from_rgb(26, 17, 0);
pub const ZAP: Color32 = PRIMARY;

pub const NEUTRAL_500: Color32 = MUTED;
pub const NEUTRAL_700: Color32 = BORDER;
pub const NEUTRAL_800: Color32 = SURFACE;
pub const NEUTRAL_900: Color32 = SURFACE_LOW;

pub const SIDEBAR_WIDTH: f32 = 208.0;
pub const HEADER_HEIGHT: f32 = 56.0;
pub const WIDE_BREAKPOINT: f32 = 720.0;

pub fn display_family() -> FontFamily {
    FontFamily::Name("display".into())
}

pub fn display_bold() -> FontFamily {
    FontFamily::Name("display-bold".into())
}

pub fn medium_family() -> FontFamily {
    FontFamily::Name("medium".into())
}

pub fn semibold_family() -> FontFamily {
    FontFamily::Name("semibold".into())
}

pub fn install(ctx: &egui::Context) {
    install_fonts(ctx);
    apply_style(ctx);
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut fd = egui::FontDefinitions::default();
    let load = |fd: &mut egui::FontDefinitions, name: &str, bytes: &'static [u8]| {
        fd.font_data
            .insert(name.to_string(), Arc::new(egui::FontData::from_static(bytes)));
    };
    load(
        &mut fd,
        "InstrumentSans-Regular",
        include_bytes!("../assets/fonts/InstrumentSans-Regular.ttf"),
    );
    load(
        &mut fd,
        "InstrumentSans-Medium",
        include_bytes!("../assets/fonts/InstrumentSans-Medium.ttf"),
    );
    load(
        &mut fd,
        "InstrumentSans-SemiBold",
        include_bytes!("../assets/fonts/InstrumentSans-SemiBold.ttf"),
    );
    load(
        &mut fd,
        "Bricolage-Bold",
        include_bytes!("../assets/fonts/Bricolage-Bold.ttf"),
    );
    load(
        &mut fd,
        "Bricolage-ExtraBold",
        include_bytes!("../assets/fonts/Bricolage-ExtraBold.ttf"),
    );

    let fallback = fd
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let with_fallback = |first: &str| {
        let mut v = vec![first.to_string()];
        v.extend(fallback.iter().cloned());
        v
    };
    fd.families
        .insert(FontFamily::Proportional, with_fallback("InstrumentSans-Regular"));
    fd.families
        .insert(medium_family(), with_fallback("InstrumentSans-Medium"));
    fd.families
        .insert(semibold_family(), with_fallback("InstrumentSans-SemiBold"));
    fd.families
        .insert(display_bold(), with_fallback("Bricolage-Bold"));
    fd.families
        .insert(display_family(), with_fallback("Bricolage-ExtraBold"));
    ctx.set_fonts(fd);
}

pub fn apply_style(ctx: &egui::Context) {
    ctx.set_theme(egui::ThemePreference::Dark);
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(FONT_SIZE, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(FONT_SIZE, medium_family()),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(FONT_SIZE_SM, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(30.72, display_family()),
        );
        let v = &mut style.visuals;
        v.panel_fill = BG;
        v.window_fill = SURFACE_LOW;
        v.window_stroke = Stroke::new(1.0, BORDER);
        v.extreme_bg_color = SURFACE_LOW;
        v.faint_bg_color = SURFACE_LOW;
        v.override_text_color = Some(TEXT);
        v.hyperlink_color = PRIMARY;
        v.selection.bg_fill = PRIMARY.gamma_multiply(0.35);
        v.selection.stroke = Stroke::new(1.0, PRIMARY);
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
        v.widgets.inactive.bg_fill = SURFACE;
        v.widgets.inactive.weak_bg_fill = SURFACE;
        v.widgets.hovered.bg_fill = BORDER;
        v.widgets.hovered.weak_bg_fill = BORDER;
    });
}

pub fn display(text: impl Into<String>, size: f32) -> egui::RichText {
    egui::RichText::new(text).font(FontId::new(size, display_family())).color(TEXT)
}

pub fn display_bold_text(text: impl Into<String>, size: f32) -> egui::RichText {
    egui::RichText::new(text).font(FontId::new(size, display_bold())).color(TEXT)
}

pub fn medium(text: impl Into<String>, size: f32) -> egui::RichText {
    egui::RichText::new(text).font(FontId::new(size, medium_family()))
}

pub fn semibold(text: impl Into<String>, size: f32) -> egui::RichText {
    egui::RichText::new(text).font(FontId::new(size, semibold_family()))
}
