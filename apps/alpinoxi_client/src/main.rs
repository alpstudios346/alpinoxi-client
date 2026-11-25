//! Alpinoxi Client launcher.
//!
//! Native `egui` window, no web view. The app is split into small screens that
//! all read the same state struct, so a change made in the skin wardrobe or the
//! HUD editor is the same data the in-game layer uses.

mod app;
mod catalog;
mod config;
mod icons;
mod skinview;
mod theme;
mod ui;

use eframe::egui::{self, Vec2};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(1180.0, 760.0))
            .with_min_inner_size(Vec2::new(960.0, 620.0))
            .with_title("Alpinoxi Client")
            .with_icon(window_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Alpinoxi Client",
        options,
        Box::new(|creation_context| {
            theme::install(&creation_context.egui_ctx);
            Ok(Box::new(app::AlpinoxiApp::new()))
        }),
    )
}

/// Window icon from the PNG bundled next to the launcher, so the taskbar shows
/// the Alpinoxi mark instead of the default eframe triangle.
fn window_icon() -> egui::IconData {
    const CANDIDATES: &[&str] = &[
        "assets/icon-512.png",
        "../assets/icon-512.png",
        "../../assets/icon-512.png",
        "../../../assets/icon-512.png",
    ];
    for candidate in CANDIDATES {
        if let Ok(bytes) = std::fs::read(candidate) {
            if let Ok(image) = image::load_from_memory(&bytes) {
                let rgba = image.to_rgba8();
                return egui::IconData {
                    rgba: rgba.as_raw().clone(),
                    width: rgba.width(),
                    height: rgba.height(),
                };
            }
        }
    }
    egui::IconData {
        rgba: vec![0; 4],
        width: 1,
        height: 1,
    }
}
