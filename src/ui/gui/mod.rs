use egui::{Align, Ui};

use crate::{
    config::{WeaponConfig, write_config},
    message::{GameMessage, GameStatus},
    ui::{app::App, color::Colors, gui::aimbot::AimbotTab},
};

mod about;
pub mod aimbot;
mod application;
pub mod body_picker;
pub mod components;
mod config;
mod grenade;
mod helpers;
mod hud;
mod player;
mod r#unsafe;

#[derive(PartialEq)]
pub enum Tab {
    Aimbot,
    Visuals,
    Grenades,
    Unsafe,
    Config,
    Application,
}

impl App {
    pub fn send_config(&self) {
        self.send_message(GameMessage(Box::new(self.config.clone())));
        self.save();
    }

    pub fn send_message(&self, message: GameMessage) {
        if self.channel.send(message).is_err() {
            std::process::exit(1);
        }
    }

    fn save(&self) {
        write_config(&self.config, &self.current_config);
    }

    fn gui(&mut self, ui: &mut Ui) {
        ui.ctx().set_pixels_per_point(self.display_scale);

        // Snapshot localized strings so we don't re-borrow `self` while
        // building widgets that already take `&mut self.*`.
        let lang = self.lang();
        let tab_aimbot = lang.tab_aimbot;
        let tab_player = lang.tab_player;
        let tab_grenades = lang.tab_grenades;
        let tab_unsafe = lang.tab_unsafe;
        let tab_config = lang.tab_config;
        let tab_application = lang.tab_application;
        let lbl_status = match self.game_status {
            GameStatus::Working => lang.status_connected,
            GameStatus::NotStarted => lang.status_waiting,
        };
        let lbl_about = lang.btn_about;
        let lbl_issue = lang.btn_report_issue;
        let search_hint = lang.search_placeholder;

        // ── Top navbar: brand, tabs, search, status, about/issue ─────────
        let dot_color = match self.game_status {
            GameStatus::Working => Colors::GREEN,
            GameStatus::NotStarted => Colors::YELLOW,
        };

        // Pre-compute "feature active" flags for tab indicator dots
        let aim_active = self.config.aim.global.aimbot.enabled
            || self.config.aim.global.triggerbot.enabled
            || self.config.aim.global.rcs.enabled;
        let visuals_active = self.config.player.enabled;
        let hud_active = self.config.hud.grenade_trails
            || self.config.hud.bomb_timer
            || self.config.hud.fov_circle;
        let unsafe_active = self.config.misc.no_flash
            || self.config.misc.fov_changer
            || self.config.misc.no_smoke
            || self.config.misc.radar_hack;

        egui::Panel::top("navbar")
            .resizable(false)
            .exact_size(80.0)
            .frame(
                egui::Frame::new()
                    .fill(Colors::BACKDROP)
                    .inner_margin(egui::Margin::symmetric(14, 4))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 14),
                    )),
            )
            .show_inside(ui, |ui| {
                // Row 1: right-aligned controls — fixed 30 px tall so the tab
                // row below is not squeezed out.
                let row_w = ui.available_width();
                ui.allocate_ui_with_layout(
                    egui::vec2(row_w, 30.0),
                    egui::Layout::right_to_left(Align::Center),
                    |ui| {
                        if ui
                            .add_sized(
                                [30.0, 30.0],
                                egui::Button::new(egui::RichText::new("?").size(13.0)),
                            )
                            .on_hover_text(lbl_about)
                            .clicked()
                        {
                            self.show_about = true;
                        }
                        ui.add_space(2.0);
                        if ui
                            .add_sized(
                                [30.0, 30.0],
                                egui::Button::new(
                                    egui::RichText::new("\u{2691}").size(13.0),
                                ),
                            )
                            .on_hover_text(lbl_issue)
                            .clicked()
                        {
                            let _ = std::process::Command::new("xdg-open")
                                .arg("https://github.com/avitran0/deadlocked/issues")
                                .status();
                        }
                        ui.add_space(6.0);
                        components::status_pill(ui, dot_color, lbl_status);
                        ui.add_space(6.0);
                        components::search_box(ui, &mut self.search_query, search_hint);
                    },
                );
                // Row 2: tabs — full width, never clips
                ui.horizontal(|ui| {
                    use components::nav_tab;
                    if nav_tab(ui, tab_aimbot, self.current_tab == Tab::Aimbot, aim_active)
                    {
                        self.current_tab = Tab::Aimbot;
                    }
                    if nav_tab(
                        ui,
                        tab_player,
                        self.current_tab == Tab::Visuals,
                        visuals_active,
                    ) {
                        self.current_tab = Tab::Visuals;
                    }
                    if nav_tab(
                        ui,
                        tab_grenades,
                        self.current_tab == Tab::Grenades,
                        hud_active,
                    ) {
                        self.current_tab = Tab::Grenades;
                    }
                    if nav_tab(
                        ui,
                        tab_unsafe,
                        self.current_tab == Tab::Unsafe,
                        unsafe_active,
                    ) {
                        self.current_tab = Tab::Unsafe;
                    }
                    if nav_tab(ui, tab_config, self.current_tab == Tab::Config, false) {
                        self.current_tab = Tab::Config;
                    }
                    if nav_tab(
                        ui,
                        tab_application,
                        self.current_tab == Tab::Application,
                        false,
                    ) {
                        self.current_tab = Tab::Application;
                    }
                });
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Colors::BASE)
                    .inner_margin(egui::Margin::same(16)),
            )
            .show_inside(ui, |ui| {
                helpers::scroll(ui, "tab_scroll", |ui| match self.current_tab {
                    Tab::Aimbot => self.aimbot_settings(ui),
                    Tab::Visuals => self.visuals_settings(ui),
                    Tab::Grenades => self.grenade_settings(ui),
                    Tab::Unsafe => self.unsafe_settings(ui),
                    Tab::Config => self.config_settings(ui),
                    Tab::Application => self.application_settings(ui),
                });
            });

        if self.show_about {
            self.about(ui.ctx());
        }

        if self.app_config.first_launch {
            self.stacktrace_popup(ui.ctx());
        }
    }

    fn weapon_config(&mut self) -> &mut WeaponConfig {
        if self.aimbot_tab == AimbotTab::Weapon {
            self.config
                .aim
                .weapons
                .get_mut(&self.aimbot_weapon)
                .unwrap()
        } else {
            &mut self.config.aim.global
        }
    }

    pub fn render(&mut self) {
        let self_ptr = self as *mut Self;

        let gui = self.gui.as_mut().unwrap();

        if let Err(err) = gui.make_current() {
            utils::error!("could not make gui window current: {err}");
            return;
        }
        gui.run(|ui| (unsafe { &mut *self_ptr }).gui(ui));
        gui.clear();
        gui.paint();

        if let Err(err) = gui.swap_buffers() {
            utils::error!("could not swap gui window buffers: {err}");
            return;
        }

        let overlay = self.overlay.as_mut().unwrap();

        overlay.window().set_cursor_hittest(false).unwrap();
        if let Err(err) = overlay.make_current() {
            utils::error!("could not make overlay window current: {err}");
            return;
        }

        overlay.run(move |ui| {
            (unsafe { &mut *self_ptr }).overlay(ui);
        });
        overlay.clear();
        overlay.paint();

        if let Err(err) = overlay.swap_buffers() {
            utils::error!("could not swap overlay window buffers: {err}");
        }
    }
}
