use egui::{Align, Layout, RichText, Ui};
use strum::IntoEnumIterator as _;

use crate::{
    config::WeaponConfig,
    cs2::entity::weapon::Weapon,
    ui::{
        app::App,
        color::Colors,
        drag_range::DragRange,
        gui::{
            body_picker,
            components::{
                self, chip, matches_filter, segmented, setting_row, slider, sub_label, switch,
                titled_card,
            },
            helpers::{keybind, scroll},
        },
    },
};

#[derive(PartialEq)]
pub enum AimbotTab {
    Global,
    Weapon,
}

impl App {
    pub fn aimbot_settings(&mut self, ui: &mut Ui) {
        let l = self.lang();
        let current_weapon = self.data.lock().weapon.clone();
        let filter = self.search_query.clone();

        // ── Header: scope (Global/Weapon), weapon picker, preview chips ──
        ui.horizontal(|ui| {
            let mut scope = self.aimbot_tab == AimbotTab::Weapon;
            if ui.selectable_label(!scope, RichText::new(l.global).size(13.0)).clicked() {
                self.aimbot_tab = AimbotTab::Global;
            }
            if ui.selectable_label(scope, RichText::new(l.weapon).size(13.0)).clicked() {
                self.aimbot_tab = AimbotTab::Weapon;
                scope = true;
            }

            if scope {
                ui.add_space(8.0);
                let selected_text = if self.aimbot_weapon == current_weapon {
                    RichText::new(format!("{:?} ★", self.aimbot_weapon))
                        .color(ui.visuals().hyperlink_color)
                } else {
                    RichText::new(format!("{:?}", self.aimbot_weapon))
                };
                egui::ComboBox::new("aimbot_weapon", "")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        for w in Weapon::iter().filter(|w| w.is_relevant()) {
                            let is_current = w == current_weapon;
                            let text = if is_current {
                                RichText::new(format!("{:?} ★", w))
                                    .color(ui.visuals().hyperlink_color)
                            } else {
                                RichText::new(format!("{:?}", w))
                            };
                            ui.selectable_value(&mut self.aimbot_weapon, w, text);
                        }
                    });

                if ui
                    .button("↺")
                    .on_hover_text("Bu silahın ayarlarını sıfırla")
                    .clicked()
                {
                    self.config
                        .aim
                        .weapons
                        .insert(self.aimbot_weapon.clone(), WeaponConfig::reset());
                    self.send_config();
                }
                if ui
                    .button("↺↺")
                    .on_hover_text("Tüm silahların ayarlarını sıfırla")
                    .clicked()
                {
                    for w in Weapon::iter() {
                        self.config.aim.weapons.insert(w, WeaponConfig::reset());
                    }
                    self.send_config();
                }
            }

            // Preview chips at the right edge.
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let wc = self.weapon_config();
                chip(
                    ui,
                    &format!("smooth {:.1}", wc.aimbot.smooth),
                    Colors::TEAL,
                    false,
                );
                chip(
                    ui,
                    &format!("fov {:.1}°", wc.aimbot.fov),
                    Colors::BLUE,
                    false,
                );
                let mode_color = if wc.aimbot.enabled {
                    Colors::GREEN
                } else {
                    Colors::SUBTEXT
                };
                chip(ui, &format!("{:?}", wc.aimbot.mode), mode_color, wc.aimbot.enabled);
            });
        });

        ui.add_space(8.0);

        ui.columns(2, |cols| {
            let left = &mut cols[0];
            scroll(left, "aimbot_left", |ui| self.aimbot_left(ui, &filter));
            let right = &mut cols[1];
            scroll(right, "aimbot_right", |ui| self.aimbot_right(ui, &filter));
        });
    }

    fn aimbot_left(&mut self, ui: &mut Ui, filter: &str) {
        let l = self.lang();
        let is_weapon = self.aimbot_tab == AimbotTab::Weapon;

        titled_card(ui, l.section_aimbot, |ui| {
            if matches_filter(filter, l.hotkey)
                && keybind(ui, "aimbot_hotkey", l.hotkey, &mut self.config.aim.aimbot_hotkey)
            {
                self.send_config();
            }
            if is_weapon && matches_filter(filter, l.enable_override) {
                let mut v = self.weapon_config().aimbot.enable_override;
                if setting_row(ui, l.enable_override, Some(l.tip_enable_override), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.weapon_config().aimbot.enable_override = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.enable) {
                let mut v = self.weapon_config().aimbot.enabled;
                if setting_row(ui, l.enable, Some(l.tip_aimbot_enable), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.weapon_config().aimbot.enabled = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.mode) {
                sub_label(ui, l.mode);
                if segmented(ui, &mut self.weapon_config().aimbot.mode, "aimbot_mode") {
                    self.send_config();
                }
            }
        });

        titled_card(ui, l.section_targeting, |ui| {
            if matches_filter(filter, l.aimbot_target_friendlies) {
                let mut v = self.weapon_config().aimbot.target_friendlies;
                if setting_row(
                    ui,
                    l.aimbot_target_friendlies,
                    Some(l.tip_aimbot_target_friendlies),
                    |ui| switch(ui, &mut v).changed(),
                ) {
                    self.weapon_config().aimbot.target_friendlies = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.aimbot_distance_fov) {
                let mut v = self.weapon_config().aimbot.distance_adjusted_fov;
                if setting_row(
                    ui,
                    l.aimbot_distance_fov,
                    Some(l.tip_aimbot_distance_fov),
                    |ui| switch(ui, &mut v).changed(),
                ) {
                    self.weapon_config().aimbot.distance_adjusted_fov = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.aimbot_fov)
                && setting_row(ui, l.aimbot_fov, Some(l.tip_aimbot_fov), |ui| {
                    slider(ui, &mut self.weapon_config().aimbot.fov, 0.1..=360.0, "°", 1)
                })
            {
                self.send_config();
            }
            if matches_filter(filter, l.aimbot_smooth)
                && setting_row(ui, l.aimbot_smooth, Some(l.tip_aimbot_smooth), |ui| {
                    slider(ui, &mut self.weapon_config().aimbot.smooth, 0.0..=20.0, "", 1)
                })
            {
                self.send_config();
            }
            if matches_filter(filter, l.aimbot_start_bullet)
                && setting_row(
                    ui,
                    l.aimbot_start_bullet,
                    Some(l.tip_aimbot_start_bullet),
                    |ui| {
                        slider(
                            ui,
                            &mut self.weapon_config().aimbot.start_bullet,
                            0.0..=10.0,
                            "",
                            0,
                        )
                    },
                )
            {
                self.send_config();
            }
            if matches_filter(filter, l.aimbot_targeting_mode) {
                sub_label(ui, l.aimbot_targeting_mode);
                if segmented(
                    ui,
                    &mut self.weapon_config().aimbot.targeting_mode,
                    "targeting_mode",
                ) {
                    self.send_config();
                }
            }
        });

        titled_card(ui, l.section_checks, |ui| {
            if matches_filter(filter, l.aimbot_visibility_check) {
                let mut v = self.weapon_config().aimbot.visibility_check;
                if setting_row(
                    ui,
                    l.aimbot_visibility_check,
                    Some(l.tip_aimbot_visibility),
                    |ui| switch(ui, &mut v).changed(),
                ) {
                    self.weapon_config().aimbot.visibility_check = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.aimbot_flash_check) {
                let mut v = self.weapon_config().aimbot.flash_check;
                if setting_row(ui, l.aimbot_flash_check, Some(l.tip_aimbot_flash), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.weapon_config().aimbot.flash_check = v;
                    self.send_config();
                }
            }
        });

        titled_card(ui, l.section_bones, |ui| {
            ui.vertical_centered(|ui| {
                if body_picker::body_picker(ui, &mut self.weapon_config().aimbot.bones) {
                    self.send_config();
                }
            });
        });
    }

    fn aimbot_right(&mut self, ui: &mut Ui, filter: &str) {
        let l = self.lang();
        let is_weapon = self.aimbot_tab == AimbotTab::Weapon;

        titled_card(ui, l.section_triggerbot, |ui| {
            if is_weapon && matches_filter(filter, l.enable_override) {
                let mut v = self.weapon_config().triggerbot.enable_override;
                if setting_row(ui, l.enable_override, None, |ui| switch(ui, &mut v).changed()) {
                    self.weapon_config().triggerbot.enable_override = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.enable) {
                let mut v = self.weapon_config().triggerbot.enabled;
                if setting_row(ui, l.enable, Some(l.tip_triggerbot_enable), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.weapon_config().triggerbot.enabled = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.hotkey)
                && keybind(
                    ui,
                    "triggerbot_hotkey",
                    l.hotkey,
                    &mut self.config.aim.triggerbot_hotkey,
                )
            {
                self.send_config();
            }
            if matches_filter(filter, l.triggerbot_delay)
                && ui
                    .add(DragRange::new(
                        l.triggerbot_delay,
                        &mut self.weapon_config().triggerbot.delay,
                        0..=999,
                    ))
                    .on_hover_text(l.tip_triggerbot_delay)
                    .changed()
            {
                self.send_config();
            }
            if matches_filter(filter, l.mode) {
                sub_label(ui, l.mode);
                if segmented(
                    ui,
                    &mut self.weapon_config().triggerbot.mode,
                    "triggerbot_mode",
                ) {
                    self.send_config();
                }
            }
            if matches_filter(filter, l.triggerbot_head_only) {
                let mut v = self.weapon_config().triggerbot.head_only;
                if setting_row(
                    ui,
                    l.triggerbot_head_only,
                    Some(l.tip_triggerbot_head_only),
                    |ui| switch(ui, &mut v).changed(),
                ) {
                    self.weapon_config().triggerbot.head_only = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.triggerbot_wallbang) {
                let mut v = self.weapon_config().triggerbot.wallbang;
                if setting_row(
                    ui,
                    l.triggerbot_wallbang,
                    Some(l.tip_triggerbot_wallbang),
                    |ui| switch(ui, &mut v).changed(),
                ) {
                    self.weapon_config().triggerbot.wallbang = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.triggerbot_shot_duration)
                && setting_row(
                    ui,
                    l.triggerbot_shot_duration,
                    Some(l.tip_triggerbot_shot_duration),
                    |ui| {
                        slider(
                            ui,
                            &mut self.weapon_config().triggerbot.shot_duration,
                            0.0..=2000.0,
                            " ms",
                            0,
                        )
                    },
                )
            {
                self.send_config();
            }
        });

        titled_card(ui, l.section_checks, |ui| {
            if matches_filter(filter, l.triggerbot_flash_check) {
                let mut v = self.weapon_config().triggerbot.flash_check;
                if setting_row(
                    ui,
                    l.triggerbot_flash_check,
                    Some(l.tip_triggerbot_flash),
                    |ui| switch(ui, &mut v).changed(),
                ) {
                    self.weapon_config().triggerbot.flash_check = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.triggerbot_scope_check) {
                let mut v = self.weapon_config().triggerbot.scope_check;
                if setting_row(
                    ui,
                    l.triggerbot_scope_check,
                    Some(l.tip_triggerbot_scope),
                    |ui| switch(ui, &mut v).changed(),
                ) {
                    self.weapon_config().triggerbot.scope_check = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.triggerbot_velocity_check) {
                let mut v = self.weapon_config().triggerbot.velocity_check;
                if setting_row(
                    ui,
                    l.triggerbot_velocity_check,
                    Some(l.tip_triggerbot_velocity),
                    |ui| switch(ui, &mut v).changed(),
                ) {
                    self.weapon_config().triggerbot.velocity_check = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.triggerbot_velocity_threshold)
                && setting_row(
                    ui,
                    l.triggerbot_velocity_threshold,
                    Some(l.tip_triggerbot_velocity),
                    |ui| {
                        slider(
                            ui,
                            &mut self.weapon_config().triggerbot.velocity_threshold,
                            0.0..=5000.0,
                            "",
                            0,
                        )
                    },
                )
            {
                self.send_config();
            }
        });

        titled_card(ui, l.section_rcs, |ui| {
            if is_weapon && matches_filter(filter, l.enable_override) {
                let mut v = self.weapon_config().rcs.enable_override;
                if setting_row(ui, l.enable_override, None, |ui| switch(ui, &mut v).changed()) {
                    self.weapon_config().rcs.enable_override = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.enable) {
                let mut v = self.weapon_config().rcs.enabled;
                if setting_row(ui, l.enable, Some(l.tip_rcs_enable), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.weapon_config().rcs.enabled = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.rcs_mode) {
                sub_label(ui, l.rcs_mode);
                if segmented(ui, &mut self.weapon_config().rcs.mode, "rcs_mode") {
                    self.send_config();
                }
            }
            if matches_filter(filter, l.rcs_strength)
                && setting_row(ui, l.rcs_strength, Some(l.tip_rcs_strength), |ui| {
                    let rcs = &mut self.weapon_config().rcs;
                    let cx = slider(ui, &mut rcs.strength.x, 0.0..=1.0, " X", 2);
                    let cy = slider(ui, &mut rcs.strength.y, 0.0..=1.0, " Y", 2);
                    cx || cy
                })
            {
                self.send_config();
            }
            if matches_filter(filter, l.rcs_smoothing)
                && setting_row(ui, l.rcs_smoothing, Some(l.tip_rcs_smoothing), |ui| {
                    slider(
                        ui,
                        &mut self.weapon_config().rcs.smoothing,
                        0.0..=0.95,
                        "",
                        2,
                    )
                })
            {
                self.send_config();
            }
        });

        let _ = components::row_gap;
    }
}
