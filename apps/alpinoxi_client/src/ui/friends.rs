//! Friends and chat: one panel, two halves, no commands.
//!
//! Presence is shown honestly: online friends first, then favourites, then by
//! name. Each row carries its provider badge so a cracked friend and an Ely.by
//! friend are never confused, and the chat box is a normal text field.

use eframe::egui::{self, CornerRadius, RichText, Vec2};

use alpinoxi_core::friends::Presence;

use crate::app::AlpinoxiApp;
use crate::theme;
use crate::ui;

pub fn show(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::eyebrow(ui, "Friends");
    ui::heading(ui, "Everyone, whichever way they log in");
    ui.label(
        RichText::new("Microsoft, Ely.by and cracked players share one list and one chat.")
            .size(13.0)
            .color(theme::TEXT_DIM),
    );
    ui.add_space(16.0);

    ui.columns(2, |columns| {
        list_card(&mut columns[0], app);
        chat_card(&mut columns[1], app);
    });
}

fn list_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui::eyebrow(ui, "Friend list");
            let online = app
                .book
                .friends()
                .iter()
                .filter(|friend| app.book.presence_of(friend).is_online())
                .count();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui::pill(ui, &format!("{online} online"), theme::MINT);
            });
        });
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut app.friend_draft)
                    .hint_text("Player name")
                    .char_limit(16)
                    .desired_width(ui.available_width() - 100.0),
            );
            if ui::primary_button(ui, "Add", Vec2::new(88.0, 32.0)).clicked() {
                app.add_friend();
            }
        });
        ui.add_space(10.0);

        let rows = app.book.sorted();
        if rows.is_empty() {
            ui::empty_state(
                ui,
                "No friends yet",
                "Add a name above. Requests appear here too.",
            );
        }
        for (friend, presence) in rows {
            let colour = match presence {
                Presence::InGame { .. } => theme::MINT,
                Presence::InLauncher => theme::ACCENT,
                Presence::Offline => theme::TEXT_DIM,
            };
            egui::Frame::new()
                .fill(theme::BG_PANEL)
                .corner_radius(CornerRadius::same(10.0))
                .inner_margin(egui::Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.painter().circle_filled(
                            ui.cursor().min + Vec2::new(5.0, 8.0),
                            4.0,
                            colour,
                        );
                        ui.add_space(14.0);
                        ui.label(
                            RichText::new(friend.name.clone())
                                .strong()
                                .color(theme::TEXT),
                        );
                        ui::pill(
                            ui,
                            friend.provider.label(),
                            theme::provider_color(friend.provider),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(presence.label()).size(11.5).color(colour));
                            if let Presence::InGame {
                                server: Some(server),
                            } = &presence
                            {
                                ui.label(
                                    RichText::new(server.clone())
                                        .size(11.0)
                                        .color(theme::TEXT_DIM),
                                );
                            }
                        });
                    });
                });
            ui.add_space(6.0);
        }

        let pending = app.book.pending().len();
        if pending > 0 {
            ui.add_space(8.0);
            ui::eyebrow(ui, "Requests");
            ui.label(
                RichText::new(format!("{pending} pending"))
                    .size(12.0)
                    .color(theme::WARN),
            );
        }
    });
}

fn chat_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui::card(ui, |ui| {
        ui::eyebrow(ui, "Chat");
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new("Global").selected(app.chat_channel == "global"))
                .clicked()
            {
                app.chat_channel = "global".to_owned();
            }
            if let Some(first) = app.book.friends().first() {
                let channel = format!("dm:{}", first.name);
                if ui
                    .add(egui::Button::new("Friends").selected(app.chat_channel == channel))
                    .clicked()
                {
                    app.chat_channel = channel;
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new("no commands needed")
                        .size(11.0)
                        .color(theme::TEXT_DIM),
                );
            });
        });
        ui.add_space(8.0);

        egui::Frame::new()
            .fill(theme::BG_PANEL)
            .corner_radius(CornerRadius::same(10.0))
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(190.0);
                match &app.last_sent {
                    Some(text) => {
                        ui.label(
                            RichText::new(format!("You: {text}"))
                                .size(13.0)
                                .color(theme::TEXT),
                        );
                        ui.label(
                            RichText::new("Sent to the game chat bridge.")
                                .size(11.0)
                                .color(theme::MINT),
                        );
                    }
                    None => {
                        ui.label(
                            RichText::new(
                                "Messages in this channel show up here, with mentions highlighted.",
                            )
                            .size(12.0)
                            .color(theme::TEXT_DIM),
                        );
                    }
                }
            });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut app.chat_draft)
                    .hint_text("Write a message…")
                    .desired_width(ui.available_width() - 100.0),
            );
            let enter = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if ui::primary_button(ui, "Send", Vec2::new(88.0, 32.0)).clicked() || enter {
                app.send_chat();
            }
        });
        ui.add_space(6.0);
        ui.label(
            RichText::new("Chat keybind: Y · Friends keybind: O · HUD editor: Shift+H")
                .size(11.0)
                .color(theme::ACCENT_SOFT),
        );
    });
}
