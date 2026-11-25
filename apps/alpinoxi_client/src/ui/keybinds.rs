//! Keybind editor.
//!
//! Recording works by polling the raw key state while a bind is "armed", so the
//! player just presses the combination they want. Conflicts are reported, never
//! silently accepted.

use eframe::egui::{self, RichText, Vec2};

use alpinoxi_core::hud::{KeyChord, KeyCode, KeybindAction};

use crate::app::AlpinoxiApp;
use crate::theme;
use crate::ui;

/// Reads the current keyboard state into a chord, if a non-modifier key is down.
pub fn capture(ctx: &egui::Context) -> Option<KeyChord> {
    ctx.input(|input| {
        let (shift, ctrl, alt) = (
            input.modifiers.shift,
            input.modifiers.ctrl,
            input.modifiers.alt,
        );
        for code in KeyCode::ALL {
            if matches!(
                code,
                KeyCode::LeftShift
                    | KeyCode::RightShift
                    | KeyCode::LeftCtrl
                    | KeyCode::RightCtrl
                    | KeyCode::LeftAlt
                    | KeyCode::RightAlt
            ) {
                continue;
            }
            let Some(key) = translate(code) else {
                continue;
            };
            if input.key_pressed(key) {
                return Some(KeyChord {
                    key: code,
                    shift,
                    ctrl,
                    alt,
                });
            }
        }
        None
    })
}

/// The egui key for a bind's key code.
///
/// `None` for a bare modifier: egui 0.31's `Key` has no Shift, Control or Alt
/// variant, because modifiers are flags on `InputState::modifiers`, not keys.
/// `capture` reads them from there and never arms on one.
fn translate(code: KeyCode) -> Option<egui::Key> {
    use egui::Key;
    Some(match code {
        KeyCode::A => Key::A,
        KeyCode::B => Key::B,
        KeyCode::C => Key::C,
        KeyCode::D => Key::D,
        KeyCode::E => Key::E,
        KeyCode::F => Key::F,
        KeyCode::G => Key::G,
        KeyCode::H => Key::H,
        KeyCode::I => Key::I,
        KeyCode::J => Key::J,
        KeyCode::K => Key::K,
        KeyCode::L => Key::L,
        KeyCode::M => Key::M,
        KeyCode::N => Key::N,
        KeyCode::O => Key::O,
        KeyCode::P => Key::P,
        KeyCode::Q => Key::Q,
        KeyCode::R => Key::R,
        KeyCode::S => Key::S,
        KeyCode::T => Key::T,
        KeyCode::U => Key::U,
        KeyCode::V => Key::V,
        KeyCode::W => Key::W,
        KeyCode::X => Key::X,
        KeyCode::Y => Key::Y,
        KeyCode::Z => Key::Z,
        KeyCode::F1 => Key::F1,
        KeyCode::F2 => Key::F2,
        KeyCode::F3 => Key::F3,
        KeyCode::F4 => Key::F4,
        KeyCode::F5 => Key::F5,
        KeyCode::F6 => Key::F6,
        KeyCode::F7 => Key::F7,
        KeyCode::F8 => Key::F8,
        KeyCode::F9 => Key::F9,
        KeyCode::F10 => Key::F10,
        KeyCode::F11 => Key::F11,
        KeyCode::F12 => Key::F12,
        KeyCode::Tab => Key::Tab,
        KeyCode::Enter => Key::Enter,
        KeyCode::Escape => Key::Escape,
        KeyCode::Space => Key::Space,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Grave => Key::Backtick,
        KeyCode::LeftShift
        | KeyCode::RightShift
        | KeyCode::LeftCtrl
        | KeyCode::RightCtrl
        | KeyCode::LeftAlt
        | KeyCode::RightAlt => return None,
    })
}

pub fn show(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::eyebrow(ui, "Keybinds");
    ui::heading(ui, "Your keys, not a command list");
    ui.label(
        RichText::new("Click a key, press the new combination. Escape cancels.")
            .size(13.0)
            .color(theme::TEXT_DIM),
    );
    ui.add_space(12.0);

    if app.recording_keybind.is_some() && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        app.recording_keybind = None;
    }

    let binds = app.settings.keybinds.clone();
    for bind in binds.binds.iter() {
        ui.horizontal(|ui| {
            ui.label(RichText::new(bind.action.label()).color(theme::TEXT));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let recording = app.recording_keybind == Some(bind.action);
                let text = if recording {
                    "press a key…".to_owned()
                } else {
                    bind.chord.label()
                };
                let colour = if recording { theme::WARN } else { theme::TEXT };
                if ui
                    .add_sized(
                        [150.0, 30.0],
                        egui::Button::new(RichText::new(text).color(colour)),
                    )
                    .clicked()
                {
                    app.recording_keybind = if recording { None } else { Some(bind.action) };
                }
                if bind.chord.conflicts_with_vanilla() {
                    ui::pill(ui, "vanilla key", theme::WARN);
                }
            });
        });
        ui::separator(ui);
    }

    ui.add_space(10.0);
    ui.horizontal(|ui| {
        if ui::ghost_button(ui, "Reset to defaults", Vec2::new(150.0, 32.0)).clicked() {
            app.settings.keybinds.reset();
            app.notice = "Keybinds reset.".to_owned();
        }
        if let Some((left, right)) = app.settings.keybinds.find_conflict() {
            ui.label(
                RichText::new(format!("Conflict: {} and {}", left.label(), right.label()))
                    .size(12.0)
                    .color(theme::DANGER),
            );
        }
    });

    ui.add_space(8.0);
    ui.label(
        RichText::new(
            "Defaults: Shift+H HUD editor · F1 toggle HUD · O friends · Y chat · U skin preview",
        )
        .size(11.5)
        .color(theme::ACCENT_SOFT),
    );
    let _ = KeybindAction::OpenHudEditor;
}
