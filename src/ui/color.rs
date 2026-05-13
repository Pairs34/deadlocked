#![allow(unused)]
use egui::Color32;
use serde::{Deserialize, Serialize};

pub struct Colors;

impl Colors {
    // Modern dark palette — slightly cooler / bluer tones for a more contemporary look.
    pub const BACKDROP: Color32 = Color32::from_rgb(18, 19, 26);   // sidebar / panel backdrop
    pub const BASE: Color32 = Color32::from_rgb(26, 27, 36);       // main content area
    pub const SURFACE: Color32 = Color32::from_rgb(34, 36, 48);    // raised surfaces (cards/buttons)
    pub const HIGHLIGHT: Color32 = Color32::from_rgb(54, 57, 78);  // hover/highlight
    pub const SUBTEXT: Color32 = Color32::from_rgb(170, 175, 195);
    pub const TEXT: Color32 = Color32::from_rgb(238, 240, 248);
    pub const RED: Color32 = Color32::from_rgb(240, 100, 100);
    pub const ORANGE: Color32 = Color32::from_rgb(240, 140, 90);
    pub const YELLOW: Color32 = Color32::from_rgb(240, 200, 120);
    pub const GREEN: Color32 = Color32::from_rgb(140, 230, 130);
    pub const TEAL: Color32 = Color32::from_rgb(80, 200, 200);
    pub const BLUE: Color32 = Color32::from_rgb(110, 160, 250);
    pub const PURPLE: Color32 = Color32::from_rgb(180, 120, 240);

    pub const ACCENT_COLORS: [(&str, Color32); 7] = [
        ("Red", Self::RED),
        ("Orange", Self::ORANGE),
        ("Yellow", Self::YELLOW),
        ("Green", Self::GREEN),
        ("Teal", Self::TEAL),
        ("Blue", Self::BLUE),
        ("Purple", Self::PURPLE),
    ];

    /// Choose readable foreground (black or white) for a given background
    /// using ITU-R BT.601 luma. Threshold ~150 keeps mid-greens & yellows
    /// (which are bright) on a dark text, while keeping reds/blues white.
    pub fn on(bg: Color32) -> Color32 {
        let r = bg.r() as f32;
        let g = bg.g() as f32;
        let b = bg.b() as f32;
        let y = 0.299 * r + 0.587 * g + 0.114 * b;
        if y > 150.0 {
            Color32::from_rgb(20, 22, 30)
        } else {
            Self::TEXT
        }
    }
}
