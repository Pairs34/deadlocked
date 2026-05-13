use std::time::Instant;

use egui::{Align2, Color32, FontId, Painter, Pos2, Stroke};

use crate::{
    cs2::entity::{
        EntityInfo, GrenadeInfo, inferno::InfernoInfo, molotov::MolotovInfo, smoke::SmokeInfo,
    },
    data::Data,
    math::world_to_screen,
    ui::{app::App, overlay::convex_hull, trail::Trail},
};

impl App {
    pub fn draw_entity(&self, painter: &Painter, entity: &EntityInfo, data: &Data) {
        match entity {
            EntityInfo::Weapon {
                weapon,
                position,
                ammo,
            } => {
                if !self.config.hud.dropped_weapons {
                    return;
                }
                let Some(position) = world_to_screen(position, data) else {
                    return;
                };
                // Use the custom icon font so we render a proper weapon icon
                // instead of a plain-text name. The icon font is in the
                // Monospace family (inserted at index 0 in prep_ctx).
                let icon = weapon.to_icon();
                let icon_font = FontId::monospace(self.config.hud.icon_size);
                let text_color = self.config.hud.text_color;
                if icon.is_empty() {
                    // Unknown weapon — fall back to text name
                    self.text(painter, format!("{weapon}"), position, Align2::CENTER_CENTER, None);
                } else {
                    painter.text(position, Align2::CENTER_CENTER, icon, icon_font, text_color);
                }
                if ammo.0 >= 0 {
                    self.text_sized(
                        painter,
                        format!("{}/{}", ammo.0, ammo.1),
                        egui::pos2(position.x, position.y + self.config.hud.icon_size),
                        Align2::CENTER_CENTER,
                        None,
                        self.config.hud.font_size,
                    );
                }
            }
            EntityInfo::Inferno(inferno) => self.inferno(painter, data, inferno),
            EntityInfo::Smoke(smoke) => self.smoke(painter, data, smoke),
            EntityInfo::Molotov(molotov) => self.molotov(painter, data, molotov),
            EntityInfo::Flashbang(info) => {
                self.draw_grenade(painter, data, info, self.config.hud.flash_trail_color)
            }
            EntityInfo::HeGrenade(info) => {
                self.draw_grenade(painter, data, info, self.config.hud.he_trail_color)
            }
            EntityInfo::Decoy(info) => {
                self.draw_grenade(painter, data, info, self.config.hud.decoy_trail_color)
            }
        };
    }

    fn draw_grenade(
        &self,
        painter: &Painter,
        data: &Data,
        info: &GrenadeInfo,
        trail_color: Color32,
    ) {
        if !self.config.hud.grenade_trails {
            return;
        }
        let Some(position) = world_to_screen(&info.position, data) else {
            return;
        };
        self.text(painter, info.name, position, Align2::CENTER_CENTER, None);

        if !self.config.hud.grenade_trails {
            return;
        }

        let stroke = Stroke::new(self.config.hud.line_width, trail_color);
        let Some(trail) = self.trails.get(&info.entity) else {
            return;
        };
        for window in trail.trail.windows(2) {
            if let [v1, v2] = window {
                use crate::math::world_to_screen;

                let Some(v1) = world_to_screen(v1, data) else {
                    continue;
                };
                let Some(v2) = world_to_screen(v2, data) else {
                    continue;
                };
                painter.line_segment([v1, v2], stroke);
            }
        }
    }

    fn inferno(&self, painter: &Painter, data: &Data, inferno: &InfernoInfo) {
        use egui::Shape;

        if !self.config.hud.grenade_trails {
            return;
        }
        let hull: Vec<Pos2> = convex_hull(&inferno.hull)
            .iter()
            .filter_map(|p| {
                use crate::math::world_to_screen;

                let p = p + (p - inferno.position).clamp_length(60.0, 60.0);
                world_to_screen(&p, data)
            })
            .collect();
        if hull.len() < 3 {
            return;
        }

        let shape = Shape::convex_polygon(
            hull,
            Color32::from_rgba_unmultiplied(255, 0, 0, 127),
            Stroke::NONE,
        );
        painter.add(shape);

        self.draw_grenade(painter, data, &inferno.grenade(), Color32::TRANSPARENT);
    }

    fn smoke(&self, painter: &Painter, data: &Data, smoke: &SmokeInfo) {
        if !self.config.hud.grenade_trails {
            return;
        }
        self.draw_grenade(
            painter,
            data,
            &smoke.grenade(),
            self.config.hud.smoke_trail_color,
        );
    }

    fn molotov(&self, painter: &Painter, data: &Data, molotov: &MolotovInfo) {
        if !self.config.hud.grenade_trails {
            return;
        }
        if molotov.is_incendiary {
            self.draw_grenade(
                painter,
                data,
                &molotov.grenade(),
                self.config.hud.incendiary_trail_color,
            );
        } else {
            self.draw_grenade(
                painter,
                data,
                &molotov.grenade(),
                self.config.hud.molotov_trail_color,
            );
        }
    }

    pub fn update_trails(&mut self) {
        let data = self.data.lock();
        for entity in &data.entities {
            let (entity, position) = match entity {
                EntityInfo::Inferno(info) => (info.entity, info.position),
                EntityInfo::Smoke(info) => (info.entity, info.position),
                EntityInfo::Molotov(info) => (info.entity, info.position),
                EntityInfo::Flashbang(info) | EntityInfo::HeGrenade(info) => {
                    (info.entity, info.position)
                }
                _ => continue,
            };
            if let Some(trail) = self.trails.get_mut(&entity) {
                if (position - trail.trail.last().unwrap()).length() < 1.0 {
                    continue;
                }
                trail.trail.push(position);
                trail.last_update = Instant::now();
            } else {
                self.trails.insert(
                    entity,
                    Trail {
                        trail: vec![position],
                        last_update: Instant::now(),
                    },
                );
            }
        }

        let now = Instant::now();
        self.trails
            .retain(|_k, trail| now.duration_since(trail.last_update) < Trail::MAX_AGE);
    }
}
