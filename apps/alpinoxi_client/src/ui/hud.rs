//! HUD settings: every element is a switch, every position is a choice.
//!
//! This screen is the launcher half of the command-free promise. The in-game
//! HUD editor edits the same struct, so a layout arranged here is the layout the
//! player sees in game.

use eframe::egui::{self, CornerRadius, RichText};

use alpinoxi_core::hud::{HudAnchor, HudElement};

use crate::app::AlpinoxiApp;
use crate::theme;
use crate::ui;

pub fn show(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::eyebrow(ui, "HUD");
    ui::heading(ui, "Everything by keybind");
    ui.label(
        RichText::new("Open the in-game editor with Shift+H. No /hud commands, ever.")
            .size(13.0)
            .color(theme::TEXT_DIM),
    );
    ui.add_space(14.0);

    ui.horizontal(|ui| {
        ui::setting_row(ui, "HUD", "Master switch for every element", |ui| {
            ui.checkbox(&mut app.settings.hud.enabled, "");
        });
    });
    ui.horizontal(|ui| {
        let mut scale = app.settings.hud.scale_percent as u32;
        if ui
            .add(
                egui::Slider::new(&mut scale, 50..=200)
                    .suffix("%")
                    .text("Scale"),
            )
            .changed()
        {
            app.settings.hud.scale_percent = scale as u8;
            app.settings.hud.normalise();
        }
        ui.checkbox(&mut app.settings.hud.background, "Background");
    });

    ui.add_space(14.0);
    ui::separator(ui);
    ui.add_space(12.0);

    egui::Grid::new("hud-elements")
        .num_columns(4)
        .spacing([16.0, 8.0])
        .striped(false)
        .show(ui, |ui| {
            ui::eyebrow(ui, "Element");
            ui::eyebrow(ui, "Visible");
            ui::eyebrow(ui, "Anchor");
            ui::eyebrow(ui, "Offset");
            ui.end_row();

            for element in HudElement::ALL {
                ui.label(RichText::new(element.label()).color(theme::TEXT));
                let mut visible = app.settings.hud.is_visible(element);
                if ui.checkbox(&mut visible, "").changed() {
                    app.settings.hud.set_visible(element, visible);
                }
                let current = app.settings.hud.placement(element).anchor;
                egui::ComboBox::from_id_salt(("anchor", element.label()))
                    .selected_text(current.label())
                    .show_ui(ui, |ui| {
                        for anchor in HudAnchor::ALL {
                            if ui
                                .selectable_label(anchor == current, anchor.label())
                                .clicked()
                            {
                                app.settings.hud.set_anchor(element, anchor);
                            }
                        }
                    });
                let placement = &mut app.settings.hud.placements[element.index()];
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut placement.offset.0)
                            .speed(1.0)
                            .prefix("x "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut placement.offset.1)
                            .speed(1.0)
                            .prefix("y "),
                    );
                });
                ui.end_row();
            }
        });

    ui.add_space(16.0);
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "Live layout");
        ui.label(
            RichText::new("A miniature of where the elements sit, drawn from the same values.")
                .size(11.5)
                .color(theme::TEXT_DIM),
        );
        ui.add_space(8.0);
        let size = egui::Vec2::new(ui.available_width(), 220.0);
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(
            rect,
            CornerRadius::same(10.0),
            egui::Color32::from_rgb(9, 18, 34),
        );
        for element in HudElement::ALL {
            if !app.settings.hud.is_visible(element) {
                continue;
            }
            let placement = app.settings.hud.placement(element);
            let (ax, ay) = placement.anchor.origin();
            let position = egui::pos2(
                rect.left() + rect.width() * ax,
                rect.top() + rect.height() * ay,
            );
            let anchor_align = match placement.anchor {
                HudAnchor::TopLeft | HudAnchor::BottomLeft => egui::Align2::LEFT_TOP,
                HudAnchor::TopRight | HudAnchor::BottomRight => egui::Align2::RIGHT_TOP,
                HudAnchor::BottomCentre => egui::Align2::CENTER_BOTTOM,
            };
            painter.text(
                position
                    + egui::vec2(
                        placement.offset.0 as f32 * 0.25,
                        placement.offset.1 as f32 * 0.25,
                    ),
                anchor_align,
                element.label(),
                egui::FontId::proportional(11.0),
                theme::ACCENT,
            );
        }
    });
}
