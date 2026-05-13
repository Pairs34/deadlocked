use glam::{Vec2, vec2};

use crate::{
    config::{Config, KeyMode},
    cs2::{
        CS2,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::{angles_to_fov, vec2_clamp},
    os::mouse::Mouse,
};

#[derive(Debug, Default)]
pub struct Aimbot {
    pub active: bool,
    /// Smoothed aiming velocity (mouse pixels/frame). Reset when aimbot is inactive.
    velocity: Vec2,
    /// Sub-pixel carry: integer truncation loses fractional pixels every frame.
    /// Accumulate the remainder so the aim actually reaches the target.
    unaccounted: Vec2,
    /// Pawn pointer of the target locked onto last frame. Used to detect
    /// target switches and reset the velocity/carry so the aim doesn't
    /// flick wildly when swapping to a new player.
    previous_target_pawn: Option<u64>,
}

/// Maximum mouse movement to emit in a single frame (pixels). Caps any
/// catastrophic jolt from a sudden target switch, crouch/stand transition,
/// teleport or aim-punch glitch. Excess is dropped (not carried) so the
/// smoother handles the rest naturally on subsequent frames.
const MAX_MOUSE_DELTA_PER_FRAME: f32 = 120.0;
/// If the desired angle delta exceeds this between two consecutive frames
/// (raw, before smoothing), treat it as a discontinuity (crouch/teleport
/// /respawn) and skip the frame to avoid jolting.
const VIEW_DISCONTINUITY_THRESHOLD: f32 = 12.0; // degrees

impl CS2 {
    pub fn aimbot(&mut self, config: &Config, mouse: &mut Mouse) -> bool {
        let hotkey = config.aim.aimbot_hotkey;
        let config = self.aimbot_config(config);

        if !config.enabled {
            self.aim.velocity = Vec2::ZERO;
            self.aim.unaccounted = Vec2::ZERO;
            self.aim.previous_target_pawn = None;
            return false;
        }

        let key_active = match config.mode {
            KeyMode::Hold => self.input.is_key_pressed(hotkey),
            KeyMode::Toggle => {
                if self.input.key_just_pressed(hotkey) {
                    self.aim.active = !self.aim.active;
                }
                self.aim.active
            }
            KeyMode::Always => true,
        };

        if !key_active {
            // Decay velocity so the aim drifts to a stop rather than cutting off abruptly
            self.aim.velocity *= 0.5;
            if self.aim.velocity.length() < 0.05 {
                self.aim.velocity = Vec2::ZERO;
            }
            self.aim.unaccounted = Vec2::ZERO;
            self.aim.previous_target_pawn = None;
            return false;
        }

        let Some(target) = &self.target.player else {
            self.aim.velocity = Vec2::ZERO;
            self.aim.previous_target_pawn = None;
            return false;
        };

        if !target.is_valid(self) {
            self.aim.velocity = Vec2::ZERO;
            self.aim.previous_target_pawn = None;
            return false;
        }

        // Target switch detection: if the locked pawn changed, blow away the
        // smoother state. Otherwise leftover momentum from the previous
        // target's trajectory causes a visible flick on the first frame.
        let target_pawn = target.pawn;
        let target_changed = self.aim.previous_target_pawn != Some(target_pawn);
        if target_changed {
            self.aim.velocity = Vec2::ZERO;
            self.aim.unaccounted = Vec2::ZERO;
        }

        let Some(local_player) = Player::local_player(self) else {
            self.aim.velocity = Vec2::ZERO;
            return false;
        };

        let weapon_class = local_player.weapon_class(self);
        let disallowed_weapons = [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
        ];
        if disallowed_weapons.contains(&weapon_class) {
            return false;
        }

        if config.flash_check && local_player.is_flashed(self) {
            return false;
        }

        if config.visibility_check && !target.visible(self, &local_player) {
            return false;
        }

        if local_player.shots_fired(self) < config.start_bullet {
            return false;
        }

        let target_angle = {
            let mut smallest_fov = 360.0;
            let mut smallest_angle = glam::Vec2::ZERO;
            for bone in &config.bones {
                let bone_pos = target.bone_position(self, bone.u64());
                let angle =
                    self.angle_to_target(&local_player, &bone_pos, &self.target.previous_aim_punch);
                let fov = angles_to_fov(&local_player.view_angles(self), &angle);
                if fov < smallest_fov {
                    smallest_fov = fov;
                    smallest_angle = angle;
                }
            }

            smallest_angle
        };

        let view_angles = local_player.view_angles(self);
        if angles_to_fov(&view_angles, &target_angle)
            > (config.fov
                * if config.distance_adjusted_fov {
                    self.distance_scale(self.target.distance)
                } else {
                    1.0
                })
        {
            return false;
        }

        let mut aim_angles = view_angles - target_angle;
        if aim_angles.y < -180.0 {
            aim_angles.y += 360.0
        }
        vec2_clamp(&mut aim_angles);

        // View discontinuity guard: if the raw angle delta this frame is
        // huge AND we were already tracking the same target last frame,
        // something jolted (crouch, teleport, respawn, aim-punch spike).
        // Skip the move to absorb the spike — the smoother will catch up.
        if !target_changed && aim_angles.length() > VIEW_DISCONTINUITY_THRESHOLD {
            self.aim.velocity *= 0.25;
            self.aim.unaccounted = Vec2::ZERO;
            self.aim.previous_target_pawn = Some(target_pawn);
            return false;
        }

        let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);

        // Desired velocity: angle delta converted to mouse pixels.
        let desired = vec2(
            aim_angles.y / sensitivity * 50.0,
            -aim_angles.x / sensitivity * 50.0,
        );

        // Exponential lerp factor derived from smooth setting.
        // smooth=0 → factor≈1.0 (instant snap), smooth=20 → factor≈0.4 (very smooth).
        let smooth = (config.smooth + 1.0).clamp(1.0, 21.0);
        let lerp_factor = (1.0 - (-8.0 / smooth).exp()).clamp(0.05, 1.0);

        // Blend current velocity toward desired velocity.
        self.aim.velocity = self.aim.velocity.lerp(desired, lerp_factor);

        // Prevent the smoothed velocity from exceeding the raw desired magnitude.
        // This avoids overshooting when the target suddenly disappears.
        let max_speed = desired.length() * 1.2 + 0.5;
        if self.aim.velocity.length() > max_speed && max_speed > 0.0 {
            self.aim.velocity = self.aim.velocity.normalize() * max_speed;
        }

        // Sub-pixel accumulator: keep fractional movement so the aim actually lands.
        let desired_move = self.aim.velocity + self.aim.unaccounted;
        let mut ready = Vec2::new(desired_move.x.trunc(), desired_move.y.trunc());
        self.aim.unaccounted = desired_move - ready;

        // Hard per-frame cap: under no circumstances emit a mouse delta
        // larger than MAX_MOUSE_DELTA_PER_FRAME. Anything beyond is dropped
        // (NOT carried) so we don't queue a delayed jolt — the smoother
        // will request the remainder on subsequent frames naturally.
        let ready_len = ready.length();
        if ready_len > MAX_MOUSE_DELTA_PER_FRAME {
            ready = ready * (MAX_MOUSE_DELTA_PER_FRAME / ready_len);
            ready = Vec2::new(ready.x.trunc(), ready.y.trunc());
            // Also clamp the smoother's internal velocity so it doesn't
            // keep trying to overshoot next frame.
            self.aim.velocity = self.aim.velocity.clamp_length_max(MAX_MOUSE_DELTA_PER_FRAME);
            self.aim.unaccounted = Vec2::ZERO;
        }

        self.aim.previous_target_pawn = Some(target_pawn);

        utils::debug!(
            "aimbot mouse movement: {:.2}/{:.2} (carry {:.2}/{:.2})",
            ready.x, ready.y,
            self.aim.unaccounted.x, self.aim.unaccounted.y
        );
        mouse.move_rel(&ready);

        // Auto-fire in Always mode: when the aim is dialed in tightly and no
        // shot is already queued, schedule one via the triggerbot pipeline so
        // we reuse its delay/duration randomness instead of slamming the
        // mouse the moment a target appears.
        if matches!(config.mode, KeyMode::Always)
            && self.trigger.shot_start.is_none()
            && self.trigger.shot_end.is_none()
        {
            let scale = if config.distance_adjusted_fov {
                self.distance_scale(self.target.distance)
            } else {
                1.0
            };
            let lock_fov = (1.5_f32).min(config.fov) * scale;
            if angles_to_fov(&view_angles, &target_angle) <= lock_fov {
                use std::time::{Duration, Instant};
                let now = Instant::now();
                let delay = Duration::from_millis(40);
                self.trigger.shot_start = Some(now + delay);
                self.trigger.shot_end =
                    Some(now + delay + Duration::from_millis(80));
            }
        }

        true
    }
}
