//! Mods and resource packs: a curated catalogue with an honest licence line.
//!
//! Alpinoxi does not bundle other people's files. Each row links to the
//! project's own page, and the launcher records the choice in the instance.

use eframe::egui::{self, RichText, Vec2};

use crate::app::AlpinoxiApp;
use crate::catalog::{self, Kind};
use crate::theme;
use crate::ui;

pub fn show(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::eyebrow(ui, "Catalogue");
    ui::heading(ui, "Mods, packs, one click");
    ui.label(
        RichText::new("Performance mods first. Every entry links to its own project page: licences stay with their authors.")
            .size(13.0)
            .color(theme::TEXT_DIM),
    );
    ui.add_space(14.0);

    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut app.catalog_query)
                .hint_text("Search…")
                .desired_width(260.0),
        );
        if ui
            .add(egui::Button::new("All").selected(app.catalog_kind.is_none()))
            .clicked()
        {
            app.catalog_kind = None;
        }
        for kind in [Kind::Mod, Kind::ResourcePack] {
            if ui
                .add(egui::Button::new(kind.label()).selected(app.catalog_kind == Some(kind)))
                .clicked()
            {
                app.catalog_kind = Some(kind);
            }
        }
        if ui
            .add(egui::Button::new("Performance only").selected(app.catalog_performance_only))
            .clicked()
        {
            app.catalog_performance_only = !app.catalog_performance_only;
        }
    });

    ui.add_space(14.0);

    let matches: Vec<&catalog::CatalogEntry> = catalog::search(
        &app.catalog,
        &app.catalog_query,
        app.catalog_kind,
        app.catalog_performance_only,
    )
    .into_iter()
    .collect();

    if matches.is_empty() {
        ui::empty_state(
            ui,
            "Nothing matches",
            "Try a shorter search or clear the filters.",
        );
        return;
    }

    // Owned ids: the entries borrow `app.catalog`, and the buttons below need
    // `app` mutably.
    let matches: Vec<String> = matches.iter().map(|entry| entry.id.to_owned()).collect();

    for id in matches {
        let Some(entry) = app
            .catalog
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| {
                (
                    entry.name,
                    entry.summary,
                    entry.kind.label(),
                    entry.licence,
                    entry.url,
                    entry.performance,
                )
            })
        else {
            continue;
        };
        let (name, summary, kind, licence, url, performance) = entry;
        let installed = app
            .settings
            .instance
            .enabled_mods
            .iter()
            .any(|id| id == entry.id);
        ui::card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(name).size(16.0).strong().color(theme::TEXT));
                        ui::pill(ui, kind, theme::ACCENT_SOFT);
                        if performance {
                            ui::pill(ui, "Performance", theme::MINT);
                        }
                    });
                    ui.label(RichText::new(summary).size(12.5).color(theme::TEXT_DIM));
                    ui.label(RichText::new(licence).size(11.0).color(theme::ACCENT_SOFT));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let label = if installed {
                        "Remove"
                    } else {
                        "Add to instance"
                    };
                    if ui::ghost_button(ui, label, Vec2::new(140.0, 32.0)).clicked() {
                        toggle(app, &id);
                    }
                    if ui::ghost_button(ui, "Open page", Vec2::new(110.0, 32.0)).clicked() {
                        ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                    }
                });
            });
        });
        ui.add_space(10.0);
    }
}

fn toggle(app: &mut AlpinoxiApp, id: &str) {
    let mods = &mut app.settings.instance.enabled_mods;
    match mods.iter().position(|existing| existing == id) {
        Some(index) => {
            mods.remove(index);
        }
        None => mods.push(id.to_owned()),
    }
}
