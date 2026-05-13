use crate::ui::app::App;

const COMMIT: &str = env!("GIT_HASH");

impl App {
    pub fn about(&mut self, ctx: &egui::Context) {
        egui::Window::new("About")
            .resizable([false, false])
            .collapsible(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.colored_label(self.config.accent_color, "Pairs Cheat");
                    ui.label(format!("Commit: #{COMMIT}"));

                    if ui.button("Close").clicked() {
                        self.show_about = false;
                    }
                });
            });
    }
}
