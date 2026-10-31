use crate::route::{RouteServices, RouteType};
use crate::theme::{self, MUTED, SIDEBAR_WIDTH, SURFACE, TEXT};
use egui::{CursorIcon, Frame, Margin, Response, Sense, Ui, Vec2};

struct Item {
    label: &'static str,
    icon: &'static [u8],
    active: bool,
    route: Option<RouteType>,
}

pub struct Sidebar {
    pub active_home: bool,
}

impl Sidebar {
    pub fn render(&self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) {
        ui.vertical(|ui| {
            ui.set_width(SIDEBAR_WIDTH - 24.0);
            ui.spacing_mut().item_spacing.y = 4.0;
            self.rows(ui, services);
        });
    }

    fn rows(&self, ui: &mut Ui, services: &mut RouteServices<'_, '_>) {
        let items = [
            Item {
                label: "Streams",
                icon: include_bytes!("../resources/icons/signal.svg"),
                active: self.active_home,
                route: Some(RouteType::HomePage),
            },
            Item {
                label: "Videos",
                icon: include_bytes!("../resources/icons/play.svg"),
                active: false,
                route: None,
            },
            Item {
                label: "Shorts",
                icon: include_bytes!("../resources/icons/film.svg"),
                active: false,
                route: None,
            },
            Item {
                label: "Categories",
                icon: include_bytes!("../resources/icons/grid.svg"),
                active: false,
                route: None,
            },
        ];
        for item in items {
            if Self::row(ui, &item).clicked() {
                if let Some(route) = item.route {
                    services.navigate(route);
                }
            }
        }
    }

    fn row(ui: &mut Ui, item: &Item) -> Response {
        let color = if item.active { TEXT } else { MUTED };
        let r = Frame::new()
            .fill(if item.active {
                SURFACE
            } else {
                egui::Color32::TRANSPARENT
            })
            .corner_radius(8)
            .inner_margin(Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(10., 0.);
                    let uri = format!("bytes://nav-{}-{}.svg", item.label, item.active);
                    let svg = String::from_utf8_lossy(item.icon)
                        .replace("currentColor", &format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b()));
                    ui.add(
                        egui::Image::from_bytes(uri, svg.into_bytes())
                            .fit_to_exact_size(Vec2::splat(20.0)),
                    );
                    ui.label(theme::medium(item.label, 16.0).color(color));
                });
            });
        r.response
            .interact(Sense::click())
            .on_hover_and_drag_cursor(CursorIcon::PointingHand)
    }
}
