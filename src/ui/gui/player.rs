use egui::Ui;

use crate::ui::{
    app::App,
    gui::{
        components::{matches_filter, segmented, setting_row, slider, sub_label, switch, titled_card},
        helpers::{color_picker, keybind, scroll},
    },
};

impl App {
    /// Combined Visuals tab: Player ESP on top, HUD settings below.
    pub fn visuals_settings(&mut self, ui: &mut Ui) {
        let filter = self.search_query.clone();
        scroll(ui, "visuals", |ui| {
            ui.columns(2, |cols| {
                self.player_left(&mut cols[0], &filter);
                self.player_right(&mut cols[1], &filter);
            });
            ui.add_space(6.0);
            ui.columns(2, |cols| {
                self.hud_left(&mut cols[0], &filter);
                self.hud_right(&mut cols[1], &filter);
            });
        });
    }

    fn player_left(&mut self, ui: &mut Ui, filter: &str) {
        let l = self.lang();

        titled_card(ui, l.section_players, |ui| {
            if matches_filter(filter, l.enable) {
                let mut v = self.config.player.enabled;
                if setting_row(ui, l.enable, Some(l.tip_player_enable), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.player.enabled = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.player_esp_hotkey)
                && keybind(ui, "esp_hotkey", l.player_esp_hotkey, &mut self.config.player.esp_hotkey)
            {
                self.send_config();
            }
            if matches_filter(filter, l.player_show_friendlies) {
                let mut v = self.config.player.show_friendlies;
                if setting_row(
                    ui,
                    l.player_show_friendlies,
                    Some(l.tip_player_show_friendlies),
                    |ui| switch(ui, &mut v).changed(),
                ) {
                    self.config.player.show_friendlies = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.player_box) {
                sub_label(ui, l.player_box);
                if segmented(ui, &mut self.config.player.draw_box, "draw_box") {
                    self.send_config();
                }
            }
            if matches_filter(filter, l.player_box_mode) {
                sub_label(ui, l.player_box_mode);
                if segmented(ui, &mut self.config.player.box_mode, "box_mode") {
                    self.send_config();
                }
            }
            if matches_filter(filter, l.player_skeleton) {
                sub_label(ui, l.player_skeleton);
                if segmented(ui, &mut self.config.player.draw_skeleton, "draw_skeleton") {
                    self.send_config();
                }
            }
            if matches_filter(filter, l.player_head_circle) {
                let mut v = self.config.player.head_circle;
                if setting_row(ui, l.player_head_circle, None, |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.player.head_circle = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.player_visible_only) {
                let mut v = self.config.player.visible_only;
                if setting_row(ui, l.player_visible_only, Some(l.tip_player_visible_only), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.player.visible_only = v;
                    self.send_config();
                }
            }
        });

        titled_card(ui, l.section_colors, |ui| {
            if matches_filter(filter, l.color_box_visible)
                && color_picker(ui, l.color_box_visible, &mut self.config.player.box_visible_color)
            {
                self.send_config();
            }
            if matches_filter(filter, l.color_box_invisible)
                && color_picker(
                    ui,
                    l.color_box_invisible,
                    &mut self.config.player.box_invisible_color,
                )
            {
                self.send_config();
            }
            if matches_filter(filter, l.color_skeleton)
                && color_picker(ui, l.color_skeleton, &mut self.config.player.skeleton_color)
            {
                self.send_config();
            }
            if matches_filter(filter, l.color_player_name)
                && color_picker(ui, l.color_player_name, &mut self.config.player.name_color)
            {
                self.send_config();
            }
            if matches_filter(filter, l.color_weapon_icon)
                && color_picker(ui, l.color_weapon_icon, &mut self.config.player.weapon_icon_color)
            {
                self.send_config();
            }
            if matches_filter(filter, l.color_tags)
                && color_picker(ui, l.color_tags, &mut self.config.player.tag_color)
            {
                self.send_config();
            }
        });
    }

    fn player_right(&mut self, ui: &mut Ui, filter: &str) {
        let l = self.lang();

        titled_card(ui, "Info", |ui| {
            if matches_filter(filter, l.player_health_bar) {
                let mut v = self.config.player.health_bar;
                if setting_row(ui, l.player_health_bar, Some(l.tip_player_health_bar), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.player.health_bar = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.player_armor_bar) {
                let mut v = self.config.player.armor_bar;
                if setting_row(ui, l.player_armor_bar, Some(l.tip_player_armor_bar), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.player.armor_bar = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.player_name) {
                let mut v = self.config.player.player_name;
                if setting_row(ui, l.player_name, None, |ui| switch(ui, &mut v).changed()) {
                    self.config.player.player_name = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.player_weapon_icon) {
                let mut v = self.config.player.weapon_icon;
                if setting_row(ui, l.player_weapon_icon, None, |ui| switch(ui, &mut v).changed()) {
                    self.config.player.weapon_icon = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.player_tags) {
                let mut v = self.config.player.tags;
                if setting_row(ui, l.player_tags, Some(l.tip_player_tags), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.player.tags = v;
                    self.send_config();
                }
            }
        });

        titled_card(ui, l.section_sound_esp, |ui| {
            if matches_filter(filter, l.enable) {
                let mut v = self.config.player.sound.enabled;
                if setting_row(ui, l.enable, Some(l.tip_sound_esp), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.player.sound.enabled = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.sound_fade_duration)
                && setting_row(ui, l.sound_fade_duration, Some(l.tip_sound_esp), |ui| {
                    slider(
                        ui,
                        &mut self.config.player.sound.fadeout_duration,
                        0.0..=10.0,
                        " s",
                        2,
                    )
                })
            {
                self.send_config();
            }
            if matches_filter(filter, l.sound_show_visible) {
                let mut v = self.config.player.sound.show_visible;
                if setting_row(ui, l.sound_show_visible, None, |ui| switch(ui, &mut v).changed()) {
                    self.config.player.sound.show_visible = v;
                    self.send_config();
                }
            }
            self.sound_diameter_row(ui, filter, "Footstep");
            self.sound_diameter_row(ui, filter, "Gunshot");
            self.sound_diameter_row(ui, filter, "Weapon");
        });
    }

    fn sound_diameter_row(&mut self, ui: &mut Ui, filter: &str, kind: &str) {
        if !matches_filter(filter, kind) {
            return;
        }
        let (value, default, max) = match kind {
            "Footstep" => (
                &mut self.config.player.sound.footstep_diameter,
                crate::constants::cs2::SOUND_ESP_FOOTSTEP_DIAMETER_DEFAULT,
                6000.0,
            ),
            "Gunshot" => (
                &mut self.config.player.sound.gunshot_diameter,
                crate::constants::cs2::SOUND_ESP_GUNSHOT_DIAMETER_DEFAULT,
                10000.0,
            ),
            _ => (
                &mut self.config.player.sound.weapon_diameter,
                crate::constants::cs2::SOUND_ESP_WEAPON_DIAMETER_DEFAULT,
                6000.0,
            ),
        };
        let mut changed = false;
        if setting_row(ui, kind, None, |ui| {
            let c = slider(ui, value, 200.0..=max, "", 0);
            if ui.button("↺").on_hover_text("Reset").clicked() {
                *value = default;
                return true;
            }
            c
        }) {
            changed = true;
        }
        if changed {
            self.send_config();
        }
    }
}
