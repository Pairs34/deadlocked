//! Body silhouette bone picker.
//!
//! Replaces the boring vertical bone list with a clickable humanoid
//! silhouette. Each anatomical region maps to a `Bones` enum variant;
//! clicking a region toggles it in the bones vector. Selected regions
//! are filled with the accent color, others with a muted base color.

#![allow(dead_code)]

use egui::{Color32, CornerRadius, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::{cs2::bones::Bones, ui::color::Colors};

/// A region of the silhouette. `points` are in 0..1 normalized space
/// (later mapped to the widget rect).
struct Region {
    bone: Bones,
    poly: &'static [(f32, f32)],
}

/// All regions of the silhouette in draw order (back to front).
/// Coordinates are 0..1 in (x, y) where (0,0) is top-left.
const REGIONS: &[Region] = &[
    // Head
    Region {
        bone: Bones::Head,
        poly: &[
            (0.42, 0.02),
            (0.58, 0.02),
            (0.62, 0.08),
            (0.60, 0.16),
            (0.40, 0.16),
            (0.38, 0.08),
        ],
    },
    // Neck
    Region {
        bone: Bones::Neck,
        poly: &[
            (0.44, 0.16),
            (0.56, 0.16),
            (0.57, 0.21),
            (0.43, 0.21),
        ],
    },
    // Upper chest (Spine4)
    Region {
        bone: Bones::Spine4,
        poly: &[
            (0.35, 0.21),
            (0.65, 0.21),
            (0.66, 0.30),
            (0.34, 0.30),
        ],
    },
    // Mid-chest (Spine3)
    Region {
        bone: Bones::Spine3,
        poly: &[
            (0.34, 0.30),
            (0.66, 0.30),
            (0.65, 0.38),
            (0.35, 0.38),
        ],
    },
    // Lower-chest (Spine2)
    Region {
        bone: Bones::Spine2,
        poly: &[
            (0.35, 0.38),
            (0.65, 0.38),
            (0.64, 0.45),
            (0.36, 0.45),
        ],
    },
    // Abdomen (Spine1)
    Region {
        bone: Bones::Spine1,
        poly: &[
            (0.36, 0.45),
            (0.64, 0.45),
            (0.62, 0.52),
            (0.38, 0.52),
        ],
    },
    // Hip / pelvis
    Region {
        bone: Bones::Hip,
        poly: &[
            (0.38, 0.52),
            (0.62, 0.52),
            (0.64, 0.60),
            (0.36, 0.60),
        ],
    },
    // ── Arms (left=character left, screen right) ──
    Region {
        bone: Bones::LeftShoulder,
        poly: &[
            (0.65, 0.22),
            (0.78, 0.24),
            (0.78, 0.32),
            (0.66, 0.30),
        ],
    },
    Region {
        bone: Bones::LeftElbow,
        poly: &[
            (0.78, 0.32),
            (0.86, 0.34),
            (0.85, 0.46),
            (0.76, 0.44),
        ],
    },
    Region {
        bone: Bones::LeftHand,
        poly: &[
            (0.85, 0.46),
            (0.92, 0.48),
            (0.92, 0.60),
            (0.84, 0.58),
        ],
    },
    Region {
        bone: Bones::RightShoulder,
        poly: &[
            (0.22, 0.24),
            (0.35, 0.22),
            (0.34, 0.30),
            (0.22, 0.32),
        ],
    },
    Region {
        bone: Bones::RightElbow,
        poly: &[
            (0.14, 0.34),
            (0.22, 0.32),
            (0.24, 0.44),
            (0.15, 0.46),
        ],
    },
    Region {
        bone: Bones::RightHand,
        poly: &[
            (0.08, 0.48),
            (0.15, 0.46),
            (0.16, 0.58),
            (0.08, 0.60),
        ],
    },
    // ── Legs ──
    Region {
        bone: Bones::LeftHip,
        poly: &[
            (0.50, 0.60),
            (0.64, 0.60),
            (0.63, 0.72),
            (0.51, 0.72),
        ],
    },
    Region {
        bone: Bones::LeftKnee,
        poly: &[
            (0.51, 0.72),
            (0.63, 0.72),
            (0.62, 0.84),
            (0.52, 0.84),
        ],
    },
    Region {
        bone: Bones::LeftFoot,
        poly: &[
            (0.52, 0.84),
            (0.62, 0.84),
            (0.62, 0.96),
            (0.51, 0.96),
        ],
    },
    Region {
        bone: Bones::RightHip,
        poly: &[
            (0.36, 0.60),
            (0.50, 0.60),
            (0.49, 0.72),
            (0.37, 0.72),
        ],
    },
    Region {
        bone: Bones::RightKnee,
        poly: &[
            (0.37, 0.72),
            (0.49, 0.72),
            (0.48, 0.84),
            (0.38, 0.84),
        ],
    },
    Region {
        bone: Bones::RightFoot,
        poly: &[
            (0.38, 0.84),
            (0.48, 0.84),
            (0.49, 0.96),
            (0.38, 0.96),
        ],
    },
];

/// Convert normalized poly to screen polygon inside `rect`.
fn map_poly(rect: Rect, poly: &[(f32, f32)]) -> Vec<Pos2> {
    poly.iter()
        .map(|(x, y)| pos2(rect.left() + x * rect.width(), rect.top() + y * rect.height()))
        .collect()
}

/// Point-in-polygon test (ray casting). All points 2D.
fn point_in_poly(p: Pos2, poly: &[Pos2]) -> bool {
    if poly.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = poly.len() - 1;
    for i in 0..poly.len() {
        let pi = poly[i];
        let pj = poly[j];
        let intersect = ((pi.y > p.y) != (pj.y > p.y))
            && (p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y + 0.0001) + pi.x);
        if intersect {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Renders an interactive body silhouette. Returns `true` if the selection changed.
/// The widget reads/writes the `bones` vector in place — clicking a region toggles
/// presence of that bone.
pub fn body_picker(ui: &mut Ui, bones: &mut Vec<Bones>) -> bool {
    let avail = ui.available_width().min(220.0);
    let size = vec2(avail, avail * 1.85);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    let accent = ui.visuals().hyperlink_color;

    // Background card behind silhouette
    painter.rect(
        rect,
        CornerRadius::same(12),
        Colors::BACKDROP,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 14)),
        StrokeKind::Inside,
    );

    let click_pos = if resp.clicked() {
        resp.interact_pointer_pos()
    } else {
        None
    };
    let hover_pos = ui.input(|i| i.pointer.hover_pos());

    let mut changed = false;
    let mut hovered_bone: Option<Bones> = None;

    // First pass: hit-test hover & click
    for r in REGIONS.iter() {
        let poly = map_poly(rect, r.poly);
        if let Some(p) = hover_pos {
            if rect.contains(p) && point_in_poly(p, &poly) {
                hovered_bone = Some(r.bone);
            }
        }
        if let Some(p) = click_pos {
            if rect.contains(p) && point_in_poly(p, &poly) {
                if let Some(idx) = bones.iter().position(|b| *b == r.bone) {
                    bones.remove(idx);
                } else {
                    bones.push(r.bone);
                }
                changed = true;
                // Don't break — allow multiple regions overlap? No, single click → first hit.
                break;
            }
        }
    }

    // Second pass: draw all regions
    for r in REGIONS.iter() {
        let poly = map_poly(rect, r.poly);
        let selected = bones.contains(&r.bone);
        let hovered = hovered_bone == Some(r.bone);

        let fill = if selected {
            Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 220)
        } else if hovered {
            Color32::from_rgba_unmultiplied(255, 255, 255, 38)
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 16)
        };
        let stroke = if hovered {
            Stroke::new(1.5, accent)
        } else {
            Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 30),
            )
        };

        painter.add(egui::Shape::convex_polygon(poly.clone(), fill, stroke));
    }

    // Tooltip for hovered region
    if let Some(b) = hovered_bone {
        let resp = resp.on_hover_ui(|ui| {
            ui.label(format!("{:?}", b));
        });
        let _ = resp;
    }

    changed
}

/// Public re-export of region count for debug/stats.
pub fn region_count() -> usize {
    REGIONS.len()
}

#[allow(dead_code)]
fn _force_use(_: Response) {}
