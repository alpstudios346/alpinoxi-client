//! The wardrobe: skin preview, model switch and the local file picker.
//!
//! Deliberately command-free and quiet: one large preview, two model options,
//! and a note that always tells the player who can actually see the skin.

use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, Vec2};

use alpinoxi_core::skin::{SkinModel, SkinSource};

use crate::app::AlpinoxiApp;
use crate::theme;
use crate::ui;

pub fn show(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    app.ensure_skin_texture(ui.ctx());

    ui::eyebrow(ui, "Wardrobe");
    ui::heading(ui, "Your skin, no commands");
    ui.label(
        RichText::new("Pick a model, drop in a PNG, done. Everything is a button here.")
            .size(13.0)
            .color(theme::TEXT_DIM),
    );
    ui.add_space(16.0);

    ui.columns(2, |columns| {
        preview_card(&mut columns[0], app);
        options_card(&mut columns[1], app);
    });
}

fn preview_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "Preview");
        ui.add_space(6.0);
        let size = Vec2::new(ui.available_width(), 320.0);
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(
            rect,
            CornerRadius::same(14.0 as u8),
            Color32::from_rgb(9, 18, 34),
        );
        let checker = 12.0;
        let mut y = rect.top();
        let mut row = 0;
        while y < rect.bottom() {
            let mut x = rect.left();
            let mut column = 0;
            while x < rect.right() {
                if (row + column) % 2 == 0 {
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(x, y),
                            Vec2::new(
                                checker.min(rect.right() - x),
                                checker.min(rect.bottom() - y),
                            ),
                        ),
                        0,
                        Color32::from_rgb(13, 24, 43),
                    );
                }
                x += checker;
                column += 1;
            }
            y += checker;
            row += 1;
        }

        match &app.skin_texture {
            Some(texture) => {
                let image_size = Vec2::new(160.0, 320.0);
                painter.image(
                    texture.id(),
                    egui::Rect::from_center_size(rect.center(), image_size),
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            None => {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No local skin loaded",
                    egui::FontId::proportional(14.0),
                    theme::TEXT_DIM,
                );
            }
        }

        ui.add_space(10.0);
        if let Some(skin) = app.wardrobe.skin.clone() {
            ui::pill(ui, skin.model.label(), theme::ACCENT);
            ui::pill(ui, skin.source.label(), theme::ACCENT_SOFT);
            if skin.source != SkinSource::Local {
                ui.label(
                    RichText::new(skin.location.clone())
                        .size(10.5)
                        .color(theme::TEXT_DIM),
                );
            }
        }
    });
}

fn options_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "Model");
        ui.horizontal(|ui| {
            for model in [SkinModel::Classic, SkinModel::Slim] {
                let label = format!(
                    "{}\n{}",
                    model.label(),
                    match model {
                        SkinModel::Classic => "4px arms · Steve",
                        SkinModel::Slim => "3px arms · Alex",
                    }
                );
                if ui
                    .add_sized(
                        [ui.available_width() / 2.0 - 4.0, 54.0],
                        egui::Button::new(RichText::new(label).color(theme::TEXT))
                            .selected(app.skin_model == model),
                    )
                    .clicked()
                {
                    app.set_skin_model(model);
                }
            }
        });

        ui.add_space(12.0);
        ui::separator(ui);
        ui.add_space(10.0);
        ui::eyebrow(ui, "Local file");
        ui.label(
            RichText::new(
                "64x64 or 64x32 PNG. Alpinoxi checks the size before it accepts the file.",
            )
            .size(12.0)
            .color(theme::TEXT_DIM),
        );
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut app.skin_path)
                    .hint_text("C:/Users/you/skin.png")
                    .desired_width(ui.available_width() - 110.0),
            );
            if ui::primary_button(ui, "Apply", Vec2::new(96.0, 32.0)).clicked() {
                let path = app.skin_path.clone();
                app.remember_skin_path(&path);
                app.load_local_skin(&path);
            }
        });

        ui.add_space(12.0);
        ui::separator(ui);
        ui.add_space(10.0);
        ui::eyebrow(ui, "Who can see it");
        let note = app
            .wardrobe
            .skin
            .as_ref()
            .map(|skin| skin.visibility_note())
            .unwrap_or("Sign in or load a PNG first.");
        egui::Frame::new()
            .fill(theme::BG_PANEL)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(CornerRadius::same(10.0 as u8))
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(note).size(12.0).color(theme::TEXT));
            });

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui::eyebrow(ui, "Upload target");
            ui.label(
                RichText::new(match app.account.as_ref().map(|account| account.provider) {
                    Some(alpinoxi_core::AccountProvider::ElyBy) => "account.ely.by",
                    Some(alpinoxi_core::AccountProvider::Microsoft) => "api.minecraftservices.com",
                    _ => "local only",
                })
                .size(12.0)
                .color(theme::TEXT_DIM),
            );
        });
    });
}
