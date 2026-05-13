use egui::Ui;

use crate::ui::{
    app::App,
    gui::{
        components::{matches_filter, setting_row, slider, switch, titled_card},
        helpers::color_picker,
    },
};

impl App {
    pub fn hud_left(&mut self, ui: &mut Ui, filter: &str) {
        let l = self.lang();

        titled_card(ui, l.section_hud, |ui| {
            if matches_filter(filter, l.hud_bomb_timer) {
                let mut v = self.config.hud.bomb_timer;
                if setting_row(ui, l.hud_bomb_timer, Some(l.tip_hud_bomb_timer), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.hud.bomb_timer = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.hud_fov_circle) {
                let mut v = self.config.hud.fov_circle;
                if setting_row(ui, l.hud_fov_circle, Some(l.tip_hud_fov_circle), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.hud.fov_circle = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.hud_dropped_weapons) {
                let mut v = self.config.hud.dropped_weapons;
                if setting_row(ui, l.hud_dropped_weapons, None, |ui| switch(ui, &mut v).changed()) {
                    self.config.hud.dropped_weapons = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.hud_keybind_list) {
                let mut v = self.config.hud.keybind_list;
                if setting_row(ui, l.hud_keybind_list, None, |ui| switch(ui, &mut v).changed()) {
                    self.config.hud.keybind_list = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.hud_spectator_list) {
                let mut v = self.config.hud.spectator_list;
                if setting_row(
                    ui,
                    l.hud_spectator_list,
                    Some(l.tip_hud_spectator_list),
                    |ui| switch(ui, &mut v).changed(),
                ) {
                    self.config.hud.spectator_list = v;
                    self.send_config();
                }
            }
        });

        titled_card(ui, l.section_crosshair, |ui| {
            if matches_filter(filter, l.crosshair_enable) {
                let mut v = self.config.hud.sniper_crosshair.enabled;
                if setting_row(ui, l.crosshair_enable, Some(l.tip_crosshair_enable), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.hud.sniper_crosshair.enabled = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.crosshair_line_length)
                && setting_row(ui, l.crosshair_line_length, None, |ui| {
                    slider(
                        ui,
                        &mut self.config.hud.sniper_crosshair.line_length,
                        0.1..=500.0,
                        " px",
                        1,
                    )
                })
            {
                self.send_config();
            }
            if matches_filter(filter, l.crosshair_line_width)
                && setting_row(ui, l.crosshair_line_width, None, |ui| {
                    slider(
                        ui,
                        &mut self.config.hud.sniper_crosshair.line_width,
                        0.1..=10.0,
                        " px",
                        2,
                    )
                })
            {
                self.send_config();
            }
            if matches_filter(filter, l.crosshair_gap)
                && setting_row(ui, l.crosshair_gap, None, |ui| {
                    slider(
                        ui,
                        &mut self.config.hud.sniper_crosshair.gap,
                        0.0..=200.0,
                        " px",
                        1,
                    )
                })
            {
                self.send_config();
            }
        });
    }

    pub fn hud_right(&mut self, ui: &mut Ui, filter: &str) {
        let l = self.lang();

        titled_card(ui, l.section_text, |ui| {
            if matches_filter(filter, l.hud_text_outline) {
                let mut v = self.config.hud.text_outline;
                if setting_row(ui, l.hud_text_outline, None, |ui| switch(ui, &mut v).changed()) {
                    self.config.hud.text_outline = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.hud_line_width)
                && setting_row(ui, l.hud_line_width, None, |ui| {
                    slider(ui, &mut self.config.hud.line_width, 0.1..=8.0, " px", 1)
                })
            {
                self.send_config();
            }
            if matches_filter(filter, l.hud_font_size)
                && setting_row(ui, l.hud_font_size, None, |ui| {
                    slider(ui, &mut self.config.hud.font_size, 1.0..=99.0, "", 1)
                })
            {
                self.send_config();
            }
            if matches_filter(filter, l.hud_icon_size)
                && setting_row(ui, l.hud_icon_size, None, |ui| {
                    slider(ui, &mut self.config.hud.icon_size, 1.0..=99.0, "", 1)
                })
            {
                self.send_config();
            }
        });

        titled_card(ui, l.section_colors, |ui| {
            if matches_filter(filter, l.hud_text_color)
                && color_picker(ui, l.hud_text_color, &mut self.config.hud.text_color)
            {
                self.send_config();
            }
            if matches_filter(filter, l.crosshair_color)
                && color_picker(
                    ui,
                    l.crosshair_color,
                    &mut self.config.hud.sniper_crosshair.color,
                )
            {
                self.send_config();
            }
        });

        titled_card(ui, l.section_grenade_trails, |ui| {
            let mut v = self.config.hud.grenade_trails;
            if setting_row(ui, l.hud_dropped_weapons, None, |ui| switch(ui, &mut v).changed()) {
                self.config.hud.grenade_trails = v;
                self.send_config();
            }
            let mut changed = false;
            if matches_filter(filter, "Smoke")
                && color_picker(ui, "Smoke", &mut self.config.hud.smoke_trail_color)
            {
                changed = true;
            }
            if matches_filter(filter, "Molotov")
                && color_picker(ui, "Molotov", &mut self.config.hud.molotov_trail_color)
            {
                changed = true;
            }
            if matches_filter(filter, "Incendiary")
                && color_picker(ui, "Incendiary", &mut self.config.hud.incendiary_trail_color)
            {
                changed = true;
            }
            if matches_filter(filter, "Flash")
                && color_picker(ui, "Flash", &mut self.config.hud.flash_trail_color)
            {
                changed = true;
            }
            if matches_filter(filter, "HE")
                && color_picker(ui, "HE", &mut self.config.hud.he_trail_color)
            {
                changed = true;
            }
            if matches_filter(filter, "Decoy")
                && color_picker(ui, "Decoy", &mut self.config.hud.decoy_trail_color)
            {
                changed = true;
            }
            if changed {
                self.send_config();
            }
        });

        titled_card(ui, "Misc", |ui| {
            if matches_filter(filter, "FPS")
                && setting_row(ui, "FPS", Some("Overlay'in FPS limiti"), |ui| {
                    slider(ui, &mut self.config.fps, 30.0..=500.0, " fps", 0)
                })
            {
                self.send_config();
            }
            if matches_filter(filter, "Debug") {
                let mut v = self.config.hud.debug;
                if setting_row(ui, "Debug Overlay", None, |ui| switch(ui, &mut v).changed()) {
                    self.config.hud.debug = v;
                    self.send_config();
                }
            }
        });
    }
}
