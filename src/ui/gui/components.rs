//! Material-3 vibe component primitives used by the redesigned UI.
//!
//! These wrap egui to produce a more cohesive visual language: cards,
//! pills/chips, segmented controls (replacing combo-boxes), and labelled
//! setting rows with hover help.

#![allow(dead_code)]

use egui::{
    Align, Color32, CornerRadius, Frame, Layout, Margin, Response, RichText, Sense, Stroke,
    StrokeKind, Ui, vec2,
};

use crate::ui::color::Colors;

/// Material-3 style elevated card container. Inner content gets a small
/// inset margin. Returns the inner UI response so the caller can chain.
pub fn card<R>(ui: &mut Ui, body: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(Colors::SURFACE)
        .corner_radius(CornerRadius::same(12))
        .stroke(Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 14),
        ))
        .inner_margin(Margin::symmetric(14, 12))
        .show(ui, body)
        .inner
}

/// Card with a title row (Material-3 "section header" feel).
pub fn titled_card<R>(
    ui: &mut Ui,
    title: &str,
    body: impl FnOnce(&mut Ui) -> R,
) -> R {
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(title).size(15.0).strong().color(Colors::TEXT));
        });
        ui.add_space(6.0);
        ui.scope(|ui| {
            // Slight subtle divider
            let painter = ui.painter();
            let r = ui.max_rect();
            let y = ui.cursor().top() - 3.0;
            painter.hline(
                r.left()..=r.right(),
                y,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 12)),
            );
        });
        ui.add_space(4.0);
        body(ui)
    })
}

/// Small pill/chip — used for status indicators and "active features" summary.
pub fn chip(ui: &mut Ui, label: &str, color: Color32, filled: bool) -> Response {
    let (bg, fg, stroke) = if filled {
        (
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 220),
            Colors::TEXT,
            Stroke::NONE,
        )
    } else {
        (
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 38),
            color,
            Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 90),
            ),
        )
    };

    let text = RichText::new(label).size(12.0).color(fg);
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(12.0),
        fg,
    );
    let pad = vec2(10.0, 4.0);
    let size = galley.size() + pad * 2.0;
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    painter.rect(rect, CornerRadius::same(10), bg, stroke, StrokeKind::Inside);
    let _ = text;
    painter.galley(rect.left_top() + pad, galley, fg);
    response
}

/// Material-3 "segmented button" — horizontal exclusive selector, replacing
/// combo-boxes. Generic over any enum implementing `IntoEnumIterator + PartialEq + Debug`.
pub fn segmented<T>(ui: &mut Ui, value: &mut T, id: &str) -> bool
where
    T: strum::IntoEnumIterator + PartialEq + std::fmt::Debug + Clone,
{
    let variants: Vec<T> = T::iter().collect();
    let n = variants.len().max(1);
    let total_w = ui.available_width().min(360.0);
    let h = 30.0;
    let inner_w = (total_w / n as f32).floor();

    let (rect, _) =
        ui.allocate_exact_size(vec2(inner_w * n as f32, h), Sense::hover());
    let painter = ui.painter();
    // Outer pill
    painter.rect(
        rect,
        CornerRadius::same(10),
        Colors::BACKDROP,
        Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 18),
        ),
        StrokeKind::Inside,
    );

    let mut changed = false;
    let accent = ui.visuals().hyperlink_color;

    for (i, v) in variants.iter().enumerate() {
        let cell = egui::Rect::from_min_size(
            rect.left_top() + vec2(inner_w * i as f32, 0.0),
            vec2(inner_w, h),
        );
        let selected = v == value;
        let id_ = ui.make_persistent_id((id, i));
        let resp = ui.interact(cell, id_, Sense::click());

        if selected {
            painter.rect_filled(
                cell.shrink(2.0),
                CornerRadius::same(8),
                Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 220),
            );
        } else if resp.hovered() {
            painter.rect_filled(
                cell.shrink(2.0),
                CornerRadius::same(8),
                Color32::from_rgba_unmultiplied(255, 255, 255, 12),
            );
        }

        let text = format!("{:?}", v);
        let color = if selected { Colors::on(accent) } else { Colors::SUBTEXT };
        painter.text(
            cell.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(13.0),
            color,
        );

        if resp.clicked() && !selected {
            *value = v.clone();
            changed = true;
        }
    }
    changed
}

/// Material-3 "switch"-ish toggle — wider tap target, accent fill when on.
pub fn switch(ui: &mut Ui, value: &mut bool) -> Response {
    let size = vec2(38.0, 22.0);
    let (rect, mut resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.clicked() {
        *value = !*value;
        resp.mark_changed();
    }
    let accent = ui.visuals().hyperlink_color;
    let painter = ui.painter();
    let bg = if *value {
        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 220)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 30)
    };
    painter.rect(
        rect,
        CornerRadius::same(11),
        bg,
        Stroke::NONE,
        StrokeKind::Inside,
    );
    let knob_x = if *value {
        rect.right() - 11.0
    } else {
        rect.left() + 11.0
    };
    painter.circle_filled(egui::pos2(knob_x, rect.center().y), 8.5, Colors::TEXT);
    resp
}

/// Material-3 style progress-bar slider. Click/drag on the track to set
/// value. The filled portion uses the accent color and a small knob marks
/// the current position; the value is overlaid as text.
///
/// Generic over any numeric type via `egui::emath::Numeric`.
pub fn slider<Num: egui::emath::Numeric>(
    ui: &mut Ui,
    value: &mut Num,
    range: std::ops::RangeInclusive<f64>,
    suffix: &str,
    decimals: usize,
) -> bool {
    let desired_w = ui.available_width().min(200.0).max(140.0);
    let h = 22.0;
    let (rect, resp) = ui.allocate_exact_size(vec2(desired_w, h), Sense::click_and_drag());

    let (min, max) = (*range.start(), *range.end());
    let mut v_f = value.to_f64();
    let mut changed = false;

    // Handle pointer interaction.
    if (resp.dragged() || resp.clicked()) && let Some(p) = resp.interact_pointer_pos() {
        let t = ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0) as f64;
        let new_v = min + (max - min) * t;
        if (new_v - v_f).abs() > f64::EPSILON {
            v_f = new_v;
            *value = Num::from_f64(new_v);
            changed = true;
        }
    }

    // Map current value -> filled fraction.
    let frac = if max > min {
        ((v_f - min) / (max - min)).clamp(0.0, 1.0) as f32
    } else {
        0.0
    };

    let painter = ui.painter();
    let accent = ui.visuals().hyperlink_color;

    // Track background.
    painter.rect(
        rect,
        CornerRadius::same(11),
        Colors::BACKDROP,
        Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(255, 255, 255, 18),
        ),
        StrokeKind::Inside,
    );

    // Filled portion.
    let fill_w = rect.width() * frac;
    if fill_w > 0.5 {
        let fill_rect = egui::Rect::from_min_size(rect.min, vec2(fill_w, rect.height()));
        painter.rect(
            fill_rect.shrink(1.5),
            CornerRadius::same(9),
            Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 200),
            Stroke::NONE,
            StrokeKind::Inside,
        );
    }

    // Knob (small dot at the right edge of the filled portion).
    let knob_x = rect.left() + fill_w;
    let knob_x = knob_x.clamp(rect.left() + 4.0, rect.right() - 4.0);
    painter.circle_filled(egui::pos2(knob_x, rect.center().y), 4.5, Colors::TEXT);

    // Value text centered.
    let text = if suffix.is_empty() {
        format!("{:.*}", decimals, v_f)
    } else {
        format!("{:.*}{}", decimals, v_f, suffix)
    };
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(12.0),
        Colors::TEXT,
    );

    changed
}

/// One-row setting: left label (with optional help tooltip), right control.
/// Body returns whether the control was changed.
pub fn setting_row(
    ui: &mut Ui,
    label: &str,
    help: Option<&str>,
    control: impl FnOnce(&mut Ui) -> bool,
) -> bool {
    ui.horizontal(|ui| {
        let label_w = 150.0;
        ui.allocate_ui_with_layout(
            vec2(label_w, ui.spacing().interact_size.y),
            Layout::left_to_right(Align::Center),
            |ui| {
                let resp = ui.label(RichText::new(label).color(Colors::TEXT).size(13.0));
                if let Some(h) = help {
                    resp.on_hover_text(h);
                }
            },
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| control(ui))
            .inner
    })
    .inner
}

/// Compact text-input search box with placeholder and clear button.
pub fn search_box(ui: &mut Ui, value: &mut String, placeholder: &str) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let frame = Frame::new()
            .fill(Colors::SURFACE)
            .corner_radius(CornerRadius::same(10))
            .stroke(Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 18),
            ))
            .inner_margin(Margin::symmetric(10, 6));
        frame.show(ui, |ui| {
            ui.set_min_width(90.0);
            ui.label(RichText::new("\u{1F50D}").size(12.0).color(Colors::SUBTEXT));
            let edit = egui::TextEdit::singleline(value)
                .hint_text(placeholder)
                .frame(egui::Frame::NONE)
                .desired_width(72.0);
            if ui.add(edit).changed() {
                changed = true;
            }
            if !value.is_empty()
                && ui
                    .button(RichText::new("\u{2715}").size(11.0))
                    .clicked()
            {
                value.clear();
                changed = true;
            }
        });
    });
    changed
}

/// Status dot + label pill (e.g. "Connected" / "Waiting").
pub fn status_pill(ui: &mut Ui, color: Color32, text: &str) -> Response {
    let pad = vec2(10.0, 5.0);
    let font = egui::FontId::proportional(12.0);
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, Colors::TEXT);
    let size = vec2(galley.size().x + 22.0 + pad.x, galley.size().y + pad.y * 2.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    painter.rect(
        rect,
        CornerRadius::same(12),
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 28),
        Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 90),
        ),
        StrokeKind::Inside,
    );
    let dot_x = rect.left() + pad.x + 4.0;
    painter.circle_filled(egui::pos2(dot_x, rect.center().y), 4.0, color);
    painter.galley(
        egui::pos2(dot_x + 10.0, rect.center().y - galley.size().y / 2.0),
        galley,
        Colors::TEXT,
    );
    resp
}

/// Big-prominent tab button for the top navbar with an optional "active feature"
/// dot. Returns clicked.
pub fn nav_tab(
    ui: &mut Ui,
    label: &str,
    selected: bool,
    active_indicator: bool,
) -> bool {
    let pad = vec2(10.0, 6.0);
    let font = egui::FontId::proportional(13.0);
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        font,
        if selected { Colors::TEXT } else { Colors::SUBTEXT },
    );
    let size = vec2(
        galley.size().x + pad.x * 2.0 + if active_indicator { 12.0 } else { 0.0 },
        32.0,
    );
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    let accent = ui.visuals().hyperlink_color;

    if selected {
        painter.rect_filled(
            rect,
            CornerRadius::same(10),
            Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 36),
        );
        // bottom underline
        let underline = egui::Rect::from_min_size(
            egui::pos2(rect.left() + 10.0, rect.bottom() - 3.0),
            vec2(rect.width() - 20.0, 2.5),
        );
        painter.rect_filled(underline, CornerRadius::same(2), accent);
    } else if resp.hovered() {
        painter.rect_filled(
            rect,
            CornerRadius::same(10),
            Color32::from_rgba_unmultiplied(255, 255, 255, 14),
        );
    }

    let mut text_x = rect.left() + pad.x;
    if active_indicator {
        painter.circle_filled(
            egui::pos2(text_x + 4.0, rect.center().y),
            3.5,
            Colors::GREEN,
        );
        text_x += 12.0;
    }
    painter.galley(
        egui::pos2(text_x, rect.center().y - galley.size().y / 2.0),
        galley,
        if selected { Colors::TEXT } else { Colors::SUBTEXT },
    );
    resp.clicked()
}

/// Floating Action Button — large pill primary action.
pub fn fab(ui: &mut Ui, label: &str) -> Response {
    let pad = vec2(20.0, 10.0);
    let font = egui::FontId::proportional(14.0);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, Colors::TEXT);
    let size = galley.size() + pad * 2.0;
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    let accent = ui.visuals().hyperlink_color;
    let bg = if resp.hovered() {
        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 255)
    } else {
        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 220)
    };
    painter.rect(
        rect,
        CornerRadius::same(rect.height() as u8 / 2),
        bg,
        Stroke::NONE,
        StrokeKind::Inside,
    );
    painter.galley(rect.left_top() + pad, galley, Colors::TEXT);
    resp
}

/// Tiny section label (above a group of settings inside a card).
pub fn sub_label(ui: &mut Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(
        RichText::new(text.to_uppercase())
            .size(10.5)
            .color(Colors::SUBTEXT)
            .strong(),
    );
    ui.add_space(2.0);
}

/// Returns whether the given text matches the search filter (case-insensitive
/// substring). An empty filter matches everything.
pub fn matches_filter(filter: &str, text: &str) -> bool {
    if filter.is_empty() {
        return true;
    }
    text.to_lowercase().contains(&filter.to_lowercase())
}

/// Vertical thin spacer used between rows inside a card.
pub fn row_gap(ui: &mut Ui) {
    ui.add_space(4.0);
}

/// Mute the unused-import warning on Vec2 in some builds.
#[allow(dead_code)]
fn _force_use() {}
