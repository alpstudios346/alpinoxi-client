//! Import from another launcher: pick a folder, see exactly what will be copied,
//! then copy it.
//!
//! The screen is deliberately a review, not a wizard. The player sees every file
//! that will move, every file that will not and why, and the size, before a single
//! byte is written. Nothing is ever deleted or moved from the other launcher.

use eframe::egui::{self, RichText, Vec2};

use alpinoxi_core::import::{self, DetectedSetup, ImportPlan, ImportSelection, SkipReason};

use crate::app::AlpinoxiApp;
use crate::theme;
use crate::ui;

/// Everything the import screen remembers between frames.
#[derive(Default)]
pub struct ImportState {
    pub folder: String,
    pub detected: Vec<DetectedSetup>,
    pub chosen: Option<usize>,
    pub selection: ImportSelection,
    pub plan: Option<ImportPlan>,
    pub status: String,
    /// Set after a successful copy, so the button cannot run twice.
    pub finished: bool,
}

impl ImportState {
    pub fn new() -> Self {
        Self {
            selection: ImportSelection::default(),
            ..Self::default()
        }
    }
}

pub fn show(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::eyebrow(ui, "Import");
    ui::heading(ui, "Bring your setup over");
    ui.label(
        RichText::new(
            "Point Alpinoxi at your existing .minecraft, Prism, MultiMC, CurseForge or Modrinth \
             folder. Your other launcher is never modified: files are copied, and accounts are \
             never imported — you sign in here.",
        )
        .size(13.0)
        .color(theme::TEXT_DIM),
    );
    ui.add_space(16.0);

    folder_card(ui, app);
    ui.add_space(12.0);

    if app.import.detected.is_empty() {
        ui::card(ui, |ui| {
            ui::empty_state(
                ui,
                "No setup found yet",
                "Paste the folder path above, or use Scan on your home folder.",
            );
        });
        return;
    }

    setup_card(ui, app);
    ui.add_space(12.0);
    selection_card(ui, app);
    ui.add_space(12.0);
    plan_card(ui, app);
}

fn folder_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "Folder");
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut app.import.folder)
                    .desired_width(ui.available_width() - 200.0)
                    .hint_text("C:/Users/you/AppData/Roaming/.minecraft"),
            );
            if ui::ghost_button(ui, "Scan", Vec2::new(72.0, 28.0)).clicked() {
                scan(app);
            }
            if ui::ghost_button(ui, "Common folders", Vec2::new(130.0, 28.0)).clicked() {
                scan_common(app);
            }
        });
        ui.label(
            RichText::new(
                "Prism and MultiMC are scanned one level down, so every instance shows up as its \
                 own row.",
            )
            .size(11.5)
            .color(theme::TEXT_DIM),
        );
    });
}

/// Scans the folder in the field.
pub fn scan(app: &mut AlpinoxiApp) {
    let path = std::path::PathBuf::from(app.import.folder.trim());
    if !path.is_dir() {
        app.import.status = format!("{} is not a folder", path.display());
        app.import.detected.clear();
        return;
    }
    let found = import::detect_all(&path);
    app.import.status = if found.is_empty() {
        "Nothing importable in that folder".to_owned()
    } else {
        format!("{} setup(s) found", found.len())
    };
    app.import.chosen = found.iter().position(|setup| setup.is_worth_importing());
    app.import.detected = found;
    app.import.plan = None;
    app.import.finished = false;
}

/// Looks in the places the common launchers actually use, so the player does not
/// have to know where their `.minecraft` lives.
pub fn scan_common(app: &mut AlpinoxiApp) {
    let mut found = Vec::new();
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    let candidates = [
        format!("{home}/AppData/Roaming/.minecraft"),
        format!("{home}/.minecraft"),
        format!("{home}/AppData/Roaming/PrismLauncher"),
        format!("{home}/AppData/Roaming/MultiMC"),
        format!("{home}/AppData/Roaming/ModrinthApp"),
        format!("{home}/Documents/Curse/Minecraft/Instances"),
        format!("{home}/curseforge/minecraft/Instances"),
    ];
    for candidate in candidates {
        let path = std::path::PathBuf::from(&candidate);
        if path.is_dir() {
            found.extend(import::detect_all(&path));
        }
    }
    app.import.status = if found.is_empty() {
        "No launcher found in the usual places".to_owned()
    } else {
        format!("{} setup(s) found", found.len())
    };
    app.import.chosen = found.iter().position(|setup| setup.is_worth_importing());
    app.import.detected = found;
    app.import.plan = None;
    app.import.finished = false;
}

fn setup_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "Found");
        let mut chosen = app.import.chosen;
        for (index, setup) in app.import.detected.iter().enumerate() {
            ui.horizontal(|ui| {
                let label = format!("{} · {}", setup.kind.label(), setup.summary());
                if ui
                    .selectable_label(chosen == Some(index), RichText::new(label).size(13.0))
                    .clicked()
                {
                    chosen = Some(index);
                }
            });
            ui.label(
                RichText::new(setup.game_dir.display().to_string())
                    .size(11.0)
                    .color(theme::TEXT_DIM),
            );
            ui.add_space(6.0);
        }
        if chosen != app.import.chosen {
            app.import.chosen = chosen;
            app.import.plan = None;
            app.import.finished = false;
        }
    });
}

fn selection_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "What to bring");
        let mut selection = app.import.selection;
        let mut changed = false;
        changed |= toggle(
            ui,
            "Mods",
            "Jars for this instance's game version only",
            &mut selection.mods,
        );
        changed |= toggle(ui, "Resource packs", "", &mut selection.resource_packs);
        changed |= toggle(ui, "Shader packs", "", &mut selection.shader_packs);
        changed |= toggle(
            ui,
            "Worlds",
            "Copied, not moved: the other launcher keeps its copy",
            &mut selection.worlds,
        );
        changed |= toggle(
            ui,
            "Options",
            "Graphics and gameplay settings",
            &mut selection.options,
        );
        changed |= toggle(
            ui,
            "Keybinds and HUD",
            "Off by default: another launcher's keys are not Alpinoxi's",
            &mut selection.settings,
        );

        ui.add_space(6.0);
        ui::separator(ui);
        ui.add_space(6.0);
        ui.label(
            RichText::new("Accounts are never imported")
                .size(12.0)
                .color(theme::TEXT_DIM)
                .strong(),
        );
        ui.label(
            RichText::new(
                "Another launcher's token is encrypted with its own key, so a copied token either \
                 fails or is one we cannot vouch for. Sign in with Microsoft, Ely.by or a cracked \
                 name once, and every instance has it.",
            )
            .size(11.5)
            .color(theme::TEXT_DIM),
        );

        if changed {
            app.import.selection = selection.sanitised();
            app.import.plan = None;
            app.import.finished = false;
        }
    });
}

fn toggle(ui: &mut egui::Ui, label: &str, hint: &str, value: &mut bool) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        if ui.checkbox(value, label).changed() {
            changed = true;
        }
        if !hint.is_empty() {
            ui.label(RichText::new(hint).size(11.5).color(theme::TEXT_DIM));
        }
    });
    changed
}

fn plan_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "Review");
        let Some(setup) = app
            .import
            .chosen
            .and_then(|index| app.import.detected.get(index))
            .cloned()
        else {
            ui.label(
                RichText::new("Pick a setup above.")
                    .size(13.0)
                    .color(theme::TEXT_DIM),
            );
            return;
        };

        if app.import.plan.is_none() {
            if ui::ghost_button(ui, "Build the list", Vec2::new(160.0, 28.0)).clicked() {
                let existing = existing_names(&setup);
                let name = setup
                    .game_dir
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Imported".to_owned());
                let plan = import::plan(setup.clone(), app.import.selection, &name, &existing);
                app.import.status = plan.headline();
                app.import.plan = Some(plan);
            }
            return;
        }

        let plan = app.import.plan.as_ref().expect("checked above");
        if plan.items.is_empty() {
            ui.label(
                RichText::new("Nothing to copy with this selection.")
                    .size(13.0)
                    .color(theme::TEXT_DIM),
            );
        } else {
            ui.label(
                RichText::new(plan.headline())
                    .size(14.0)
                    .color(theme::TEXT)
                    .strong(),
            );
            egui::ScrollArea::vertical()
                .max_height(160.0)
                .show(ui, |ui| {
                    for item in plan.items.iter().take(200) {
                        ui.label(
                            RichText::new(format!("{} · {}", item.destination, human(item.bytes)))
                                .size(11.5)
                                .color(theme::TEXT_DIM),
                        );
                    }
                });
        }

        if !plan.skipped.is_empty() {
            ui.add_space(6.0);
            let version_misses = plan
                .skipped
                .iter()
                .filter(|item| matches!(item.reason, SkipReason::VersionMismatch { .. }))
                .count();
            ui.label(
                RichText::new(format!(
                    "{} left behind{}",
                    plan.skipped.len(),
                    if version_misses > 0 {
                        format!(", {version_misses} for another game version")
                    } else {
                        String::new()
                    }
                ))
                .size(12.0)
                .color(theme::WARN),
            );
            egui::ScrollArea::vertical()
                .max_height(110.0)
                .show(ui, |ui| {
                    for item in &plan.skipped {
                        ui.label(
                            RichText::new(format!("{} — {}", item.relative, item.reason.message()))
                                .size(11.0)
                                .color(theme::TEXT_DIM),
                        );
                    }
                });
        }

        if let Some(warning) = &plan.warning {
            ui.add_space(6.0);
            ui.label(RichText::new(warning).size(12.0).color(theme::WARN));
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let label = if app.import.finished {
                "Imported"
            } else {
                "Import"
            };
            if ui::primary_button(ui, label, Vec2::new(140.0, 32.0)).clicked()
                && !app.import.finished
            {
                match execute(app) {
                    Ok(summary) => {
                        app.import.status = summary.clone();
                        app.import.finished = true;
                        app.toast = Some((summary, 0.0));
                    }
                    Err(error) => app.import.status = format!("Import failed: {error}"),
                }
            }
            ui.label(
                RichText::new(&app.import.status)
                    .size(12.0)
                    .color(theme::TEXT_DIM),
            );
        });
    });
}

/// Names already in the destination, so a second import does not overwrite.
fn existing_names(setup: &DetectedSetup) -> std::collections::BTreeSet<String> {
    let mut names = std::collections::BTreeSet::new();
    for folder in ["mods", "resourcepacks", "shaderpacks", "saves"] {
        if let Ok(entries) = std::fs::read_dir(setup.game_dir.join(folder)) {
            for entry in entries.flatten() {
                names.insert(entry.file_name().to_string_lossy().to_string());
            }
        }
    }
    names
}

/// Copies the reviewed plan into a new instance folder.
///
/// Writes into `<launcher data>/instances/<name>/` and stops at the first failure
/// rather than leaving a half-copied jar behind: a corrupt mod folder is worse
/// than no import.
fn execute(app: &mut AlpinoxiApp) -> Result<String, String> {
    let plan = app.import.plan.clone().ok_or("nothing to import")?;
    let root = crate::config::data_dir()
        .join("instances")
        .join(&plan.instance.name);
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;

    let mut copied = 0;
    for item in &plan.items {
        if !item.relative.starts_with("saves/") {
            let source = plan.source.game_dir.join(&item.relative);
            let destination = root.join(&item.destination);
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            std::fs::copy(&source, &destination)
                .map_err(|error| format!("{}: {error}", item.relative))?;
            copied += 1;
        } else {
            copy_tree(
                &plan.source.game_dir.join(&item.relative),
                &root.join(&item.destination),
            )?;
            copied += 1;
        }
    }

    app.settings.instance = plan.instance.clone();
    crate::config::save(&app.settings);
    Ok(format!("{copied} items imported into {}", root.display()))
}

fn copy_tree(source: &std::path::Path, destination: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(destination).map_err(|error| error.to_string())?;
    let entries = std::fs::read_dir(source).map_err(|error| error.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        let target = destination.join(entry.file_name());
        if path.is_dir() {
            copy_tree(&path, &target)?;
        } else {
            std::fs::copy(&path, &target).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn human(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}
