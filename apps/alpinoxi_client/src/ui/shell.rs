//! The signed-in shell: navigation rail, top bar and the screen area.
//!
//! The rail is fixed-width and icon-led so the content area never jumps when a
//! label changes; the top bar carries the account chip and the tagline.

use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, Vec2};

use crate::app::{AlpinoxiApp, Screen};
use crate::icons::{self, Icon};
use crate::theme;
use crate::ui;

pub fn show(ctx: &egui::Context, app: &mut AlpinoxiApp) {
    egui::SidePanel::left("navigation")
        .resizable(false)
        .exact_width(232.0)
        .frame(
            egui::Frame::new()
                .fill(theme::BG_PANEL)
                .stroke(Stroke::new(1.0, theme::BORDER))
                .inner_margin(egui::Margin::symmetric(14, 18)),
        )
        .show(ctx, |ui| {
            rail(ui, app);
        });

    egui::TopBottomPanel::top("top-bar")
        .exact_height(66.0)
        .frame(
            egui::Frame::new()
                .fill(theme::BG_PANEL)
                .stroke(Stroke::new(1.0, theme::BORDER))
                .inner_margin(egui::Margin::symmetric(22, 12)),
        )
        .show(ctx, |ui| top_bar(ui, app));

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(theme::BG_DEEP)
                .inner_margin(egui::Margin::symmetric(26, 22)),
        )
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| match app.screen {
                    Screen::Home => ui::home::show(ui, app),
                    Screen::Instances => ui::instances::show(ui, app),
                    Screen::Skin => ui::skin::show(ui, app),
                    Screen::Friends => ui::friends::show(ui, app),
                    Screen::Mods => ui::mods::show(ui, app),
                    Screen::Settings => ui::settings::show(ui, app),
                });
        });
}

fn rail(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui.horizontal(|ui| {
        icons::draw(ui, Icon::Box, theme::ACCENT, 22.0);
        ui.vertical(|ui| {
            ui.label(RichText::new("ALPINOXI").size(18.0).strong().color(theme::TEXT));
            ui.label(RichText::new("CLIENT").size(10.0).strong().color(theme::TEXT_DIM));
        });
    });
    ui.add_space(22.0);
    ui::separator(ui);
    ui.add_space(14.0);

    for screen in Screen::ALL {
        let selected = app.screen == screen;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 40.0), egui::Sense::click());
        let painter = ui.painter();
        if selected {
            painter.rect_filled(rect, CornerRadius::same(10.0), theme::BG_CARD_HOVER);
            let bar = egui::Rect::from_min_size(
                rect.left_top() + Vec2::new(0.0, 8.0),
                Vec2::new(3.0, rect.height() - 16.0),
            );
            painter.rect_filled(bar, CornerRadius::same(2.0), theme::ACCENT);
        } else if response.hovered() {
            painter.rect_filled(rect, CornerRadius::same(10.0), theme::BG_CARD);
        }
        let colour = if selected { theme::ACCENT } else { theme::TEXT_DIM };
        let icon_rect = egui::Rect::from_center_size(
            rect.left_center() + Vec2::new(20.0, 0.0),
            Vec2::splat(20.0),
        );
        icons::paint(painter, icon_rect, rail_icon(screen), colour);
        painter.text(
            rect.left_center() + Vec2::new(42.0, 0.0),
            egui::Align2::LEFT_CENTER,
            screen.label(),
            egui::FontId::proportional(14.5),
            if selected { theme::TEXT } else { theme::TEXT_DIM },
        );
        if response.clicked() {
            app.screen = screen;
            if screen == Screen::Friends {
                app.refresh_presence();
            }
        }
        ui.add_space(4.0);
    }

    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        ui.label(
            RichText::new(format!("v{} · Rust native", env!("CARGO_PKG_VERSION")))
                .size(11.0)
                .color(theme::TEXT_DIM),
        );
        ui.label(
            RichText::new("No Electron. No web view.")
                .size(11.0)
                .color(theme::ACCENT_SOFT),
        );
        ui.add_space(6.0);
        ui::separator(ui);
        ui.add_space(8.0);
        let unread = app.book.pending().len();
        if unread > 0 {
            ui.label(
                RichText::new(format!("{unread} friend request(s)"))
                    .size(12.0)
                    .color(theme::WARN),
            );
        }
    });
}

fn rail_icon(screen: Screen) -> Icon {
    match screen {
        Screen::Home => Icon::Home,
        Screen::Instances => Icon::Box,
        Screen::Skin => Icon::Skin,
        Screen::Friends => Icon::Friends,
        Screen::Mods => Icon::Mods,
        Screen::Settings => Icon::Settings,
    }
}

fn top_bar(ui: &mut egui::Ui, app: &mut AlpinoxiApp) {
    ui.horizontal_centered(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(app.slogan.clone()).size(13.5).color(theme::ACCENT));
            ui.label(
                RichText::new(format!(
                    "{} · {} MiB heap · {} mods enabled",
                    app.settings.instance.game_version,
                    app.launch_plan().mebibytes,
                    app.settings.instance.enabled_mods.len()
                ))
                .size(11.5)
                .color(theme::TEXT_DIM),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(account) = app.account.clone() {
                let colour = theme::provider_color(account.provider);
                egui::Frame::new()
                    .fill(theme::BG_CARD)
                    .stroke(Stroke::new(1.0, theme::BORDER))
                    .corner_radius(CornerRadius::same(10.0))
                    .inner_margin(egui::Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.painter().circle_filled(
                                ui.cursor().min + Vec2::new(5.0, 8.0),
                                4.5,
                                colour,
                            );
                            ui.add_space(14.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(account.display_name.clone())
                                        .size(13.5)
                                        .strong()
                                        .color(theme::TEXT),
                                );
                                ui.label(
                                    RichText::new(account.provider.label())
                                        .size(10.5)
                                        .color(colour),
                                );
                            });
                        });
                    });
                if ui::ghost_button(ui, "Sign out", Vec2::new(88.0, 32.0)).clicked() {
                    app.sign_out();
                }
            }
            if ui::primary_button(ui, "Launch", Vec2::new(118.0, 34.0)).clicked() {
                app.launch();
            }
            ui.add_space(4.0);
            if ui
                .add(egui::Button::new(RichText::new("↻").size(16.0).color(theme::TEXT_DIM)).frame(false))
                .on_hover_text("Another tagline")
                .clicked()
            {
                app.next_slogan();
            }
        });
    });
}

/// Small helper shared by screens that show a provider badge.
pub fn provider_badge(ui: &mut egui::Ui, provider: alpinoxi_core::AccountProvider) {
    let colour: Color32 = theme::provider_color(provider);
    ui::pill(ui, provider.label(), colour);
}
