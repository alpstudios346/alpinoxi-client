//! Small vector marks drawn with the painter.
//!
//! No icon font and no SVG dependency: the launcher keeps a few shapes in code,
//! which keeps the binary small and every mark crisp at any scale.

use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Home,
    Box,
    Skin,
    Friends,
    Mods,
    Settings,
    Play,
    Plus,
    Check,
    Cross,
    Search,
    Arrow,
}

pub fn draw(ui: &mut egui::Ui, icon: Icon, color: Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    paint(ui.painter(), rect, icon, color);
}

pub fn paint(painter: &egui::Painter, rect: Rect, icon: Icon, color: Color32) {
    let stroke = Stroke::new((rect.width() * 0.09).max(1.2), color);
    let c = rect.center();
    let r = rect.width() * 0.36;
    match icon {
        Icon::Home => {
            painter.line_segment([Pos2::new(c.x - r, c.y), Pos2::new(c.x, c.y - r)], stroke);
            painter.line_segment([Pos2::new(c.x, c.y - r), Pos2::new(c.x + r, c.y)], stroke);
            painter.rect_stroke(
                Rect::from_min_max(
                    Pos2::new(c.x - r * 0.72, c.y),
                    Pos2::new(c.x + r * 0.72, c.y + r),
                ),
                CornerRadius::same(2.0),
                stroke,
                egui::StrokeKind::Middle,
            );
        }
        Icon::Box => {
            let corners = [
                Pos2::new(c.x - r, c.y - r * 0.5),
                Pos2::new(c.x, c.y - r),
                Pos2::new(c.x + r, c.y - r * 0.5),
                Pos2::new(c.x + r, c.y + r * 0.5),
                Pos2::new(c.x, c.y + r),
                Pos2::new(c.x - r, c.y + r * 0.5),
            ];
            for pair in corners.windows(2) {
                painter.line_segment([pair[0], pair[1]], stroke);
            }
            painter.line_segment([corners[5], corners[0]], stroke);
            painter.line_segment([Pos2::new(c.x, c.y), corners[1]], stroke);
            painter.line_segment([Pos2::new(c.x, c.y), corners[4]], stroke);
            painter.line_segment(
                [Pos2::new(c.x, c.y), Pos2::new(c.x - r, c.y + r * 0.5)],
                stroke,
            );
            painter.line_segment(
                [Pos2::new(c.x, c.y), Pos2::new(c.x + r, c.y + r * 0.5)],
                stroke,
            );
        }
        Icon::Skin => {
            painter.circle_stroke(Pos2::new(c.x, c.y - r * 0.55), r * 0.42, stroke);
            painter.add(egui::Shape::line(
                vec![
                    Pos2::new(c.x - r * 0.85, c.y + r),
                    Pos2::new(c.x - r * 0.6, c.y + r * 0.15),
                    Pos2::new(c.x, c.y - r * 0.05),
                    Pos2::new(c.x + r * 0.6, c.y + r * 0.15),
                    Pos2::new(c.x + r * 0.85, c.y + r),
                ],
                stroke,
            ));
        }
        Icon::Friends => {
            painter.circle_stroke(Pos2::new(c.x - r * 0.45, c.y - r * 0.4), r * 0.32, stroke);
            painter.circle_stroke(Pos2::new(c.x + r * 0.5, c.y - r * 0.25), r * 0.26, stroke);
            painter.add(egui::Shape::line(
                vec![
                    Pos2::new(c.x - r, c.y + r * 0.85),
                    Pos2::new(c.x - r * 0.5, c.y + r * 0.05),
                    Pos2::new(c.x, c.y + r * 0.85),
                ],
                stroke,
            ));
        }
        Icon::Mods => {
            for index in 0..4 {
                let x = c.x - r * 0.75 + index as f32 * r * 0.5;
                painter.rect_stroke(
                    Rect::from_center_size(Pos2::new(x, c.y - r * 0.45), Vec2::splat(r * 0.42)),
                    CornerRadius::same(2.0),
                    stroke,
                    egui::StrokeKind::Middle,
                );
                painter.rect_stroke(
                    Rect::from_center_size(Pos2::new(x, c.y + r * 0.45), Vec2::splat(r * 0.42)),
                    CornerRadius::same(2.0),
                    stroke,
                    egui::StrokeKind::Middle,
                );
            }
        }
        Icon::Settings => {
            painter.circle_stroke(c, r * 0.42, stroke);
            for index in 0..8 {
                let angle = index as f32 * std::f32::consts::TAU / 8.0;
                let inner = Pos2::new(c.x + angle.cos() * r * 0.62, c.y + angle.sin() * r * 0.62);
                let outer = Pos2::new(c.x + angle.cos() * r, c.y + angle.sin() * r);
                painter.line_segment([inner, outer], stroke);
            }
        }
        Icon::Play => {
            painter.add(egui::Shape::convex_polygon(
                vec![
                    Pos2::new(c.x - r * 0.55, c.y - r),
                    Pos2::new(c.x + r, c.y),
                    Pos2::new(c.x - r * 0.55, c.y + r),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Plus => {
            painter.line_segment([Pos2::new(c.x - r, c.y), Pos2::new(c.x + r, c.y)], stroke);
            painter.line_segment([Pos2::new(c.x, c.y - r), Pos2::new(c.x, c.y + r)], stroke);
        }
        Icon::Check => {
            painter.add(egui::Shape::line(
                vec![
                    Pos2::new(c.x - r, c.y),
                    Pos2::new(c.x - r * 0.2, c.y + r * 0.7),
                    Pos2::new(c.x + r, c.y - r * 0.7),
                ],
                stroke,
            ));
        }
        Icon::Cross => {
            painter.line_segment(
                [
                    Pos2::new(c.x - r * 0.8, c.y - r * 0.8),
                    Pos2::new(c.x + r * 0.8, c.y + r * 0.8),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x + r * 0.8, c.y - r * 0.8),
                    Pos2::new(c.x - r * 0.8, c.y + r * 0.8),
                ],
                stroke,
            );
        }
        Icon::Search => {
            painter.circle_stroke(Pos2::new(c.x - r * 0.2, c.y - r * 0.2), r * 0.6, stroke);
            painter.line_segment(
                [
                    Pos2::new(c.x + r * 0.25, c.y + r * 0.25),
                    Pos2::new(c.x + r, c.y + r),
                ],
                stroke,
            );
        }
        Icon::Arrow => {
            painter.line_segment([Pos2::new(c.x - r, c.y), Pos2::new(c.x + r, c.y)], stroke);
            painter.line_segment(
                [
                    Pos2::new(c.x + r * 0.2, c.y - r * 0.7),
                    Pos2::new(c.x + r, c.y),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(c.x + r * 0.2, c.y + r * 0.7),
                    Pos2::new(c.x + r, c.y),
                ],
                stroke,
            );
        }
    }
}
