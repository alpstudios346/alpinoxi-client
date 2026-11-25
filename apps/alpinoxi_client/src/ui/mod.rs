//! Screens. Each one is a function over `AlpinoxiApp`, so state lives in one
//! place and screens stay short.

pub mod friends;
pub mod home;
pub mod hud;
pub mod import;
pub mod instances;
pub mod keybinds;
pub mod login;
pub mod mods;
pub mod settings;
pub mod shell;
pub mod skin;

use eframe::egui::{self, Color32, CornerRadius, Response, RichText, Sense, Stroke, Vec2};

use crate::theme;

/// A framed card with the Alpinoxi surface colour. Returns the inner rect so the
/// caller can lay out content, or use `card` for the closure form.
pub fn card(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    let width = ui.available_width();
    egui::Frame::new()
        .fill(theme::BG_CARD)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(CornerRadius::same(theme::RADIUS_U8))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(width - 34.0);
            add(ui);
        });
}

/// A section label: small, wide-tracked, dim.
pub fn eyebrow(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(11.5)
            .color(theme::TEXT_DIM)
            .strong(),
    );
}

pub fn heading(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).size(24.0).color(theme::TEXT).strong());
}

/// Primary action button, filled with the accent colour.
pub fn primary_button(ui: &mut egui::Ui, text: &str, size: Vec2) -> Response {
    let button = egui::Button::new(
        RichText::new(text)
            .color(Color32::from_rgb(6, 14, 26))
            .strong(),
    )
    .fill(theme::ACCENT)
    .corner_radius(CornerRadius::same(10.0 as u8));
    ui.add_sized(size, button)
}

/// Quiet, outlined button.
pub fn ghost_button(ui: &mut egui::Ui, text: &str, size: Vec2) -> Response {
    let button = egui::Button::new(RichText::new(text).color(theme::TEXT))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(CornerRadius::same(10.0 as u8));
    ui.add_sized(size, button)
}

/// A pill: rounded label with a tinted background.
pub fn pill(ui: &mut egui::Ui, text: &str, colour: Color32) {
    let text = RichText::new(text).size(11.5).color(colour).strong();
    let galley = ui.painter().layout_no_wrap(
        text.text().to_owned(),
        egui::FontId::proportional(11.5),
        colour,
    );
    let padding = Vec2::new(8.0, 3.0);
    let (rect, _) = ui.allocate_exact_size(galley.size() + padding * 2.0, Sense::hover());
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(7.0 as u8),
        Color32::from_rgba_unmultiplied(colour.r(), colour.g(), colour.b(), 34),
    );
    ui.painter().galley(rect.min + padding, galley, colour);
}

/// A labelled row: dim caption on the left, control on the right.
pub fn setting_row(ui: &mut egui::Ui, label: &str, hint: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(label).color(theme::TEXT));
            if !hint.is_empty() {
                ui.label(RichText::new(hint).size(12.0).color(theme::TEXT_DIM));
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add);
    });
}

/// Horizontal rule that matches the card borders.
pub fn separator(ui: &mut egui::Ui) {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 1.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, theme::BORDER);
}

/// Empty-state block: icon-free, one line plus a hint.
pub fn empty_state(ui: &mut egui::Ui, title: &str, hint: &str) {
    ui.add_space(18.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(title).size(16.0).color(theme::TEXT).strong());
        ui.label(RichText::new(hint).size(13.0).color(theme::TEXT_DIM));
    });
    ui.add_space(18.0);
}
