use std::time::{Duration, Instant};

use egui::{Align2, Color32, FontId, Painter, Stroke, pos2};
use glam::vec3;

use crate::{
    config::{BoxMode, DrawMode},
    cs2::bones::Bones,
    data::{Data, PlayerData, SoundType},
    math::{world_to_screen, world_to_screen_loose},
    ui::app::App,
};

impl App {
    pub fn draw_player(&self, painter: &Painter, player: &PlayerData, data: &Data) {
        if self.config.player.visible_only && !player.visible {
            return;
        }

        let sound = self.player_sounds.get(&player.steam_id);
        let sound_alpha = if self.config.player.sound.enabled {
            self.player_sound_alpha(player, sound, data)
        } else {
            None
        };

        // Animated footstep rings — drawn under the box so they don't clutter names
        if self.config.player.sound.enabled {
            self.draw_sound_rings(painter, player, sound, data);
        }

        self.player_box(painter, player, data, sound_alpha);
        self.skeleton(painter, player, data, sound_alpha);
    }

    fn player_sound_alpha(
        &self,
        player: &PlayerData,
        sound: Option<&(Instant, SoundType)>,
        data: &Data,
    ) -> Option<f32> {
        if self.config.player.sound.show_visible && player.visible {
            return Some(1.0);
        }

        let Some((time, sound)) = sound else {
            return Some(0.0);
        };

        let local_player = &data.local_player;
        let max_distance = match sound {
            SoundType::Footstep => self.config.player.sound.footstep_diameter,
            SoundType::Gunshot => self.config.player.sound.gunshot_diameter,
            SoundType::Weapon => self.config.player.sound.weapon_diameter,
        };
        if local_player.position.distance(player.position) > max_distance {
            return Some(0.0);
        }

        if time.elapsed() > self.total_sound_duration() {
            return Some(0.0);
        }

        Some(
            1.0 - ((time.elapsed().as_secs_f32() - self.config.player.sound.fadeout_start)
                / self.config.player.sound.fadeout_duration),
        )
    }

    fn total_sound_duration(&self) -> Duration {
        Duration::from_secs_f32(
            self.config.player.sound.fadeout_start + self.config.player.sound.fadeout_duration,
        )
    }

    fn alpha(color: Color32, alpha: f32) -> Color32 {
        Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            (alpha.clamp(0.0, 1.0) * 255.0) as u8,
        )
    }

    fn player_box(&self, painter: &Painter, player: &PlayerData, data: &Data, alpha: Option<f32>) {
        use crate::config::DrawMode;

        let alpha = match alpha {
            Some(alpha) => alpha.clamp(0.0, 1.0),
            None => 1.0,
        };
        let distance = data
            .local_player
            .position
            .distance(player.position)
            .max(1.0);

        let esp_scale = (500.0 / distance).clamp(0.4, 1.0);
        let line_width = self.config.hud.line_width * esp_scale;

        let health_color =
            self.health_color(player.health, self.config.player.box_visible_color.a());
        let mut color = match &self.config.player.draw_box {
            DrawMode::None => health_color,
            DrawMode::Health => health_color,
            DrawMode::Color => {
                if player.visible {
                    self.config.player.box_visible_color
                } else {
                    self.config.player.box_invisible_color
                }
            }
        };

        color = Self::alpha(color, alpha);

        let stroke = Stroke::new(line_width, color);
        let icon_font = FontId::monospace(self.config.hud.icon_size * esp_scale);

        let midpoint = (player.position + player.head) / 2.0;
        let height = player.head.z - player.position.z + 24.0;
        let half_height = height / 2.0;
        let top = midpoint + vec3(0.0, 0.0, half_height);
        let bottom = midpoint - vec3(0.0, 0.0, half_height);

        let Some(top) = world_to_screen_loose(&top, data) else {
            return;
        };
        let Some(bottom) = world_to_screen_loose(&bottom, data) else {
            return;
        };
        let half_height = bottom.y - top.y;
        let width = half_height / 2.0;
        let half_width = width / 2.0;
        // quarter width
        let qw = half_width - 2.0;
        // eigth width
        let ew = qw / 2.0;

        let tl = pos2(top.x - half_width, top.y);
        let tr = pos2(top.x + half_width, top.y);
        let bl = pos2(bottom.x - half_width, bottom.y);
        let br = pos2(bottom.x + half_width, bottom.y);

        if self.config.player.draw_box != DrawMode::None {
            if self.config.player.box_mode == BoxMode::Gap {
                painter.line(
                    vec![pos2(tl.x + ew, tl.y), tl, pos2(tl.x, tl.y + qw)],
                    stroke,
                );
                painter.line(
                    vec![pos2(tr.x - ew, tl.y), tr, pos2(tr.x, tr.y + qw)],
                    stroke,
                );
                painter.line(
                    vec![pos2(bl.x + ew, bl.y), bl, pos2(bl.x, bl.y - qw)],
                    stroke,
                );
                painter.line(
                    vec![pos2(br.x - ew, bl.y), br, pos2(br.x, br.y - qw)],
                    stroke,
                );
            } else {
                painter.rect(
                    egui::Rect::from_min_max(tl, br),
                    0,
                    Color32::TRANSPARENT,
                    stroke,
                    egui::StrokeKind::Middle,
                );
            }
        }

        // health bar
        if self.config.player.health_bar {
            let x = bl.x - line_width * 2.0;
            let delta = bl.y - tl.y;
            painter.line(
                vec![
                    pos2(x, bl.y),
                    pos2(x, bl.y - (delta * player.health as f32 / 100.0)),
                ],
                Stroke::new(line_width, Self::alpha(health_color, alpha)),
            );
        }

        if self.config.player.armor_bar && player.armor > 0 {
            let x = bl.x
                - line_width
                    * if self.config.player.health_bar {
                        4.0
                    } else {
                        2.0
                    };
            let delta = bl.y - tl.y;
            painter.line(
                vec![
                    pos2(x, bl.y),
                    pos2(x, bl.y - (delta * player.armor as f32 / 100.0)),
                ],
                Stroke::new(
                    line_width,
                    Self::alpha(Color32::BLUE, alpha),
                ),
            );
        }

        let mut offset = 0.0;
        let font_size = self.config.hud.font_size * esp_scale;
        let name_color  = Self::alpha(self.config.player.name_color,         alpha);
        let weapon_color = Self::alpha(self.config.player.weapon_icon_color, alpha);
        let tag_color   = Self::alpha(self.config.player.tag_color,          alpha);
        if self.config.player.player_name {
            self.text_sized(
                painter,
                &player.name,
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                Some(name_color),
                font_size,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_defuser {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "\u{e00f}",
                icon_font.clone(),
                tag_color,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_helmet {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "\u{e017}",
                icon_font.clone(),
                tag_color,
            );
            offset += font_size;
        }

        if self.config.player.tags && player.has_bomb {
            painter.text(
                pos2(tr.x + ew, tr.y + offset),
                Align2::LEFT_TOP,
                "\u{e01e}",
                icon_font.clone(),
                tag_color,
            );
        }

        if self.config.player.weapon_icon {
            painter.text(
                pos2(bl.x + half_width, bl.y),
                Align2::CENTER_TOP,
                player.weapon.to_icon(),
                icon_font.clone(),
                weapon_color,
            );
            if player.ammo.0 >= 0 {
                self.text_sized(
                    painter,
                    format!("{}/{}", player.ammo.0, player.ammo.1),
                    pos2(bl.x + half_width, bl.y + font_size),
                    Align2::CENTER_TOP,
                    Some(weapon_color),
                    font_size,
                );
            }
        }
    }

    fn skeleton(&self, painter: &Painter, player: &PlayerData, data: &Data, alpha: Option<f32>) {
        let distance = data
            .local_player
            .position
            .distance(player.position)
            .max(1.0);
        // Same clamp as player_box to maintain visual consistency at range.
        let esp_scale = (500.0 / distance).clamp(0.4, 1.0);

        let mut color = match &self.config.player.draw_skeleton {
            DrawMode::None => return,
            DrawMode::Health => {
                self.health_color(player.health, self.config.player.skeleton_color.a())
            }
            DrawMode::Color => self.config.player.skeleton_color,
        };
        if let Some(alpha) = alpha {
            color = Self::alpha(color, alpha);
        }
        let stroke = Stroke::new(self.config.hud.line_width * esp_scale, color);

        for (a, b) in &Bones::CONNECTIONS {
            let Some(a) = player.bones.get(a) else {
                continue;
            };
            let Some(b) = player.bones.get(b) else {
                continue;
            };

            let Some(a) = world_to_screen(a, data) else {
                continue;
            };
            let Some(b) = world_to_screen(b, data) else {
                continue;
            };

            painter.line(vec![a, b], stroke);
        }

        // head circle
        if !self.config.player.head_circle {
            return;
        }
        let Some(neck) = player.bones.get(&Bones::Neck) else {
            return;
        };
        let Some(spine) = player.bones.get(&Bones::Spine3) else {
            return;
        };

        let Some(neck) = world_to_screen(neck, data) else {
            return;
        };
        let Some(spine) = world_to_screen(spine, data) else {
            return;
        };

        let height = spine.y - neck.y;
        let pos = pos2(neck.x - (spine.x - neck.x) / 2.0, neck.y - height / 2.0);
        painter.circle_stroke(pos, height / 2.0, stroke);
    }

    /// Draw animated expanding rings at the player's feet when a footstep was recently heard.
    /// Three rings staggered in phase create a ripple / sonar wave effect.
    fn draw_sound_rings(
        &self,
        painter: &Painter,
        player: &PlayerData,
        sound: Option<&(Instant, SoundType)>,
        data: &Data,
    ) {
        let Some((time, sound_type)) = sound else {
            return;
        };
        if *sound_type != SoundType::Footstep {
            return;
        }

        // Check distance gate (same as sound_alpha uses)
        let max_dist = self.config.player.sound.footstep_diameter;
        if data.local_player.position.distance(player.position) > max_dist {
            return;
        }

        let Some(feet_screen) = world_to_screen(&player.position, data) else {
            return;
        };

        let elapsed = time.elapsed().as_secs_f32();
        let total = self.total_sound_duration().as_secs_f32();
        if elapsed >= total {
            return;
        }

        // Global fade: 1.0 at start → 0.0 at end of sound duration
        let global_alpha = 1.0 - (elapsed / total);

        const NUM_RINGS: usize = 3;
        const RING_PERIOD: f32 = 0.7; // seconds for one ring to expand fully
        const MAX_RADIUS: f32 = 22.0;
        const STROKE_WIDTH: f32 = 1.5;

        for i in 0..NUM_RINGS {
            let phase_offset = i as f32 * (RING_PERIOD / NUM_RINGS as f32);
            let t = ((elapsed - phase_offset).max(0.0) % RING_PERIOD) / RING_PERIOD;
            let radius = t * MAX_RADIUS;
            let ring_alpha = (1.0 - t) * global_alpha;
            if ring_alpha <= 0.01 {
                continue;
            }

            let a = (ring_alpha * 220.0) as u8;
            let color = Color32::from_rgba_unmultiplied(80, 200, 255, a);
            painter.circle_stroke(feet_screen, radius, Stroke::new(STROKE_WIDTH, color));
        }
    }

    pub fn update_player_sounds(&mut self) {        let data = self.data.lock();

        for player in &data.players {
            let Some(sound) = &player.sound else {
                continue;
            };

            // Only create a NEW sound event when:
            //   1. There is no existing entry (first time we detect this player making sound)
            //   2. The sound TYPE changed (e.g. walking → shooting)
            //   3. Enough time has passed that the previous ring cycle has finished
            //      animating (each ring takes RING_PERIOD=0.7s; after ~0.6s we restart)
            //
            // Overwriting Instant::now() every frame kept elapsed ≈ 0 forever,
            // which locked ring radius to 0 — making footstep rings invisible.
            const RING_CYCLE: Duration = Duration::from_millis(600);
            let needs_insert = match self.player_sounds.get(&player.steam_id) {
                None => true,
                Some((prev_time, prev_sound)) => {
                    *prev_sound != *sound || prev_time.elapsed() >= RING_CYCLE
                }
            };
            if needs_insert {
                self.player_sounds
                    .insert(player.steam_id, (Instant::now(), *sound));
            }
        }

        let total_duration = self.total_sound_duration();
        self.player_sounds
            .retain(|_, (time, _)| time.elapsed() < total_duration);
    }
}
