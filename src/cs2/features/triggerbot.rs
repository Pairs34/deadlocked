use std::time::{Duration, Instant};

use glam::Vec2;
use rand::rng;

use crate::{
    config::{Config, KeyMode},
    cs2::{
        CS2,
        bones::Bones,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::angles_to_fov,
    os::mouse::Mouse,
};

#[derive(Debug, Default)]
pub struct Triggerbot {
    pub shot_start: Option<Instant>,
    pub shot_end: Option<Instant>,
    pub active: bool,
}

impl CS2 {
    pub fn triggerbot(&mut self, config: &Config) {
        let hotkey = config.aim.triggerbot_hotkey;
        let config = self.triggerbot_config(config);

        if !config.enabled {
            return;
        }

        match config.mode {
            KeyMode::Hold => {
                if !self.input.is_key_pressed(hotkey) {
                    return;
                }
            }
            KeyMode::Toggle => {
                if self.input.key_just_pressed(hotkey) {
                    self.trigger.active = !self.trigger.active;
                }
                if !self.trigger.active {
                    return;
                }
            }
            KeyMode::Always => {} // always active; no key check needed
        }

        if self.trigger.shot_start.is_some() || self.trigger.shot_end.is_some() {
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        if config.flash_check && local_player.is_flashed(self) {
            return;
        }

        if config.scope_check
            && local_player.weapon_class(self) == WeaponClass::Sniper
            && !local_player.is_scoped(self)
        {
            return;
        }

        if config.velocity_check && local_player.velocity(self).length() > config.velocity_threshold
        {
            return;
        }

        let player = match local_player.crosshair_entity(self) {
            Some(p) => p,
            None if config.wallbang => {
                // Wallbang: the game says nothing is under the crosshair
                // because a wall is in the way. Pick the enemy whose nearest
                // bone is closest to the aim line, regardless of LoS — the
                // shot will be fired and the game decides if the bullet
                // penetrates.
                let view_angles = local_player.view_angles(self);
                let mut best: Option<(Player, f32)> = None;
                let bones = &[
                    Bones::Head,
                    Bones::Neck,
                    Bones::Spine3,
                    Bones::Spine1,
                    Bones::Hip,
                ];
                for player in self.players.iter().copied() {
                    if player.pawn == local_player.pawn {
                        continue;
                    }
                    if !player.is_valid(self) {
                        continue;
                    }
                    if !self.is_ffa() && player.team(self) == local_player.team(self) {
                        continue;
                    }
                    for bone in bones {
                        let pos = player.bone_position(self, bone.u64());
                        let angle = self.angle_to_target(&local_player, &pos, &Vec2::ZERO);
                        let fov = angles_to_fov(&view_angles, &angle);
                        // ~3° cone, scaled looser at distance
                        let dist =
                            (local_player.position(self) - player.position(self)).length();
                        let threshold = 3.0 + (dist / 1000.0).min(3.0);
                        if fov <= threshold {
                            match best {
                                None => best = Some((player, fov)),
                                Some((_, bf)) if fov < bf => best = Some((player, fov)),
                                _ => {}
                            }
                        }
                    }
                }
                let Some((p, _)) = best else {
                    return;
                };
                p
            }
            None => return,
        };

        if !self.is_ffa() && player.team(self) == local_player.team(self) {
            return;
        }

        if config.head_only {
            let head = player.bone_position(self, Bones::Head.u64());

            let target_angle = self.angle_to_target(&local_player, &head, &Vec2::ZERO);
            let view_angles = local_player.view_angles(self);
            let fov = angles_to_fov(&view_angles, &target_angle);

            let head_radius_fov =
                3.5 / (local_player.position(self) - player.position(self)).length() * 100.0;

            if fov > head_radius_fov {
                return;
            }
        }

        let mean = (*config.delay.start() + *config.delay.end()) as f32 / 2.0;
        // Ensure std_dev is positive so Normal::new never fails (happens when start == end).
        let std_dev = ((*config.delay.end() - *config.delay.start()) as f32 / 2.0).max(1.0);

        let normal = rand_distr::Normal::new(mean, std_dev).expect("std_dev >= 1.0");
        use rand_distr::Distribution as _;
        let delay = normal.sample(&mut rng()).max(0.0) as u64;

        let now = Instant::now();
        let delay = Duration::from_millis(delay);
        self.trigger.shot_start = Some(now + delay);
        self.trigger.shot_end = Some(now + delay + Duration::from_millis(config.shot_duration));
    }

    pub fn triggerbot_shoot(&mut self, mouse: &mut Mouse) {
        let now = Instant::now();

        if let Some(shot_time) = self.trigger.shot_start
            && now >= shot_time
        {
            mouse.left_press();
            self.trigger.shot_start = None;
        }

        if let Some(shot_end) = self.trigger.shot_end
            && now >= shot_end
        {
            mouse.left_release();
            self.trigger.shot_end = None;
        }
    }
}
