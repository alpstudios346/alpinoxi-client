//! Instances: named, versioned setups. One list, one editor, no hidden state.

use eframe::egui::{self, RichText, Vec2};

use alpinoxi_core::launcher::MemoryProfile;

use crate::app::AlpinoxiApp;
use crate::theme;
use crate::ui;

pub fn show(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::eyebrow(ui, "Instances");
    ui::heading(ui, "Your worlds, separated");
    ui.label(
        RichText::new("Each instance keeps its own mods, resource packs, HUD layout and RAM plan.")
            .size(13.0)
            .color(theme::TEXT_DIM),
    );
    ui.add_space(16.0);

    ui::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui::eyebrow(ui, "Name");
                ui.add(
                    egui::TextEdit::singleline(&mut app.settings.instance.name)
                        .desired_width(240.0)
                        .hint_text("Vanilla"),
                );
            });
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui::eyebrow(ui, "Game version");
                ui.add(
                    egui::TextEdit::singleline(&mut app.settings.instance.game_version)
                        .desired_width(120.0)
                        .hint_text("1.21.1"),
                );
            });
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui::eyebrow(ui, "Loader");
                egui::ComboBox::from_id_salt("loader")
                    .selected_text(app.settings.instance.loader.clone())
                    .show_ui(ui, |ui| {
                        for loader in ["fabric", "quilt", "neoforge", "vanilla"] {
                            ui.selectable_value(
                                &mut app.settings.instance.loader,
                                loader.to_owned(),
                                loader,
                            );
                        }
                    });
            });
        });
        ui.add_space(12.0);
        ui::separator(ui);
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui::eyebrow(ui, "RAM");
            for (profile, title) in [
                (MemoryProfile::Balanced, "Balanced · 2 GB"),
                (MemoryProfile::Performance, "Performance · 4 GB"),
            ] {
                if ui
                    .add(egui::Button::new(title).selected(app.settings.memory == profile))
                    .clicked()
                {
                    app.set_memory(profile);
                }
            }
            ui.label(
                RichText::new(format!("→ {} MiB", app.launch_plan().mebibytes))
                    .size(12.0)
                    .color(theme::ACCENT),
            );
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("JVM arguments")
                    .size(12.0)
                    .color(theme::TEXT_DIM),
            );
            let args = app.launch_plan().arguments().join(" ");
            ui.add(
                egui::Label::new(
                    RichText::new(args)
                        .monospace()
                        .size(11.0)
                        .color(theme::TEXT_DIM),
                )
                .truncate(),
            );
        });
    });

    ui.add_space(16.0);
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "Content");
        ui.horizontal_wrapped(|ui| {
            for id in app.settings.instance.enabled_mods.clone() {
                let name = app
                    .catalog
                    .iter()
                    .find(|entry| entry.id == id)
                    .map(|entry| entry.name)
                    .unwrap_or(id.as_str());
                ui::pill(ui, name, theme::MINT);
            }
            if app.settings.instance.enabled_mods.is_empty() {
                ui.label(
                    RichText::new("No mods enabled. Add them from Mods & packs.")
                        .size(12.0)
                        .color(theme::TEXT_DIM),
                );
            }
        });
        ui.add_space(10.0);
        if ui::ghost_button(ui, "Open Mods & packs", Vec2::new(160.0, 32.0)).clicked() {
            app.screen = crate::app::Screen::Mods;
        }
    });
}
