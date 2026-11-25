//! Settings: account, performance, HUD and keybinds in one scroll, with the
//! facts a player needs to trust the client (what it uses, what it never does).

use eframe::egui::{self, RichText, Vec2};

use alpinoxi_core::launcher::{MemoryProfile, PERFORMANCE_MODS};

use crate::app::AlpinoxiApp;
use crate::theme;
use crate::ui;

pub fn show(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::eyebrow(ui, "Settings");
    ui::heading(ui, "The client, in your hands");
    ui.add_space(14.0);

    ui::card(ui, |ui| {
        ui::eyebrow(ui, "Performance");
        ui.label(
            RichText::new(
                "The launcher is native Rust: it draws one window and no web view, so it stays a few megabytes while Minecraft runs beside it.",
            )
            .size(12.5)
            .color(theme::TEXT_DIM),
        );
        ui.add_space(8.0);
        ui::setting_row(
            ui,
            "Heap for Minecraft",
            "Capped at half of system RAM",
            |ui| {
                for (profile, title) in [
                    (MemoryProfile::Balanced, "2 GB"),
                    (MemoryProfile::Performance, "4 GB"),
                ] {
                    if ui
                        .add(egui::Button::new(title).selected(app.settings.memory == profile))
                        .clicked()
                    {
                        app.set_memory(profile);
                    }
                }
            },
        );
        ui::setting_row(
            ui,
            "Keep launcher light",
            "No background scanning while you play",
            |ui| {
                ui.checkbox(&mut app.settings.keep_launcher_light, "");
            },
        );
        ui::setting_row(
            ui,
            "Close launcher on launch",
            "The game keeps running",
            |ui| {
                ui.checkbox(&mut app.settings.close_launcher_on_launch, "");
            },
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(format!(
                "Resolved: {} MiB · {} flags",
                app.launch_plan().mebibytes,
                app.launch_plan().jvm_args.len()
            ))
            .size(11.5)
            .color(theme::ACCENT_SOFT),
        );
        if let Some(warning) = app.memory_warning() {
            ui.label(RichText::new(warning).size(11.5).color(theme::WARN));
        }
    });

    ui.add_space(14.0);
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "HUD and keys");
        ui.horizontal(|ui| {
            if ui::ghost_button(ui, "HUD editor", Vec2::new(130.0, 34.0)).clicked() {
                app.screen = crate::app::Screen::Settings;
                app.notice = "HUD settings are on this page below.".to_owned();
            }
            if ui::ghost_button(ui, "Cancel key capture", Vec2::new(150.0, 34.0)).clicked() {
                app.recording_keybind = None;
            }
        });
        ui.add_space(8.0);
        ui::separator(ui);
        ui.add_space(8.0);
        ui::hud::show(ui, app);
        ui.add_space(10.0);
        ui::separator(ui);
        ui.add_space(10.0);
        ui::keybinds::show(ui, app);
    });

    ui.add_space(14.0);
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "Recommended performance set");
        for mod_info in PERFORMANCE_MODS {
            ui.horizontal(|ui| {
                ui.label(RichText::new(mod_info.name).strong().color(theme::TEXT));
                ui.label(
                    RichText::new(mod_info.what_it_fixes)
                        .size(12.0)
                        .color(theme::TEXT_DIM),
                );
            });
        }
    });

    ui.add_space(14.0);
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "Account and privacy");
        ui.label(
            RichText::new(
                "Alpinoxi never asks for a Microsoft password, never logs a token and never stores one in a plain file. Microsoft sign-in uses the device code flow; Ely.by uses PKCE with client id alpinoxi-client.",
            )
            .size(12.0)
            .color(theme::TEXT_DIM),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui::ghost_button(ui, "Save settings", Vec2::new(140.0, 34.0)).clicked() {
                app.save_settings();
            }
            ui.label(
                RichText::new(format!(
                    "Settings file: {}",
                    crate::config::settings_path().display()
                ))
                .size(11.0)
                .color(theme::TEXT_DIM),
            );
        });
    });
}
