//! The Alpinoxi look: deep navy glass, cyan accents, one clear type scale.
//!
//! Kept in one place so every screen shares the same spacing and colour tokens
//! instead of each panel inventing its own.

use eframe::egui::{self, Color32, FontFamily, FontId, CornerRadius, Stroke, TextStyle};

pub const BG_DEEP: Color32 = Color32::from_rgb(7, 14, 30);
pub const BG_PANEL: Color32 = Color32::from_rgb(12, 22, 43);
pub const BG_CARD: Color32 = Color32::from_rgb(17, 30, 55);
pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(23, 40, 72);
pub const ACCENT: Color32 = Color32::from_rgb(79, 209, 255);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(38, 128, 190);
pub const MINT: Color32 = Color32::from_rgb(94, 234, 178);
pub const WARN: Color32 = Color32::from_rgb(255, 196, 92);
pub const DANGER: Color32 = Color32::from_rgb(255, 107, 129);
pub const TEXT: Color32 = Color32::from_rgb(226, 236, 250);
pub const TEXT_DIM: Color32 = Color32::from_rgb(148, 168, 198);
pub const BORDER: Color32 = Color32::from_rgb(30, 50, 84);

pub const RADIUS: f32 = 12.0;

/// Provider badge colours, so Microsoft / Ely.by / Cracked are recognisable at a
/// glance in the login cards, the friend list and the account chip.
pub const MS_COLOR: Color32 = Color32::from_rgb(108, 190, 120);
pub const ELY_COLOR: Color32 = Color32::from_rgb(168, 126, 255);
pub const CRACK_COLOR: Color32 = Color32::from_rgb(255, 158, 92);

pub fn install(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = BG_DEEP;
    visuals.panel_fill = BG_PANEL;
    visuals.extreme_bg_color = Color32::from_rgb(9, 17, 33);
    visuals.faint_bg_color = BG_CARD;
    visuals.override_text_color = Some(TEXT);
    visuals.hyperlink_color = ACCENT;
    visuals.selection.bg_fill = ACCENT_SOFT;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_corner_radius = CornerRadius::same(RADIUS);

    let widgets = &mut visuals.widgets;
    widgets.noninteractive.bg_fill = BG_CARD;
    widgets.noninteractive.weak_bg_fill = BG_CARD;
    widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS);

    widgets.inactive.bg_fill = BG_CARD;
    widgets.inactive.weak_bg_fill = BG_CARD;
    widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    widgets.inactive.corner_radius = CornerRadius::same(RADIUS);

    widgets.hovered.bg_fill = BG_CARD_HOVER;
    widgets.hovered.weak_bg_fill = BG_CARD_HOVER;
    widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT_SOFT);
    widgets.hovered.fg_stroke = Stroke::new(1.4, Color32::WHITE);
    widgets.hovered.corner_radius = CornerRadius::same(RADIUS);

    widgets.active.bg_fill = ACCENT_SOFT;
    widgets.active.weak_bg_fill = ACCENT_SOFT;
    widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    widgets.active.fg_stroke = Stroke::new(1.6, Color32::WHITE);
    widgets.active.corner_radius = CornerRadius::same(RADIUS);

    widgets.open.bg_fill = BG_CARD_HOVER;
    widgets.open.weak_bg_fill = BG_CARD_HOVER;
    widgets.open.bg_stroke = Stroke::new(1.0, ACCENT_SOFT);
    widgets.open.corner_radius = CornerRadius::same(RADIUS);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.interact_size.y = 34.0;
    style.spacing.scroll.bar_width = 8.0;
    style.text_styles = [
        (TextStyle::Heading, FontId::new(26.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(15.0, FontFamily::Proportional)),
        (TextStyle::Button, FontId::new(15.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(12.5, FontFamily::Proportional)),
        (
            TextStyle::Monospace,
            FontId::new(13.0, FontFamily::Monospace),
        ),
    ]
    .into();
    ctx.set_style(style);
}

/// Colour for a provider badge.
pub fn provider_color(provider: alpinoxi_core::AccountProvider) -> Color32 {
    match provider {
        alpinoxi_core::AccountProvider::Microsoft => MS_COLOR,
        alpinoxi_core::AccountProvider::ElyBy => ELY_COLOR,
        alpinoxi_core::AccountProvider::Offline => CRACK_COLOR,
    }
}
