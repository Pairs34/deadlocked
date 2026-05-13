// Several helpers below are retained as a convenience layer; some are unused
// after the UI redesign but kept for parity / future reuse.
#![allow(dead_code)]

use std::hash::Hash;

use egui::{CollapsingHeader, Color32, DragValue, Event, Sense, Ui, Widget};

use crate::cs2::key_codes::KeyCode;

pub fn collapsing_open(ui: &mut Ui, title: &str, add_body: impl FnOnce(&mut Ui)) {
    CollapsingHeader::new(title)
        .default_open(true)
        .show(ui, add_body);
}

/// Flat non-collapsing section header with accent left border.
pub fn section_header(ui: &mut Ui, label: &str) {
    ui.add_space(6.0);
    let height = ui.text_style_height(&egui::TextStyle::Body) + 4.0;
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(3.0, height), Sense::hover());
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(1),
            ui.visuals().hyperlink_color,
        );
        ui.add_space(5.0);
        ui.label(egui::RichText::new(label).strong());
    });
    ui.add_space(2.0);
}

pub fn scroll(ui: &mut Ui, id: &str, add_content: impl FnOnce(&mut Ui)) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .id_salt(id)
        .show(ui, add_content);
}

pub fn checkbox(ui: &mut Ui, label: &str, value: &mut bool) -> bool {
    ui.checkbox(value, label).changed()
}

pub fn checkbox_hover(ui: &mut Ui, label: &str, hover_text: &str, value: &mut bool) -> bool {
    ui.checkbox(value, label)
        .on_hover_text(hover_text)
        .changed()
}

pub fn drag(ui: &mut Ui, label: &str, drag: DragValue) -> bool {
    ui.horizontal(|ui| {
        let res = ui.add(drag);
        ui.label(label);
        res
    })
    .inner
    .changed()
}

pub fn drag_hover(ui: &mut Ui, label: &str, tooltip: &str, drag: DragValue) -> bool {
    ui.horizontal(|ui| {
        let res = ui.add(drag);
        ui.label(label).on_hover_text(tooltip);
        res
    })
    .inner
    .changed()
}

pub fn combo_box<T: std::fmt::Debug + strum::IntoEnumIterator + PartialEq>(
    ui: &mut Ui,
    id: &str,
    label: &str,
    value: &mut T,
) -> bool {
    let mut changed = false;
    egui::ComboBox::new(id, label)
        .selected_text(format!("{:?}", *value))
        .show_ui(ui, |ui| {
            for mode in T::iter() {
                let text = format!("{:?}", &mode);
                if ui.selectable_value(value, mode, text).clicked() {
                    changed = true;
                }
            }
        });
    changed
}

pub fn color_picker(ui: &mut Ui, label: &str, color: &mut Color32) -> bool {
    // Material-3 style palette picker: swatch row + native edit popup for
    // fine-grained adjustments. Returns true if the colour was changed.
    use crate::ui::color::Colors;
    const PRESETS: [Color32; 14] = [
        Colors::RED,
        Colors::ORANGE,
        Colors::YELLOW,
        Colors::GREEN,
        Colors::TEAL,
        Colors::BLUE,
        Colors::PURPLE,
        Color32::WHITE,
        Color32::from_rgb(255, 192, 203), // pink
        Color32::from_rgb(255, 105, 180), // hot pink
        Color32::from_rgb(0, 200, 255),   // cyan
        Color32::from_rgb(150, 75, 0),    // brown
        Color32::from_rgb(50, 50, 50),    // dark grey
        Color32::BLACK,
    ];

    let mut changed = false;
    ui.horizontal(|ui| {
        // Current colour preview (also opens the native edit popup)
        let resp = ui.color_edit_button_srgba(color);
        if resp.changed() {
            changed = true;
        }
        ui.add_space(4.0);

        // Preset swatches
        for preset in PRESETS {
            let size = egui::vec2(18.0, 18.0);
            let (rect, response) = ui.allocate_exact_size(size, Sense::click());
            let painter = ui.painter();
            let selected = (color.r(), color.g(), color.b()) == (preset.r(), preset.g(), preset.b());
            painter.rect_filled(
                rect,
                egui::CornerRadius::same(5),
                preset,
            );
            if selected {
                painter.rect_stroke(
                    rect.expand(1.5),
                    egui::CornerRadius::same(6),
                    egui::Stroke::new(1.5, Colors::TEXT),
                    egui::StrokeKind::Outside,
                );
            }
            if response.clicked() {
                let a = color.a();
                *color = Color32::from_rgba_unmultiplied(preset.r(), preset.g(), preset.b(), a);
                changed = true;
            }
            ui.add_space(2.0);
        }

        if !label.is_empty() {
            ui.add_space(6.0);
            ui.label(label);
        }
    });
    changed
}

pub fn keybind(ui: &mut Ui, id: &str, label: &str, keycode: &mut KeyCode) -> bool {
    ui.horizontal(|ui| {
        let res = ui.add(Keybind::new(keycode, id));
        ui.label(label);
        res
    })
    .inner
    .changed()
}

pub struct Keybind<'gui> {
    keycode: &'gui mut KeyCode,
    id: egui::Id,
}

impl<'gui> Keybind<'gui> {
    pub fn new(keycode: &'gui mut KeyCode, id: impl Hash) -> Self {
        Self {
            keycode,
            id: egui::Id::new(id),
        }
    }
}

impl<'gui> Widget for Keybind<'gui> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let listening_id = ui.make_persistent_id(self.id);

        let mut listening = {
            let ctx = ui.ctx();
            ctx.memory(|mem| mem.data.get_temp::<bool>(listening_id).unwrap_or(false))
        };

        let text = if listening {
            "...".to_string()
        } else {
            format!("{:?}", self.keycode)
        };

        let mut response = ui.button(text);

        if response.clicked() {
            listening = !listening;
        }

        if response.secondary_clicked() {
            listening = false;
        }

        if listening {
            let input = ui.input(|i| {
                for event in &i.events {
                    if let Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        dbg!(key);
                        if *key == egui::Key::F35 {
                            return KeyCode::from_egui_modifiers(*modifiers);
                        } else {
                            return KeyCode::from_egui(*key);
                        }
                    }

                    if let Event::PointerButton {
                        button,
                        pressed: true,
                        ..
                    } = event
                    {
                        return Some(KeyCode::from_egui_mouse(*button));
                    }
                }
                None
            });

            if let Some(input) = input {
                if input != KeyCode::Escape {
                    *self.keycode = input;
                    response.mark_changed();
                }
                listening = false;
            }
        }

        let ctx = ui.ctx();
        ctx.memory_mut(|mem| mem.data.insert_temp(listening_id, listening));

        response
    }
}
