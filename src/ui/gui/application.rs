use std::sync::atomic::Ordering;

use egui::Ui;
use strum::IntoEnumIterator as _;

use crate::{
    config::{Language, write_app_config},
    os::crash::STACKTRACE_SENT,
    ui::{
        app::App,
        gui::components::{matches_filter, setting_row, switch, titled_card},
    },
};

impl App {
    pub fn application_settings(&mut self, ui: &mut Ui) {
        let l = self.lang();
        let filter = self.search_query.clone();

        titled_card(ui, l.section_application, |ui| {
            if matches_filter(&filter, l.app_language) {
                setting_row(ui, l.app_language, Some(l.tip_app_language), |ui| {
                    let mut changed = false;
                    for lang in Language::iter() {
                        let label = match lang {
                            Language::English => "English",
                            Language::Turkish => "Türkçe",
                        };
                        let selected = self.app_config.language == lang;
                        if ui.add(egui::Button::new(label).selected(selected)).clicked() {
                            self.app_config.language = lang;
                            write_app_config(&self.app_config);
                            changed = true;
                        }
                    }
                    changed
                });
            }

            if matches_filter(&filter, l.app_send_stacktraces) {
                let mut v = self.app_config.send_stacktraces;
                if setting_row(
                    ui,
                    l.app_send_stacktraces,
                    Some(l.tip_send_stacktraces),
                    |ui| switch(ui, &mut v).changed(),
                ) {
                    self.app_config.send_stacktraces = v;
                    write_app_config(&self.app_config);
                    STACKTRACE_SENT.store(!self.app_config.send_stacktraces, Ordering::Relaxed);
                }
            }
        });
    }

    pub fn stacktrace_popup(&mut self, ctx: &egui::Context) {
        let l = self.lang();
        egui::Window::new("Send Crash Reports?")
            .resizable([false, false])
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("This helps me fix bugs, and only includes application stack traces.");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button(l.app_stacktraces_yes).clicked() {
                        self.app_config.first_launch = false;
                        self.app_config.send_stacktraces = true;
                        write_app_config(&self.app_config);
                    }

                    if ui.button(l.app_stacktraces_no).clicked() {
                        self.app_config.first_launch = false;
                        self.app_config.send_stacktraces = false;
                        write_app_config(&self.app_config);
                        STACKTRACE_SENT.store(true, Ordering::Relaxed);
                    }
                });
            });
    }
}
