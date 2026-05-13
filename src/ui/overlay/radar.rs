use egui::{Color32, Painter, Pos2, Stroke, pos2};

use crate::{data::Data, ui::app::App};

impl App {
    pub fn draw_radar(&self, painter: &Painter, data: &Data) {
        let cfg = &self.config.hud.radar;
        if !cfg.enabled || !data.in_game {
            return;
        }

        let size = cfg.size;
        let half = size / 2.0;

        // Radar origin (top-left corner with configured padding)
        let origin = pos2(cfg.pos_x, cfg.pos_y);
        let center = pos2(origin.x + half, origin.y + half);

        // Background rectangle
        let bg_color = Color32::from_rgba_premultiplied(0, 0, 0, cfg.background_alpha);
        painter.rect_filled(
            egui::Rect::from_min_size(origin, egui::vec2(size, size)),
            egui::CornerRadius::same(6),
            bg_color,
        );
        // Border
        painter.rect_stroke(
            egui::Rect::from_min_size(origin, egui::vec2(size, size)),
            egui::CornerRadius::same(6),
            Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 60)),
            egui::StrokeKind::Outside,
        );

        // Cross-hair lines at center
        let dim = Color32::from_rgba_premultiplied(255, 255, 255, 25);
        painter.line_segment(
            [pos2(center.x, origin.y + 2.0), pos2(center.x, origin.y + size - 2.0)],
            Stroke::new(1.0, dim),
        );
        painter.line_segment(
            [pos2(origin.x + 2.0, center.y), pos2(origin.x + size - 2.0, center.y)],
            Stroke::new(1.0, dim),
        );

        let local_pos = data.local_player.position;
        let local_yaw = data.view_angles.y; // degrees, used when rotate_with_player

        // Draw enemies
        for player in &data.players {
            if player.health <= 0 {
                continue;
            }
            let dot = self.world_to_radar(
                player.position,
                local_pos,
                local_yaw,
                center,
                half,
                cfg.zoom,
                cfg.rotate_with_player,
            );
            if !self.radar_in_bounds(dot, origin, size) {
                continue;
            }

            // Color: green if spotted/visible, red otherwise
            let color = if player.visible {
                Color32::from_rgb(100, 240, 100)
            } else {
                Color32::from_rgb(240, 80, 80)
            };

            painter.circle_filled(dot, 4.0, color);
            painter.circle_stroke(dot, 4.0, Stroke::new(1.0, Color32::from_rgba_premultiplied(0, 0, 0, 180)));

            if cfg.show_names && !player.name.is_empty() {
                painter.text(
                    pos2(dot.x + 6.0, dot.y - 6.0),
                    egui::Align2::LEFT_BOTTOM,
                    &player.name,
                    egui::FontId::proportional(10.0),
                    Color32::WHITE,
                );
            }
        }

        // Draw local player as a white triangle pointing in view direction
        self.draw_radar_player_arrow(painter, center, local_yaw, cfg.rotate_with_player);
    }

    fn world_to_radar(
        &self,
        world_pos: glam::Vec3,
        local_pos: glam::Vec3,
        yaw_deg: f32,
        center: Pos2,
        half: f32,
        zoom: f32,
        rotate: bool,
    ) -> Pos2 {
        let dx = world_pos.x - local_pos.x;
        let dy = world_pos.y - local_pos.y;

        let (rx, ry) = if rotate {
            // Rotate so player faces up
            let yaw = yaw_deg.to_radians();
            let (sin_y, cos_y) = yaw.sin_cos();
            (
                dx * cos_y + dy * sin_y,
                -dx * sin_y + dy * cos_y,
            )
        } else {
            (dx, dy)
        };

        // Scale: zoom controls how many world units fit in half the radar
        let scale = (half - 8.0) / (1.0 / zoom.max(0.001));
        pos2(
            center.x + rx * scale * zoom,
            center.y - ry * scale * zoom,
        )
    }

    fn radar_in_bounds(&self, dot: Pos2, origin: Pos2, size: f32) -> bool {
        dot.x >= origin.x + 3.0
            && dot.x <= origin.x + size - 3.0
            && dot.y >= origin.y + 3.0
            && dot.y <= origin.y + size - 3.0
    }

    fn draw_radar_player_arrow(&self, painter: &Painter, center: Pos2, yaw_deg: f32, rotate: bool) {
        // When rotating with player, the arrow always points up
        let angle = if rotate { 0.0f32 } else { (-yaw_deg).to_radians() };

        let (sin_a, cos_a) = angle.sin_cos();
        let tip_len = 8.0;
        let wing_len = 4.5;

        let tip = pos2(center.x + sin_a * tip_len, center.y - cos_a * tip_len);
        let left = pos2(
            center.x + (-cos_a - sin_a * 0.4) * wing_len,
            center.y + (-sin_a + (-cos_a) * 0.4) * wing_len,
        );
        let right = pos2(
            center.x + (cos_a - sin_a * 0.4) * wing_len,
            center.y + (sin_a + (-cos_a) * 0.4) * wing_len,
        );

        let fill = Color32::from_rgb(220, 220, 255);
        let stroke = Stroke::new(1.0, Color32::from_rgba_premultiplied(0, 0, 0, 200));

        painter.add(egui::Shape::convex_polygon(
            vec![tip, left, center, right],
            fill,
            stroke,
        ));
    }
}
