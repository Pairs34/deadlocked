use egui::Ui;

use crate::ui::{
    app::App,
    gui::{
        components::{matches_filter, setting_row, slider, switch, titled_card},
        helpers::{color_picker, scroll},
    },
};

impl App {
    pub fn unsafe_settings(&mut self, ui: &mut Ui) {
        let l = self.lang();
        let filter = self.search_query.clone();
        scroll(ui, "unsafe", |ui| {
            ui.columns(2, |cols| {
                self.unsafe_left(&mut cols[0], &filter);
                self.unsafe_right(&mut cols[1], &filter);
            });
        });
        // Avoid lang move warning across closures.
        let _ = l;
    }

    fn unsafe_left(&mut self, ui: &mut Ui, filter: &str) {
        let l = self.lang();

        titled_card(ui, l.section_no_flash, |ui| {
            if matches_filter(filter, l.unsafe_no_flash) {
                let mut v = self.config.misc.no_flash;
                if setting_row(ui, l.unsafe_no_flash, Some(l.tip_no_flash), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.misc.no_flash = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.unsafe_max_flash)
                && setting_row(ui, l.unsafe_max_flash, Some(l.tip_unsafe_max_flash), |ui| {
                    slider(
                        ui,
                        &mut self.config.misc.max_flash_alpha,
                        0.0..=255.0,
                        "",
                        0,
                    )
                })
            {
                self.send_config();
            }
        });

        titled_card(ui, l.section_fov_changer, |ui| {
            if matches_filter(filter, l.unsafe_fov_changer) {
                let mut v = self.config.misc.fov_changer;
                if setting_row(ui, l.unsafe_fov_changer, Some(l.tip_fov_changer), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.misc.fov_changer = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.unsafe_desired_fov)
                && setting_row(ui, l.unsafe_desired_fov, None, |ui| {
                    let changed = slider(
                        ui,
                        &mut self.config.misc.desired_fov,
                        1.0..=179.0,
                        "°",
                        0,
                    );
                    if ui.button("↺").clicked() {
                        self.config.misc.desired_fov = crate::constants::cs2::DEFAULT_FOV;
                        return true;
                    }
                    changed
                })
            {
                self.send_config();
            }
        });
    }

    fn unsafe_right(&mut self, ui: &mut Ui, filter: &str) {
        let l = self.lang();

        titled_card(ui, l.section_smokes, |ui| {
            if matches_filter(filter, l.unsafe_no_smoke) {
                let mut v = self.config.misc.no_smoke;
                if setting_row(ui, l.unsafe_no_smoke, Some(l.tip_no_smoke), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.misc.no_smoke = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.unsafe_change_smoke_color) {
                let mut v = self.config.misc.change_smoke_color;
                if setting_row(
                    ui,
                    l.unsafe_change_smoke_color,
                    Some(l.tip_change_smoke_color),
                    |ui| switch(ui, &mut v).changed(),
                ) {
                    self.config.misc.change_smoke_color = v;
                    self.send_config();
                }
            }
            if matches_filter(filter, l.unsafe_smoke_color)
                && color_picker(ui, l.unsafe_smoke_color, &mut self.config.misc.smoke_color)
            {
                self.send_config();
            }
        });

        titled_card(ui, l.section_radar, |ui| {
            if matches_filter(filter, l.radar_enable) {
                let mut v = self.config.misc.radar_hack;
                if setting_row(ui, l.radar_enable, Some(l.tip_radar_enable), |ui| {
                    switch(ui, &mut v).changed()
                }) {
                    self.config.misc.radar_hack = v;
                    self.send_config();
                }
            }
        });
    }
}
