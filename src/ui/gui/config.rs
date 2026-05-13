use egui::{Align, Button, Ui};

use crate::{
    config::{
        BASE_PATH, CONFIG_PATH, Config, available_configs, delete_config, parse_config,
        write_app_config, write_config,
    },
    ui::{
        app::App,
        color::Colors,
        grenades::read_grenades,
        gui::{
            components::{sub_label, titled_card},
            helpers::scroll,
        },
    },
};

impl App {
    pub fn config_settings(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            scroll(left, "config_left", |left| self.config_left(left));

            let right = &mut cols[1];
            scroll(right, "config_right", |right| self.config_right_pane(right));
        });
    }

    fn config_left(&mut self, ui: &mut Ui) {
        let l = self.lang();

        titled_card(ui, l.section_config, |ui| {
            if ui
                .button(l.config_reset)
                .on_hover_text(l.tip_config_reset)
                .clicked()
            {
                self.config = Config::legit();
                self.send_config();
                utils::info!("loaded legit default config");
            }

            if ui.button(l.config_folder).clicked() {
                if let Err(e) = std::process::Command::new("xdg-open")
                    .arg(BASE_PATH.as_os_str())
                    .status()
                {
                    utils::error!("xdg-open failed: {e}");
                }
            }
        });

        titled_card(ui, "Accent Color", |ui| {
            sub_label(ui, "Accent");
            ui.horizontal_wrapped(|ui| {
                for (name, color) in Colors::ACCENT_COLORS {
                    let selected = color == self.config.accent_color;
                    if ui
                        .add(Button::selectable(selected, name).fill(color))
                        .clicked()
                    {
                        self.config.accent_color = color;
                        ui.ctx()
                            .global_style_mut(|style| style.visuals.selection.bg_fill = color);
                        self.send_config();
                    }
                }
            });
        });
    }

    fn config_right_pane(&mut self, ui: &mut Ui) {
        let l = self.lang();

        titled_card(ui, l.section_config, |ui| {
            ui.horizontal(|ui| {
                if ui.button(l.config_refresh).clicked() {
                    self.available_configs = available_configs();
                    self.grenades = read_grenades();
                }
                if ui.button(l.config_save_as).clicked() && !self.new_config_name.is_empty() {
                    if !self.new_config_name.ends_with(".toml") {
                        self.new_config_name.push_str(".toml");
                    }
                    let path = CONFIG_PATH.join(&self.new_config_name);
                    write_config(&self.config, &path);
                    self.new_config_name.clear();
                    self.current_config = path;
                    self.available_configs = available_configs();
                }
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_config_name)
                        .hint_text(l.config_new_name),
                );
            });

            sub_label(ui, "Saved Configs");
            self.config_list(ui);
        });
    }

    fn config_list(&mut self, ui: &mut Ui) {
        let mut clicked_config = None;
        let mut delete = None;
        let mut set_default = None;

        for config in &self.available_configs {
            let file_name = config.file_name().unwrap().to_str().unwrap().to_string();
            let is_default = self.app_config.default_config.as_deref() == Some(file_name.as_str());

            ui.horizontal(|ui| {
                let label = if is_default {
                    format!("★ {file_name}")
                } else {
                    file_name.clone()
                };
                if ui
                    .add(Button::selectable(*config == self.current_config, label))
                    .clicked()
                {
                    clicked_config = Some(config.clone());
                }
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("\u{f0a7a}").on_hover_text("Sil").clicked() {
                        delete = Some(config.clone());
                    }
                    let star_tip = if is_default {
                        "Varsayılan (açılışta yüklenir) — kaldırmak için tıkla"
                    } else {
                        "Bunu varsayılan yap (açılışta yüklensin)"
                    };
                    let star_icon = if is_default { "★" } else { "☆" };
                    if ui.button(star_icon).on_hover_text(star_tip).clicked() {
                        set_default = Some((file_name.clone(), is_default));
                    }
                });
            });
        }

        if let Some(config_path) = clicked_config {
            self.config = parse_config(&config_path);
            self.current_config = config_path;
            self.send_config();
            ui.ctx().global_style_mut(|style| {
                style.visuals.selection.bg_fill = self.config.accent_color
            });
        }

        if let Some((name, was_default)) = set_default {
            self.app_config.default_config = if was_default { None } else { Some(name) };
            write_app_config(&self.app_config);
        }

        if let Some(config) = delete {
            if let Some(name) = config.file_name().and_then(|n| n.to_str())
                && self.app_config.default_config.as_deref() == Some(name)
            {
                self.app_config.default_config = None;
                write_app_config(&self.app_config);
            }
            delete_config(&config);
            self.available_configs = available_configs();
            self.current_config = self.available_configs[0].clone();
            self.config = parse_config(&self.current_config);
        }
    }
}
