//! Home: the one screen a player lands on. Launch, the instance card, the RAM
//! plan and a short account summary, nothing else.

use eframe::egui::{self, Color32, RichText, Vec2};

use alpinoxi_core::launcher::{MemoryProfile, SLOGANS};

use crate::app::AlpinoxiApp;
use crate::theme;
use crate::ui;

pub fn show(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui::eyebrow(ui, "Welcome back");
            ui.label(
                RichText::new(format!("Ready, {}?", app.account.as_ref().map(|a| a.display_name.as_str()).unwrap_or("player")))
                    .size(28.0)
                    .strong()
                    .color(theme::TEXT),
            );
        });
    });
    ui.add_space(16.0);

    instance_card(ui, app);
    ui.add_space(16.0);

    ui.columns(2, |columns| {
        memory_card(&mut columns[0], app);
        account_card(&mut columns[1], app);
    });

    ui.add_space(16.0);
    performance_card(ui, app);

    if let Some((text, _)) = app.toast.clone() {
        ui.add_space(14.0);
        ui::pill(ui, &text, theme::MINT);
    }
}

fn instance_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui::eyebrow(ui, "Selected instance");
                ui.label(
                    RichText::new(app.settings.instance.name.clone())
                        .size(22.0)
                        .strong()
                        .color(theme::TEXT),
                );
                ui.label(
                    RichText::new(format!(
                        "{} · {} · {} mods · {} resource packs",
                        app.settings.instance.game_version,
                        app.settings.instance.loader,
                        app.settings.instance.enabled_mods.len(),
                        app.settings.instance.resource_packs.len()
                    ))
                    .size(12.5)
                    .color(theme::TEXT_DIM),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui::primary_button(ui, "Launch game", Vec2::new(160.0, 44.0)).clicked() {
                    app.launch();
                }
                if ui::ghost_button(ui, "Edit", Vec2::new(74.0, 40.0)).clicked() {
                    app.screen = crate::app::Screen::Instances;
                }
            });
        });
        ui.add_space(12.0);
        ui::separator(ui);
        ui.add_space(10.0);
        ui.horizontal_wrapped(|ui| {
            let plan = app.launch_plan();
            ui::pill(ui, &format!("{} MiB heap", plan.mebibytes), theme::ACCENT);
            ui::pill(ui, "G1GC tuned", theme::MINT);
            ui::pill(
                ui,
                if app.settings.keep_launcher_light {
                    "Launcher stays light"
                } else {
                    "Launcher kept in tray"
                },
                theme::ACCENT_SOFT,
            );
            if let Some(note) = app.memory_warning() {
                ui::pill(ui, "RAM capped", theme::WARN);
                ui.add_space(4.0);
                ui.label(RichText::new(note).size(12.0).color(theme::WARN));
            }
        });
    });
}

fn memory_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "Memory for Minecraft");
        ui.label(RichText::new("Low RAM, high frames").size(18.0).strong().color(theme::TEXT));
        ui.label(
            RichText::new("A bigger heap is not a faster game. Alpinoxi caps it at half of your RAM.")
                .size(12.0)
                .color(theme::TEXT_DIM),
        );
        ui.add_space(10.0);
        for (profile, title, detail) in [
            (MemoryProfile::Balanced, "Balanced", "2 GB · vanilla and Sodium"),
            (MemoryProfile::Performance, "Performance", "4 GB · modded gameplay"),
        ] {
            let selected = app.settings.memory == profile;
            let label = format!("{title}   {detail}");
            if ui
                .add_sized(
                    [ui.available_width(), 34.0],
                    egui::Button::new(RichText::new(label).color(if selected { theme::TEXT } else { theme::TEXT_DIM }))
                        .selected(selected),
                )
                .clicked()
            {
                app.set_memory(profile);
            }
        }
        ui.add_space(8.0);
        let mut custom = app.settings.memory.mebibytes() as u32;
        let slider = ui.add(
            egui::Slider::new(&mut custom, 512..=16_384)
                .step_by(512.0)
                .suffix(" MiB")
                .text("Custom"),
        );
        if slider.changed() {
            app.set_memory(MemoryProfile::Custom(custom as u16));
        }
        ui.label(
            RichText::new(format!(
                "System RAM: {} MiB · ceiling for this machine: {} MiB",
                app.settings.system_memory_mib,
                app.launch_plan().mebibytes
            ))
            .size(11.5)
            .color(theme::TEXT_DIM),
        );
    });
}

fn account_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "Account");
        match app.account.clone() {
            Some(account) => {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(account.display_name.clone()).size(18.0).strong().color(theme::TEXT));
                    ui::pill(ui, account.provider.label(), theme::provider_color(account.provider));
                });
                ui.label(RichText::new(format!("UUID {}", account.uuid)).size(11.0).color(theme::TEXT_DIM));
                ui.add_space(8.0);
                ui.label(
                    RichText::new(match app.wardrobe.skin.as_ref() {
                        Some(skin) => skin.visibility_note(),
                        None => "No skin selected yet.",
                    })
                    .size(12.0)
                    .color(theme::TEXT_DIM),
                );
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui::ghost_button(ui, "Open wardrobe", Vec2::new(130.0, 34.0)).clicked() {
                        app.screen = crate::app::Screen::Skin;
                    }
                    if ui::ghost_button(ui, "Friends", Vec2::new(96.0, 34.0)).clicked() {
                        app.refresh_presence();
                        app.screen = crate::app::Screen::Friends;
                    }
                });
            }
            None => {
                ui.label(RichText::new("Not signed in").size(18.0).color(theme::TEXT));
            }
        }
        ui.add_space(10.0);
        ui::separator(ui);
        ui.add_space(8.0);
        ui.label(RichText::new(app.slogan.clone()).size(12.5).italics().color(theme::ACCENT));
        ui.horizontal_wrapped(|ui| {
            for line in SLOGANS.iter().take(3) {
                ui::pill(ui, line, Color32::from_rgb(120, 150, 190));
            }
        });
    });
}

fn performance_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui::eyebrow(ui, "Performance");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui::ghost_button(ui, "Open catalogue", Vec2::new(132.0, 32.0)).clicked() {
                    app.screen = crate::app::Screen::Mods;
                    app.catalog_performance_only = true;
                }
            });
        });
        ui.label(
            RichText::new("Client-side fixes only: they change how the game renders, never what it does.")
                .size(12.0)
                .color(theme::TEXT_DIM),
        );
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            for entry in crate::catalog::entries().iter().filter(|entry| entry.performance) {
                let installed = app
                    .settings
                    .instance
                    .enabled_mods
                    .iter()
                    .any(|id| id == entry.id);
                let colour = if installed { theme::MINT } else { theme::TEXT_DIM };
                if ui
                    .add(
                        egui::Button::new(RichText::new(entry.name).size(12.5).color(colour))
                            .selected(installed),
                    )
                    .on_hover_text(entry.summary)
                    .clicked()
                {
                    toggle_mod(app, entry.id);
                }
            }
        });
    });
}

fn toggle_mod(app: &mut AlpinoxiApp, id: &str) {
    let mods = &mut app.settings.instance.enabled_mods;
    match mods.iter().position(|existing| existing == id) {
        Some(index) => {
            mods.remove(index);
        }
        None => mods.push(id.to_owned()),
    }
}
