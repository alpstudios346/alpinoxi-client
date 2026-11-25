//! The login screen: three equal cards, one clear flow each.
//!
//! Visual choices that matter here: the provider cards are the same size and
//! weight (no provider is a second-class citizen), the cracked field validates
//! as you type, and the browser steps are written as steps rather than as an
//! error, because waiting for a browser is the normal path.

use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Stroke, Vec2};

use alpinoxi_core::account::{AccountProfile, AccountProvider};

use crate::app::AlpinoxiApp;
use crate::theme;
use crate::ui;

pub fn show(ctx: &egui::Context, app: &mut AlpinoxiApp) {
    let screen = ctx.screen_rect();
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(theme::BG_DEEP))
        .show(ctx, |ui| {
            paint_backdrop(ui, screen);
            ui.vertical_centered(|ui| {
                ui.add_space(46.0);
                ui.label(
                    RichText::new("ALPINOXI")
                        .size(40.0)
                        .strong()
                        .color(theme::TEXT),
                );
                ui.label(
                    RichText::new("CLIENT")
                        .size(13.0)
                        .strong()
                        .color(theme::ACCENT),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Choose how you play. Same client, same HUD, same friends.")
                        .size(14.0)
                        .color(theme::TEXT_DIM),
                );
                ui.add_space(34.0);
            });

            let width = (screen.width() - 120.0).min(1060.0);
            ui.allocate_ui_with_layout(
                Vec2::new(width, screen.height()),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 18.0;
                        let card_width = (width - 36.0) / 3.0;
                        for provider in AccountProvider::ALL {
                            provider_card(ui, app, provider, card_width);
                        }
                    });
                    ui.add_space(20.0);
                    status_bar(ui, app);
                    ui.add_space(14.0);
                    ui.label(
                        RichText::new(app.slogan.clone())
                            .size(13.0)
                            .italics()
                            .color(theme::TEXT_DIM),
                    );
                },
            );
        });
}

fn paint_backdrop(ui: &egui::Ui, rect: egui::Rect) {
    let painter = ui.painter();
    let centre = rect.center_top() + Vec2::new(0.0, 40.0);
    for (radius, alpha) in [(620.0, 26_u8), (420.0, 20), (260.0, 16)] {
        painter.circle_filled(
            centre,
            radius,
            Color32::from_rgba_unmultiplied(48, 140, 210, alpha),
        );
    }
    let bottom = rect.center_bottom();
    painter.circle_filled(
        bottom,
        520.0,
        Color32::from_rgba_unmultiplied(120, 80, 220, 18),
    );
}

fn provider_card(ui: &mut egui::Ui, app: &mut AlpinoxiApp, provider: AccountProvider, width: f32) {
    let accent = theme::provider_color(provider);
    let selected = app.login.provider == provider;
    let mut clicked = false;

    egui::Frame::new()
        .fill(if selected { theme::BG_CARD_HOVER } else { theme::BG_CARD })
        .stroke(Stroke::new(if selected { 1.6 } else { 1.0 }, if selected { accent } else { theme::BORDER }))
        .corner_radius(CornerRadius::same(16.0))
        .inner_margin(egui::Margin::same(18))
        .show(ui, |ui| {
            ui.set_width(width - 36.0);
            ui.horizontal(|ui| {
                ui.painter().circle_filled(ui.cursor().min + Vec2::new(6.0, 8.0), 5.0, accent);
                ui.add_space(16.0);
                ui.label(RichText::new(provider.label()).size(19.0).strong().color(theme::TEXT));
            });
            ui.add_space(4.0);
            ui.label(RichText::new(provider.tagline()).size(13.0).color(theme::TEXT_DIM));
            ui.add_space(14.0);
            ui::separator(ui);
            ui.add_space(12.0);

            match provider {
                AccountProvider::Microsoft => {
                    bullet(ui, "Official Microsoft sign-in");
                    bullet(ui, "Your premium skin and cape");
                    bullet(ui, "No password ever typed here");
                    ui.add_space(16.0);
                    if ui::primary_button(ui, "Sign in with Microsoft", Vec2::new(ui.available_width(), 40.0)).clicked() {
                        clicked = true;
                    }
                }
                AccountProvider::ElyBy => {
                    bullet(ui, "Ely.by OAuth · alpinoxi-client");
                    bullet(ui, "Ely.by skins and capes");
                    bullet(ui, "Works on Ely.by servers");
                    ui.add_space(16.0);
                    if ui::primary_button(ui, "Sign in with Ely.by", Vec2::new(ui.available_width(), 40.0)).clicked() {
                        clicked = true;
                    }
                }
                AccountProvider::Offline => {
                    bullet(ui, "Any name you like");
                    bullet(ui, "Local skin, no account needed");
                    bullet(ui, "Offline-mode servers only");
                    ui.add_space(12.0);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut app.login.cracked_name)
                            .hint_text("Player name")
                            .char_limit(16)
                            .desired_width(f32::INFINITY),
                    );
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        clicked = true;
                    }
                    let valid = AccountProfile::validate_name(&app.login.cracked_name).is_ok();
                    if !app.login.cracked_name.is_empty() && !valid {
                        ui.label(
                            RichText::new("3-16 characters: A-Z, 0-9, _")
                                .size(11.5)
                                .color(theme::WARN),
                        );
                    } else {
                        ui.add_space(15.0);
                    }
                    ui.add_space(2.0);
                    let button = egui::Button::new(
                        RichText::new("Play cracked").color(if valid { Color32::from_rgb(6, 14, 26) } else { theme::TEXT_DIM }).strong(),
                    )
                    .fill(if valid { accent } else { theme::BG_PANEL })
                    .corner_radius(CornerRadius::same(10.0));
                    if ui.add_sized([ui.available_width(), 40.0], button).clicked() && valid {
                        clicked = true;
                    }
                }
            }
        });

    // The whole card is a target too, so clicking the title selects the provider.
    if ui.interact(ui.min_rect(), ui.id().with(provider.id()), Sense::click()).clicked() {
        app.login.provider = provider;
    }
    if clicked {
        app.login(provider);
    }
}

fn bullet(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("·").color(theme::ACCENT_SOFT).strong());
        ui.label(RichText::new(text).size(13.0).color(theme::TEXT));
    });
}

fn status_bar(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    if app.notice.is_empty() {
        return;
    }
    let colour = match &app.login.state {
        alpinoxi_auth_system::AuthenticationState::Failed(_) => theme::DANGER,
        alpinoxi_auth_system::AuthenticationState::AwaitingBrowser { .. } => theme::ACCENT,
        _ => theme::MINT,
    };
    egui::Frame::new()
        .fill(theme::BG_PANEL)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(CornerRadius::same(12.0))
        .inner_margin(egui::Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width().min(700.0));
            ui.horizontal(|ui| {
                ui.label(RichText::new("●").color(colour).size(11.0));
                ui.label(RichText::new(app.notice.clone()).size(13.0).color(theme::TEXT));
            });
        });
}
