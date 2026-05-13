use glam::Vec2;

use crate::{
    config::{Config, RcsMode},
    cs2::{
        CS2,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    os::mouse::Mouse,
};

#[derive(Debug, Default)]
pub struct Recoil {
    pub previous: Vec2,
    pub unaccounted: Vec2,
    /// Smoothed correction velocity (exponential moving average of the raw delta).
    pub velocity: Vec2,
    /// Weapon class last frame. Used to wipe the smoother when the player
    /// switches weapons so stale `previous` punch from the old gun doesn't
    /// produce a one-frame jolt on the next trigger pull.
    pub previous_weapon: Option<WeaponClass>,
}

impl Recoil {
    pub fn reset(&mut self) {
        *self = Recoil::default();
    }
}

impl CS2 {
    pub fn rcs(&mut self, config: &Config, mouse: &mut Mouse) {
        let config = self.rcs_config(config);

        if !config.enabled {
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        let weapon_class = local_player.weapon_class(self);
        let disallowed_weapons = [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
            WeaponClass::Pistol,
            WeaponClass::Shotgun,
        ];
        if disallowed_weapons.contains(&weapon_class) {
            // Wipe smoother so the next allowed-weapon pull starts clean.
            self.recoil.reset();
            self.recoil.previous_weapon = Some(weapon_class);
            return;
        }

        // Weapon switch detection: stale `previous` punch from the previous
        // gun (different recoil cache) would produce a phantom delta on the
        // first frame of the new gun. Reset on transition.
        if self.recoil.previous_weapon != Some(weapon_class) {
            self.recoil.reset();
            self.recoil.previous_weapon = Some(weapon_class);
        }

        let shots_fired = local_player.shots_fired(self);
        let aim_punch = match (weapon_class, local_player.aim_punch(self)) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length() == 0.0 && shots_fired > 1 => self.recoil.previous,
            (_, punch) => punch,
        };

        // Aiming mode: only activate when the player is actively firing.
        // Do NOT couple to self.target.player — that caused RCS to activate
        // in sync with the aimbot lock, compounding aim-toward-target with
        // recoil correction and producing unwanted aimlock behaviour.
        if config.mode == RcsMode::Aiming {
            if shots_fired == 0 {
                self.recoil.reset();
                return;
            }
        }

        let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);

        // Guard against large spurious deltas caused by stance changes (crouch/stand)
        // or punch cache resets. Check in ANGULAR units (degrees), not screen pixels,
        // so the threshold is independent of sensitivity & FOV.
        // Real per-frame recoil angle deltas never exceed ~1.5°; anything bigger is
        // an artifact — skip the frame and resync previous.
        let angle_delta = aim_punch - self.recoil.previous;
        const MAX_ANGLE_DELTA: f32 = 1.5;
        if angle_delta.length() > MAX_ANGLE_DELTA {
            self.recoil.previous = aim_punch;
            return;
        }

        // CS2: aim_punch is stored as half the actual recoil angle — multiply by 2.0
        let delta = Vec2::new(
            angle_delta.y * 2.0 / sensitivity * 100.0,
            -angle_delta.x * 2.0 / sensitivity * 100.0,
        );

        // ── Unified compensation path ──────────────────────────────────────
        // Whether the punch is growing (firing) or decaying (post-fire), the
        // same `strength` × `smoothing` filter applies. Previously the decay
        // path bypassed both filters and applied raw `delta`, which over-
        // shot the windup phase and pushed the aim upward after firing.
        // Using one symmetric formula guarantees the net cumulative motion
        // is zero: whatever fraction we pulled down during firing, we push
        // back up by the same fraction during decay.
        let raw = delta * config.strength.clamp(Vec2::ZERO, Vec2::ONE);

        let smooth = config.smoothing.clamp(0.0, 0.95);
        self.recoil.velocity = if smooth > 0.0 {
            self.recoil.velocity * smooth + raw * (1.0 - smooth)
        } else {
            raw
        };

        // When not actively firing AND the smoother has bled out AND the
        // punch is fully decayed, fully reset so the next spray starts clean.
        if shots_fired < 1
            && aim_punch.length_squared() < 0.001
            && self.recoil.velocity.length_squared() < 0.001
        {
            self.recoil.previous = aim_punch;
            self.recoil.unaccounted = Vec2::ZERO;
            self.recoil.velocity = Vec2::ZERO;
            return;
        }

        let desired = self.recoil.velocity + self.recoil.unaccounted;
        self.recoil.previous = aim_punch;
        let mut ready = Vec2::new(desired.x.trunc(), desired.y.trunc());
        self.recoil.unaccounted = desired - ready;

        // Per-frame mouse delta cap: prevents a single bizarre frame from
        // producing a visible cursor jolt (e.g. punch table corruption,
        // first-frame after rapid weapon swap that slipped the guard).
        const RCS_MAX_DELTA_PER_FRAME: f32 = 80.0;
        let ready_len = ready.length();
        if ready_len > RCS_MAX_DELTA_PER_FRAME {
            ready = ready * (RCS_MAX_DELTA_PER_FRAME / ready_len);
            ready = Vec2::new(ready.x.trunc(), ready.y.trunc());
            self.recoil.unaccounted = Vec2::ZERO;
            self.recoil.velocity = self
                .recoil
                .velocity
                .clamp_length_max(RCS_MAX_DELTA_PER_FRAME);
        }

        utils::debug!(
            "rcs: delta={:.2}/{:.2} vel={:.2}/{:.2}",
            delta.x, delta.y,
            self.recoil.velocity.x, self.recoil.velocity.y
        );
        mouse.move_rel(&ready)
    }
}


