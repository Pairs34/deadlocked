use glam::Vec2;

use crate::{
    config::{Config, TargetingMode},
    constants::cs2,
    cs2::{
        CS2,
        bones::Bones,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::angles_to_fov,
};

#[derive(Debug, Default)]
pub struct Target {
    pub player: Option<Player>,
    pub angle: Vec2,
    pub distance: f32,
    pub bone_index: u64,
    pub local_pawn_index: u64,
    pub previous_aim_punch: Vec2,
}

impl Target {
    pub fn reset(&mut self) {
        *self = Target::default();
    }
}

impl CS2 {
    pub fn find_target(&mut self, config: &Config) {
        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        let team = local_player.team(self);
        if team != cs2::TEAM_CT && team != cs2::TEAM_T {
            self.target.reset();
            return;
        }

        let weapon_class = local_player.weapon_class(self);

        let view_angles = local_player.view_angles(self);
        let ffa = self.is_ffa();
        let shots_fired = local_player.shots_fired(self);
        let aim_punch = match (weapon_class, local_player.aim_punch(self) * 2.0) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length() == 0.0 && shots_fired > 1 => {
                self.target.previous_aim_punch
            }
            (_, punch) => punch,
        };
        self.target.previous_aim_punch = aim_punch;

        let aimbot_config = self.aimbot_config(config);
        let targeting_mode = &aimbot_config.targeting_mode;
        let max_fov = aimbot_config.fov;

        let mut best_fov = 360.0;
        let mut best_distance = f32::MAX;
        let eye_position = local_player.eye_position(self);

        if self.target.player.is_none() {
            self.target.reset();
        }
        if let Some(player) = &self.target.player
            && !player.is_valid(self)
        {
            self.target.reset();
        }

        if self.players.is_empty() {
            self.target.reset();
            return;
        }

        let target_friendlies = aimbot_config.target_friendlies;

        // ── Sticky target ─────────────────────────────────────────────────
        // If we already locked onto a target last frame and it's still:
        //   • Valid (alive, exists)
        //   • Within FOV (with distance scaling)
        //   • Visible (if visibility check is on)
        // then KEEP that target instead of constantly re-selecting whichever
        // happens to be closest to the crosshair this frame. This prevents
        // mid-spray flicks between two players standing close together.
        let max_fov_scaled = |dist: f32| max_fov * self.distance_scale(dist);

        if let Some(current) = self.target.player {
            let mut still_good = current.is_valid(self);
            if still_good && !(ffa || target_friendlies) && team == current.team(self) {
                still_good = false;
            }
            if still_good {
                // Find smallest-FOV bone of the current target
                let bones = &aimbot_config.bones;
                let mut cur_best_fov = 360.0_f32;
                let mut cur_best_ang = Vec2::ZERO;
                let mut cur_best_dist = f32::MAX;
                if bones.is_empty() {
                    let pos = current.bone_position(self, Bones::Head.u64());
                    let ang = self.angle_to_target(&local_player, &pos, &aim_punch);
                    cur_best_fov = angles_to_fov(&view_angles, &ang);
                    cur_best_ang = ang;
                    cur_best_dist = eye_position.distance(pos);
                } else {
                    for bone in bones {
                        let pos = current.bone_position(self, bone.u64());
                        let ang = self.angle_to_target(&local_player, &pos, &aim_punch);
                        let f = angles_to_fov(&view_angles, &ang);
                        if f < cur_best_fov {
                            cur_best_fov = f;
                            cur_best_ang = ang;
                            cur_best_dist = eye_position.distance(pos);
                        }
                    }
                }
                if cur_best_fov <= max_fov_scaled(cur_best_dist) {
                    // Keep this target; update angle/distance and skip selection loop
                    self.target.angle = cur_best_ang;
                    self.target.distance = cur_best_dist;
                    self.target.bone_index = Bones::Head.u64();
                    return;
                }
                // During an active spray ALWAYS keep the current target as long as
                // it is alive. Recoil naturally pushes the crosshair off-target; the
                // smoothed aim will bring it back. Never let a nearby enemy steal the
                // lock mid-burst — the user chose this target to spray.
                if shots_fired > 1 {
                    self.target.angle = cur_best_ang;
                    self.target.distance = cur_best_dist;
                    self.target.bone_index = Bones::Head.u64();
                    return;
                }
            }
            // Current target no longer suitable → clear and re-select below
            self.target.reset();
        }

        for player in &self.players {
            if !(ffa || target_friendlies) && team == player.team(self) {
                continue;
            }

            // Use the configured aim bones (e.g. Head + Neck) for FOV evaluation.
            // This prevents selecting a player just because their head is in FOV
            // while the user is actually aiming near a different player's body.
            let bones = &aimbot_config.bones;
            let (fov, angle, distance) = if bones.is_empty() {
                // Fallback to head when no bones configured
                let pos = player.bone_position(self, Bones::Head.u64());
                let ang = self.angle_to_target(&local_player, &pos, &aim_punch);
                let dist = eye_position.distance(pos);
                (angles_to_fov(&view_angles, &ang), ang, dist)
            } else {
                let mut best_fov = 360.0_f32;
                let mut best_ang = Vec2::ZERO;
                let mut best_dist = f32::MAX;
                for bone in bones {
                    let pos = player.bone_position(self, bone.u64());
                    let ang = self.angle_to_target(&local_player, &pos, &aim_punch);
                    let f = angles_to_fov(&view_angles, &ang);
                    if f < best_fov {
                        best_fov = f;
                        best_ang = ang;
                        best_dist = eye_position.distance(pos);
                    }
                }
                (best_fov, best_ang, best_dist)
            };

            let fov_limit = max_fov * self.distance_scale(distance);
            if fov > fov_limit {
                continue;
            }

            let should_select = match targeting_mode {
                TargetingMode::Fov => fov < best_fov,
                TargetingMode::Distance => distance < best_distance,
            };

            if should_select {
                best_fov = fov;
                best_distance = distance;

                self.target.player = Some(*player);
                self.target.angle = angle;
                self.target.distance = distance;
                self.target.bone_index = Bones::Head.u64();
            }
        }

        let Some(target) = &self.target.player else {
            return;
        };

        // update target angle — respect the user's configured bones so that
        // disabled regions (e.g. only torso enabled) are never targeted, and
        // enabled regions (e.g. feet) are picked when closest to the crosshair.
        let mut smallest_fov = 360.0;
        let configured_bones: Vec<Bones> = if aimbot_config.bones.is_empty() {
            vec![Bones::Head]
        } else {
            aimbot_config.bones.clone()
        };
        for bone in &configured_bones {
            let bone_position = target.bone_position(self, bone.u64());
            let distance = eye_position.distance(bone_position);
            let angle = self.angle_to_target(&local_player, &bone_position, &aim_punch);
            let fov = angles_to_fov(&view_angles, &angle);

            if fov < smallest_fov {
                smallest_fov = fov;

                self.target.angle = angle;
                self.target.distance = distance;
                self.target.bone_index = bone.u64();
            }
        }
        /*
        let head_position = self.get_bone_position(process, self.target.pawn, Bones::Head.u64());
        let distance = eye_position.distance(head_position);
        let angle = self.get_target_angle(process, local_pawn, head_position, aim_punch);

        self.target.angle = angle;
        self.target.distance = distance;
        self.target.bone_index = Bones::Head.u64();
        */
    }
}
